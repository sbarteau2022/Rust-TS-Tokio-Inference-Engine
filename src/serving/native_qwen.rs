//! Native complete hybrid text decoder. This reference prioritizes state correctness over copies.
use super::{
    completion_ledger::{Footprint, Lease, Ledger},
    native_mlx::{Device, NativeWeights},
    scheduler::{
        BackendFuture, BackendInfo, ModelBackend, ReleaseFuture, StepInput, StepOutput, Storage,
    },
    weights::Manifest,
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{c_char, c_void, CStr, CString},
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread::JoinHandle,
};
use tokio::sync::{mpsc, oneshot, OwnedSemaphorePermit, Semaphore};
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum ActivationPrecision {
    Bf16 = 0,
    F32 = 1,
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Architecture {
    hidden: i32,
    layers: i32,
    vocab: i32,
    q_heads: i32,
    kv_heads: i32,
    head_dim: i32,
    key_heads: i32,
    value_heads: i32,
    key_dim: i32,
    value_dim: i32,
    conv: i32,
    interval: i32,
    experts: i32,
    top: i32,
    intermediate: i32,
    shared: i32,
    group: i32,
    bits: i32,
    context: i32,
    chunk: i32,
    rope_dims: i32,
    activation_precision: i32,
    epsilon: f32,
    theta: f32,
}
impl Architecture {
    fn workhorse(manifest: &Manifest) -> Result<Self> {
        let bytes = super::weights::bounded(&manifest.directory.join("config.json"), 1024 * 1024)?;
        ensure!(bytes.len() <= 1024 * 1024, "model config cap");
        let v = super::weights::strict(&bytes)?;
        let t = &v["text_config"];
        ensure!(
            v["model_type"] == "qwen3_5_moe" && t["model_type"] == "qwen3_5_moe_text",
            "unsupported model architecture"
        );
        ensure!(
            t["hidden_act"] == "silu"
                && (t["norm_topk_prob"].is_null() || t["norm_topk_prob"] == true)
                && (t["decoder_sparse_step"].is_null() || t["decoder_sparse_step"] == 1),
            "unsupported expert activation or routing"
        );
        let fields = [
            ("hidden_size", 2048),
            ("num_hidden_layers", 40),
            ("vocab_size", 248320),
            ("num_attention_heads", 16),
            ("num_key_value_heads", 2),
            ("head_dim", 256),
            ("linear_num_key_heads", 16),
            ("linear_num_value_heads", 32),
            ("linear_key_head_dim", 128),
            ("linear_value_head_dim", 128),
            ("linear_conv_kernel_dim", 4),
            ("full_attention_interval", 4),
            ("num_experts", 256),
            ("num_experts_per_tok", 8),
            ("moe_intermediate_size", 512),
            ("shared_expert_intermediate_size", 512),
        ];
        for (key, expected) in fields {
            ensure!(t[key].as_i64() == Some(expected), "unsupported {key}");
        }
        let layers = t["layer_types"].as_array().context("layer types missing")?;
        ensure!(layers.len() == 40, "layer type count");
        for (i, kind) in layers.iter().enumerate() {
            ensure!(
                kind == if (i + 1) % 4 == 0 {
                    "full_attention"
                } else {
                    "linear_attention"
                },
                "unsupported layer order"
            );
        }
        ensure!(
            t["rms_norm_eps"].as_f64() == Some(1e-6)
                && t["rope_parameters"]["rope_theta"].as_f64() == Some(10000000.)
                && t["partial_rotary_factor"].as_f64() == Some(0.25),
            "unsupported norm/RoPE"
        );
        ensure!(
            v["tie_word_embeddings"] == false
                && t["attention_bias"] == false
                && t["attn_output_gate"] == true,
            "unsupported projection semantics"
        );
        Ok(Self {
            hidden: 2048,
            layers: 40,
            vocab: 248320,
            q_heads: 16,
            kv_heads: 2,
            head_dim: 256,
            key_heads: 16,
            value_heads: 32,
            key_dim: 128,
            value_dim: 128,
            conv: 4,
            interval: 4,
            experts: 256,
            top: 8,
            intermediate: 512,
            shared: 512,
            group: manifest.quantization.group,
            bits: manifest.quantization.bits,
            context: 512,
            chunk: 8,
            rope_dims: 64,
            activation_precision: 1,
            epsilon: 1e-6,
            theta: 10000000.,
        })
    }
    fn state_len(self, offset: usize) -> usize {
        let full = (self.layers / self.interval) as usize;
        let linear = self.layers as usize - full;
        linear
            * ((self.conv - 1) as usize
                * (2 * self.key_heads * self.key_dim + self.value_heads * self.value_dim) as usize
                + (self.value_heads * self.value_dim * self.key_dim) as usize)
            + full * 2 * self.kv_heads as usize * offset * self.head_dim as usize
    }
}
#[derive(Clone)]
pub struct ModelState {
    owner: u64,
    offset: usize,
    values: Arc<Vec<f32>>,
}
impl ModelState {
    pub fn position(&self) -> usize {
        self.offset
    }
    pub fn bytes(&self) -> usize {
        self.values.len() * 4
    }
}
pub struct ModelStep {
    pub state: ModelState,
    pub logits: Vec<f32>,
}
impl ModelStep {
    pub fn greedy_token(&self) -> u32 {
        self.logits
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1).then_with(|| b.0.cmp(&a.0)))
            .unwrap()
            .0 as u32
    }
}
struct Command {
    tokens: Vec<u32>,
    state: ModelState,
    reply: oneshot::Sender<Result<ModelStep>>,
    _permit: OwnedSemaphorePermit,
    _lease: Lease,
}
pub struct QwenExecutor {
    tx: Option<mpsc::Sender<Command>>,
    worker: Option<JoinHandle<()>>,
    permit: Arc<Semaphore>,
    ledger: Ledger,
    closed: Arc<AtomicBool>,
    owner: u64,
    manifest: Arc<Manifest>,
    architecture: Architecture,
}
impl QwenExecutor {
    pub async fn open(device: Device) -> Result<Self> {
        Self::open_with_precision(device, ActivationPrecision::F32).await
    }
    pub async fn open_with_precision(
        device: Device,
        precision: ActivationPrecision,
    ) -> Result<Self> {
        tokio::task::spawn_blocking(move || {
            let path = Path::new(include_str!("../../fixtures/native-model.path").trim());
            let manifest = Arc::new(Manifest::read(path)?);
            let mut architecture = Architecture::workhorse(&manifest)?;
            architecture.activation_precision = precision as i32;
            Self::start(device, manifest, architecture)
        })
        .await?
    }
    fn start(device: Device, manifest: Arc<Manifest>, architecture: Architecture) -> Result<Self> {
        ensure!(
            architecture.state_len(architecture.context as usize) * 4 <= 128 * 1024 * 1024,
            "model state cap"
        );
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let owner = NEXT.fetch_add(1, Ordering::Relaxed);
        let ledger = Ledger::new(Footprint {
            host_bytes: 768 * 1024 * 1024,
            tensor_bytes: 1536 * 1024 * 1024,
            graph_nodes: 3,
        })?;
        let (tx, mut rx) = mpsc::channel::<Command>(3);
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let closed = Arc::new(AtomicBool::new(false));
        let worker_closed = closed.clone();
        let model = manifest.clone();
        let worker = std::thread::Builder::new()
            .name("rust-qwen-native".into())
            .spawn(move || {
                let weights = (|| {
                    let weights = NativeWeights::open(&model)?;
                    for (name, q) in &model.overrides {
                        let name = CString::new(name.as_str())?;
                        let mut e = [0 as c_char; 1024];
                        native_status(
                            unsafe {
                                rs_qwen_quant(
                                    weights.0,
                                    name.as_ptr(),
                                    q.group,
                                    q.bits,
                                    e.as_mut_ptr(),
                                    e.len(),
                                )
                            },
                            &e,
                        )?;
                    }
                    Ok::<_, anyhow::Error>(weights)
                })();
                let weights = match weights {
                    Ok(w) => w,
                    Err(e) => {
                        worker_closed.store(true, Ordering::Release);
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                let _ = ready_tx.send(Ok(()));
                while let Some(command) = rx.blocking_recv() {
                    let result = if worker_closed.load(Ordering::Acquire) {
                        Err(anyhow::anyhow!("model executor closed"))
                    } else {
                        forward(
                            &weights,
                            &model,
                            architecture,
                            device,
                            owner,
                            &command.tokens,
                            &command.state,
                        )
                    };
                    if result.is_err() {
                        worker_closed.store(true, Ordering::Release);
                    }
                    let Command {
                        reply,
                        _permit,
                        _lease,
                        ..
                    } = command;
                    drop(_lease);
                    drop(_permit);
                    let _ = reply.send(result);
                }
                worker_closed.store(true, Ordering::Release);
                drop(weights);
                unsafe {
                    rs_mlx_cleanup();
                }
            })?;
        if let Err(e) = ready_rx.recv().context("model initializer stopped")? {
            let _ = worker.join();
            return Err(e);
        }
        Ok(Self {
            tx: Some(tx),
            worker: Some(worker),
            permit: Arc::new(Semaphore::new(3)),
            ledger,
            closed,
            owner,
            manifest,
            architecture,
        })
    }
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn empty_state(&self) -> ModelState {
        ModelState {
            owner: self.owner,
            offset: 0,
            values: Arc::new(Vec::new()),
        }
    }
    pub async fn step(&self, state: &ModelState, tokens: Vec<u32>) -> Result<ModelStep> {
        ensure!(
            !self.closed.load(Ordering::Acquire),
            "model admission closed"
        );
        let c = self.architecture;
        ensure!(
            state.owner == self.owner && state.offset <= c.context as usize,
            "foreign model state"
        );
        ensure!(
            (state.offset == 0 && state.values.is_empty())
                || (state.offset > 0 && state.values.len() == c.state_len(state.offset)),
            "state shape mismatch"
        );
        ensure!(
            !tokens.is_empty()
                && tokens.len() <= c.chunk as usize
                && state.offset + tokens.len() <= c.context as usize
                && tokens.iter().all(|t| *t < c.vocab as u32),
            "model token/context limits"
        );
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("model in-flight limit reached")?;
        let next = c.state_len(state.offset + tokens.len()) * 4;
        let lease = self.ledger.reserve(Footprint {
            host_bytes: state.bytes() + next + c.vocab as usize * 4,
            tensor_bytes: state.bytes() + next + 256 * 1024 * 1024,
            graph_nodes: 1,
        })?;
        let (reply, receive) = oneshot::channel();
        self.tx
            .as_ref()
            .context("model stopped")?
            .try_send(Command {
                tokens,
                state: state.clone(),
                reply,
                _permit: permit,
                _lease: lease,
            })
            .map_err(|e| anyhow::anyhow!("model queue: {e}"))?;
        receive.await.context("model stopped before receipt")?
    }
    pub fn resources(&self) -> super::completion_ledger::Snapshot {
        self.ledger.snapshot()
    }
    async fn stop(&mut self) -> Result<()> {
        self.tx.take();
        if let Some(worker) = self.worker.take() {
            tokio::task::spawn_blocking(move || {
                worker
                    .join()
                    .map_err(|_| anyhow::anyhow!("model worker panicked"))
            })
            .await??;
        }
        Ok(())
    }
    pub async fn shutdown(mut self) -> Result<super::completion_ledger::Snapshot> {
        self.stop().await?;
        Ok(self.resources())
    }
}
impl Drop for QwenExecutor {
    fn drop(&mut self) {
        self.tx.take();
        self.worker.take();
    }
}
fn native_status(code: i32, e: &[c_char; 1024]) -> Result<()> {
    if code != 0 {
        anyhow::bail!(
            "native Qwen: {}",
            unsafe { CStr::from_ptr(e.as_ptr()) }.to_string_lossy()
        );
    }
    Ok(())
}
fn forward(
    weights: &NativeWeights,
    manifest: &Manifest,
    c: Architecture,
    device: Device,
    owner: u64,
    tokens: &[u32],
    state: &ModelState,
) -> Result<ModelStep> {
    manifest.verify_stamps()?;
    let offset = state.offset + tokens.len();
    let mut values = vec![0f32; c.state_len(offset)];
    let mut logits = vec![0f32; c.vocab as usize];
    let mut e = [0 as c_char; 1024];
    native_status(
        unsafe {
            rs_qwen_forward(
                weights.0,
                i32::from(device == Device::Gpu),
                &c,
                tokens.as_ptr(),
                tokens.len(),
                state.offset,
                state.values.as_ptr(),
                state.values.len(),
                values.as_mut_ptr(),
                values.len(),
                logits.as_mut_ptr(),
                logits.len(),
                e.as_mut_ptr(),
                e.len(),
            )
        },
        &e,
    )?;
    manifest.verify_stamps()?;
    ensure!(
        values.iter().chain(&logits).all(|v| v.is_finite()),
        "nonfinite model result"
    );
    Ok(ModelStep {
        state: ModelState {
            owner,
            offset,
            values: Arc::new(values),
        },
        logits,
    })
}
unsafe extern "C" {
    fn rs_qwen_quant(
        store: *mut c_void,
        module: *const c_char,
        group: i32,
        bits: i32,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_forward(
        store: *mut c_void,
        gpu: i32,
        config: *const Architecture,
        tokens: *const u32,
        count: usize,
        offset: usize,
        old_state: *const f32,
        old_count: usize,
        new_state: *mut f32,
        new_count: usize,
        logits: *mut f32,
        logits_count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_mlx_cleanup();
}
/// Scheduler adapter: validate every candidate before publishing any sequence state.
pub struct ScheduledQwen {
    executor: QwenExecutor,
    states: BTreeMap<u64, ModelState>,
    eos: Vec<u32>,
}
impl ScheduledQwen {
    pub fn new(executor: QwenExecutor, eos: Vec<u32>) -> Self {
        Self {
            executor,
            states: BTreeMap::new(),
            eos,
        }
    }
    pub fn storage(&self) -> Storage {
        Storage::BackendManaged {
            fingerprint: Sha256::digest(b"native-qwen-regular-kv-fp32-snapshot-v1").into(),
            sequence_bytes: self
                .executor
                .architecture
                .state_len(self.executor.architecture.context as usize)
                * 4,
            experimental: false,
        }
    }
}
impl ModelBackend for ScheduledQwen {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: format!(
                "{}:native-hybrid-precision{}-fp32-state-v1",
                self.executor.manifest.identity, self.executor.architecture.activation_precision
            ),
            width: 0,
            vocabulary_size: self.executor.architecture.vocab as u32,
            max_context_tokens: self.executor.architecture.context as usize,
            workspace_bytes: 512 * 1024 * 1024,
            max_in_flight: 3,
            eos_tokens: self.eos.clone(),
        }
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            ensure!(!batch.is_empty() && batch.len() <= 3, "model batch cap");
            let mut seen = BTreeSet::new();
            let new_count = batch
                .iter()
                .filter(|i| !self.states.contains_key(&i.sequence))
                .count();
            ensure!(self.states.len() + new_count <= 3, "resident sequence cap");
            for item in &batch {
                ensure!(
                    seen.insert(item.sequence) && item.history.is_empty(),
                    "duplicate row or external KV unsupported"
                );
                ensure!(
                    self.states.get(&item.sequence).map_or(0, |s| s.offset) == item.position,
                    "sequence position mismatch"
                );
            }
            let mut next = Vec::new();
            let mut outputs = Vec::new();
            for item in batch {
                let old = self
                    .states
                    .get(&item.sequence)
                    .cloned()
                    .unwrap_or_else(|| self.executor.empty_state());
                let result = self.executor.step(&old, item.tokens).await?;
                let token = if item.prompt_complete {
                    Some(result.greedy_token())
                } else {
                    None
                };
                outputs.push(StepOutput {
                    sequence: item.sequence,
                    kv: Vec::new(),
                    next_token: token,
                    state_bytes: result.state.bytes(),
                });
                next.push((item.sequence, result.state));
            }
            for (id, state) in next {
                self.states.insert(id, state);
            }
            Ok(outputs)
        })
    }
    fn release(&mut self, sequence: u64) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.states.remove(&sequence);
            Ok(())
        })
    }
    fn shutdown(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.states.clear();
            self.executor.stop().await
        })
    }
}
/// Local generation through the same fair scheduler used by serving requests.
pub async fn generate_cli(
    prompt: &str,
    max_new: usize,
    precision: ActivationPrecision,
    device: Device,
) -> Result<()> {
    use super::{config::Config, native_tokenizer::NativeTokenizer, scheduler::Engine};
    ensure!((1..=128).contains(&max_new), "generation limit 1..128");
    let started = std::time::Instant::now();
    let executor = QwenExecutor::open_with_precision(device, precision).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let ids = tokenizer.chat(None, prompt)?;
    println!("NATIVE-QWEN identity={} device={device:?} mode={precision:?}-activations/fp32-recurrence kv=regular max_in_flight=3 context=512 chunk=8 prompt_tokens={} load_ms={:.3}",executor.manifest.identity,ids.len(),started.elapsed().as_secs_f64()*1000.);
    let adapter = ScheduledQwen::new(executor, tokenizer.eos());
    let storage = adapter.storage();
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 512,
        max_new_tokens: 128,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    let engine = Engine::new(config, storage, adapter)?;
    let begin = std::time::Instant::now();
    let result = engine.submit(ids, max_new)?.result().await?;
    println!("{}", tokenizer.decode(&result.tokens)?);
    println!(
        "NATIVE-QWEN generated={} elapsed_ms={:.3} finish={} tokens={:?} held_out_qualified=false",
        result.tokens.len(),
        begin.elapsed().as_secs_f64() * 1000.,
        result.finish_reason,
        result.tokens
    );
    let stats = engine.shutdown().await?;
    ensure!(
        stats.memory_used == 0 && !stats.admission_closed,
        "scheduler did not drain cleanly"
    );
    Ok(())
}

