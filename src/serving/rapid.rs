//! Explicit external MLX numerical baseline; the custom Metal backend remains integer-only.
use super::{
    config::Config,
    scheduler::{
        BackendFuture, BackendInfo, ModelBackend, ReleaseFuture, StepInput, StepOutput, Storage,
    },
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RapidConfig {
    pub version: u32,
    pub python: PathBuf,
    pub model: PathBuf,
    pub codec: String,
    pub sequence_bytes: usize,
    pub workspace_bytes: usize,
    pub allocator_limit_bytes: usize,
    pub rpc_timeout_seconds: u64,
    pub scheduler: Config,
}
impl RapidConfig {
    pub fn validate(&self) -> Result<()> {
        self.scheduler.validate()?;
        ensure!(
            self.version == 1
                && ["regular", "rapid-turboquant-v4", "rapid-turboquant-k8v4"]
                    .contains(&self.codec.as_str()),
            "unsupported model configuration/codec"
        );
        ensure!(
            self.python.is_absolute()
                && self.python.is_file()
                && self.model.is_absolute()
                && self.model.is_dir(),
            "explicit absolute Python/model paths required"
        );
        ensure!(
            self.sequence_bytes > 0
                && self
                    .sequence_bytes
                    .checked_add(self.workspace_bytes)
                    .is_some_and(|n| n <= self.scheduler.model_memory_bytes),
            "state/workspace reservations exceed budget"
        );
        ensure!(
            (1..=300).contains(&self.rpc_timeout_seconds),
            "RPC timeout outside 1..300 seconds"
        );
        ensure!(
            (1024usize.pow(3)..=40 * 1024usize.pow(3)).contains(&self.allocator_limit_bytes),
            "MLX allocator cap outside 1..40 GiB"
        );
        Ok(())
    }
}
struct Rpc {
    _child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    timeout: Duration,
    poisoned: bool,
}
impl Rpc {
    async fn call(&mut self, request: Value) -> Result<Value> {
        ensure!(!self.poisoned, "backend RPC is latched closed");
        let result = tokio::time::timeout(self.timeout, self.exchange(request)).await;
        match result {
            Ok(Ok(value)) => Ok(value),
            other => {
                self.poisoned = true;
                let _ = self._child.kill().await;
                match other {
                    Ok(Err(e)) => Err(e),
                    Err(_) => anyhow::bail!("model RPC timeout"),
                    _ => unreachable!(),
                }
            }
        }
    }
    async fn exchange(&mut self, request: Value) -> Result<Value> {
        let mut encoded = serde_json::to_vec(&request)?;
        ensure!(encoded.len() < 512 * 1024, "RPC request too large");
        encoded.push(b'\n');
        self.input.write_all(&encoded).await?;
        self.input.flush().await?;
        let mut line = Vec::new();
        loop {
            let chunk = self.output.fill_buf().await?;
            ensure!(!chunk.is_empty(), "backend EOF before receipt");
            let count = chunk
                .iter()
                .position(|b| *b == b'\n')
                .map_or(chunk.len(), |n| n + 1);
            ensure!(
                line.len() + count <= 2 * 1024 * 1024,
                "backend receipt exceeds 2 MiB"
            );
            line.extend_from_slice(&chunk[..count]);
            self.output.consume(count);
            if line.last() == Some(&b'\n') {
                break;
            }
        }
        let value: Value = serde_json::from_slice(&line).context("invalid model RPC JSON")?;
        ensure!(
            value.get("error").is_none(),
            "backend error: {}",
            value["error"]
        );
        Ok(value)
    }
}
#[derive(Clone)]
pub struct RapidReport(Arc<Mutex<Rpc>>);
impl RapidReport {
    pub async fn finish(&self, tokens: &[Vec<u32>]) -> Result<Value> {
        self.0
            .lock()
            .await
            .call(json!({"op":"finish","tokens":tokens}))
            .await
    }
}
pub struct RapidBackend {
    rpc: Arc<Mutex<Rpc>>,
    info: BackendInfo,
}
impl RapidBackend {
    pub async fn start(
        config: &RapidConfig,
        worker: &Path,
        prompts: &[String],
        probe_directory: Option<&Path>,
    ) -> Result<(Self, Storage, Vec<Vec<u32>>, Value, RapidReport)> {
        config.validate()?;
        ensure!(
            !prompts.is_empty()
                && prompts.len() <= 32
                && prompts.iter().all(|p| !p.is_empty() && p.len() <= 16384),
            "1..32 nonempty prompts of at most 16 KiB required"
        );
        let mut child = Command::new(&config.python)
            .arg(worker)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut rpc = Rpc {
            _child: child,
            input,
            output,
            timeout: Duration::from_secs(config.rpc_timeout_seconds),
            poisoned: false,
        };
        let ready=rpc.call(json!({"op":"init","model":config.model,"codec":config.codec,"allocator_limit_bytes":config.allocator_limit_bytes,"sequence_bytes":config.sequence_bytes,"max_sequences":config.scheduler.max_sequences,"max_context_tokens":config.scheduler.max_context_tokens,"prompts":prompts,"probe_directory":probe_directory})).await?;
        ensure!(ready["ready"] == true, "backend readiness missing");
        let identity = ready["identity"]
            .as_str()
            .context("model identity missing")?
            .to_string();
        let vocabulary_size = serde_json::from_value(ready["vocabulary_size"].clone())?;
        let eos_tokens = serde_json::from_value(ready["eos_tokens"].clone())?;
        let tokens: Vec<Vec<u32>> = serde_json::from_value(ready["prompts"].clone())?;
        ensure!(tokens.len() == prompts.len(), "backend prompt cardinality");
        let fingerprint = ready["codec_fingerprint"]
            .as_str()
            .context("codec fingerprint missing")?;
        ensure!(
            fingerprint.len() == 64 && fingerprint.bytes().all(|b| b.is_ascii_hexdigit()),
            "codec hash shape"
        );
        let mut hash = [0u8; 32];
        for (i, byte) in hash.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&fingerprint[i * 2..i * 2 + 2], 16)?;
        }
        let storage = Storage::BackendManaged {
            fingerprint: hash,
            sequence_bytes: config.sequence_bytes,
            experimental: config.codec != "regular",
        };
        let info = BackendInfo {
            identity,
            width: 0,
            vocabulary_size,
            max_context_tokens: config.scheduler.max_context_tokens,
            workspace_bytes: config.workspace_bytes,
            max_in_flight: config.scheduler.max_in_flight,
            eos_tokens,
        };
        let rpc = Arc::new(Mutex::new(rpc));
        let report = RapidReport(rpc.clone());
        Ok((Self { rpc, info }, storage, tokens, ready, report))
    }
}
impl ModelBackend for RapidBackend {
    fn info(&self) -> BackendInfo {
        self.info.clone()
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            ensure!(
                batch.iter().all(|v| v.history.is_empty()),
                "MLX backend cannot consume host Q1.31 history"
            );
            let batch:Vec<_>=batch.into_iter().map(|v|json!({"sequence":v.sequence,"position":v.position,"tokens":v.tokens,"prompt_complete":v.prompt_complete})).collect();
            let result = self
                .rpc
                .lock()
                .await
                .call(json!({"op":"step","batch":batch}))
                .await?;
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct WireOutput {
                sequence: u64,
                kv: Vec<Value>,
                next_token: Option<u32>,
                state_bytes: usize,
            }
            let outputs: Vec<WireOutput> = serde_json::from_value(result["outputs"].clone())?;
            outputs
                .into_iter()
                .map(|v| {
                    ensure!(v.kv.is_empty(), "unexpected host KV from MLX");
                    Ok(StepOutput {
                        sequence: v.sequence,
                        kv: Vec::new(),
                        next_token: v.next_token,
                        state_bytes: v.state_bytes,
                    })
                })
                .collect()
        })
    }
    fn release(&mut self, sequence: u64) -> ReleaseFuture<'_> {
        Box::pin(async move {
            let value = self
                .rpc
                .lock()
                .await
                .call(json!({"op":"release","sequence":sequence}))
                .await?;
            ensure!(value["released"] == true, "release acknowledgement missing");
            Ok(())
        })
    }
}