#[cfg(test)]
#[path = "qwen_tests.rs"]
mod tests;
fn drift(a: &[f32], b: &[f32]) -> Result<(f64, f64)> {
    ensure!(a.len() == b.len() && !a.is_empty(), "drift shapes");
    let mut error = 0f64;
    let mut magnitude = 0f64;
    let mut max = 0f64;
    for (a, b) in a.iter().zip(b) {
        let d = *a as f64 - *b as f64;
        error += d * d;
        magnitude += (*a as f64) * (*a as f64);
        max = max.max(d.abs());
    }
    Ok(((error / magnitude.max(1e-30)).sqrt(), max))
}
/// Fixed local qualification fixtures; output is a plain text report, never a runtime JSON protocol.
pub async fn qualify_cli() -> Result<()> {
    use super::{config::Config, native_tokenizer::NativeTokenizer, scheduler::Engine};
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    println!("QUALIFICATION identity={} SDK={} profile=native-f32/fp32-state regular-kv context=512 chunk=8",executor.manifest.identity,super::native_mlx::version());
    let mut prefix =
        tokenizer.encode("The quick brown fox jumps over the lazy dog and continues walking.")?;
    prefix.truncate(8);
    ensure!(prefix.len() == 8, "continuation fixture too short");
    let full = executor
        .step(&executor.empty_state(), prefix.clone())
        .await?;
    let mut sequential = executor.empty_state();
    let mut last = None;
    for token in prefix {
        let result = executor.step(&sequential, vec![token]).await?;
        sequential = result.state.clone();
        last = Some(result);
    }
    let single = last.unwrap();
    let (logit_rmse, logit_max) = drift(&full.logits, &single.logits)?;
    let (state_rmse, state_max) = drift(&full.state.values, &single.state.values)?;
    let continuation =
        full.greedy_token() == single.greedy_token() && logit_rmse <= 0.01 && state_rmse <= 0.01;
    println!("CONTINUATION pass={continuation} top1_chunk={} top1_single={} logit_nrmse={logit_rmse:.8} logit_max_abs={logit_max:.8} state_nrmse={state_rmse:.8} state_max_abs={state_max:.8}",full.greedy_token(),single.greedy_token());
    drop(full);
    drop(single);
    drop(sequential);
    let backend = ScheduledQwen::new(executor, tokenizer.eos());
    let storage = backend.storage();
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 512,
        max_new_tokens: 32,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    let engine = Engine::new(config, storage, backend)?;
    let cases = include_str!("../../fixtures/native-heldout.tsv")
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split('\t').collect();
            ensure!(parts.len() == 3, "heldout row");
            Ok((parts[0], parts[1], parts[2]))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut passed = 0;
    let mut total = 0;
    for chunk in cases.chunks(3) {
        let mut pending = Vec::new();
        for (id, prompt, expected) in chunk {
            pending.push((
                *id,
                *expected,
                engine.submit(tokenizer.chat(None, prompt)?, 32)?,
            ));
        }
        for (id, expected, request) in pending {
            let result = request.result().await?;
            let text = tokenizer.decode(&result.tokens)?;
            let pass = text.trim() == expected && result.finish_reason == "eos";
            total += 1;
            if pass {
                passed += 1;
            }
            println!(
                "HELDOUT id={id} pass={pass} finish={} expected={expected:?} actual={text:?}",
                result.finish_reason
            );
        }
    }
    let stats = engine.shutdown().await?;
    let drain = stats.memory_used == 0 && !stats.admission_closed;
    println!("SUMMARY heldout={passed}/{total} continuation={continuation} drain={drain} scope=limited-smoke production_qualified=false");
    ensure!(
        continuation && passed == total && drain,
        "native qualification gate failed; retain report and do not promote"
    );
    Ok(())
}
