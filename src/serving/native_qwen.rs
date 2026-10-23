//! Native complete hybrid text decoder. This reference prioritizes state correctness over copies.
use super::{
    completion_ledger::{Footprint, Lease, Ledger},
    model_ir::{HybridProgram, LayerIr, ModelStepIr, StepKind},
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
fn next_owner() -> Result<u64> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
        .map_err(|_| anyhow::anyhow!("model owner exhausted"))
}
#[derive(Clone)]
pub struct ModelState {
    owner: u64,
    offset: usize,
    values: Arc<Vec<f32>>,
}
impl ModelState {
    pub(super) fn snapshot_values(&self) -> &[f32] {
        &self.values
    }
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
    pub trace: Vec<f32>,
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
    ir: ModelStepIr,
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
    program: Arc<HybridProgram>,
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
        let program = Arc::new(HybridProgram::compile(
            architecture.layers as usize,
            architecture.interval as usize,
        )?);
        let worker_program = program.clone();
        let owner = next_owner()?;
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
                            &worker_program,
                            &command.ir,
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
            program,
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
        let ir = ModelStepIr::compile(
            StepKind::Prefill,
            state.position(),
            tokens,
            self.architecture.vocab as u32,
        )?;
        self.execute(state, ir).await
    }
    pub fn program(&self) -> &HybridProgram {
        &self.program
    }
    pub async fn execute(&self, state: &ModelState, ir: ModelStepIr) -> Result<ModelStep> {
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
            ir.position().get() == state.offset
                && ir.vocabulary() == c.vocab as u32
                && ir.len() <= c.chunk as usize
                && state.offset + ir.len() <= c.context as usize,
            "IR/state geometry mismatch"
        );
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("model in-flight limit reached")?;
        let next = c.state_len(state.offset + ir.len()) * 4;
        let trace_bytes = if ir.trace_enabled() {
            c.layers as usize * 5 * ir.len() * c.hidden as usize * 4
        } else {
            0
        };
        let lease = self.ledger.reserve(Footprint {
            host_bytes: state.bytes() + next + c.vocab as usize * 4 + trace_bytes,
            tensor_bytes: state.bytes() + next + 256 * 1024 * 1024 + trace_bytes,
            graph_nodes: 1,
        })?;
        let (reply, receive) = oneshot::channel();
        self.tx
            .as_ref()
            .context("model stopped")?
            .try_send(Command {
                ir,
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
    program: &HybridProgram,
    ir: &ModelStepIr,
    state: &ModelState,
) -> Result<ModelStep> {
    manifest.verify_stamps()?;
    let tokens = ir.ffi_ids();
    let offset = state.offset + tokens.len();
    let mut values = vec![0f32; c.state_len(offset)];
    let mut logits = vec![0f32; c.vocab as usize];
    let mut trace = if ir.trace_enabled() {
        vec![0.; c.layers as usize * 5 * tokens.len() * c.hidden as usize]
    } else {
        Vec::new()
    };
    let mut e = [0 as c_char; 1024];
    native_status(
        unsafe {
            rs_qwen_forward(
                weights.0,
                i32::from(device == Device::Gpu),
                &c,
                program.layers().as_ptr(),
                program.layers().len(),
                tokens.as_ptr(),
                tokens.len(),
                state.offset,
                state.values.as_ptr(),
                state.values.len(),
                values.as_mut_ptr(),
                values.len(),
                logits.as_mut_ptr(),
                logits.len(),
                ir.diagnostic_flags(),
                ir.projection_outputs().as_ptr(),
                ir.projection_outputs().len(),
                trace.as_mut_ptr(),
                trace.len(),
                e.as_mut_ptr(),
                e.len(),
            )
        },
        &e,
    )?;
    manifest.verify_stamps()?;
    ensure!(
        values
            .iter()
            .chain(&logits)
            .chain(&trace)
            .all(|v| v.is_finite()),
        "nonfinite model result"
    );
    Ok(ModelStep {
        state: ModelState {
            owner: state.owner,
            offset,
            values: Arc::new(values),
        },
        logits,
        trace,
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
        layers: *const LayerIr,
        layer_count: usize,
        tokens: *const u32,
        count: usize,
        offset: usize,
        old_state: *const f32,
        old_count: usize,
        new_state: *mut f32,
        new_count: usize,
        logits: *mut f32,
        logits_count: usize,
        projection_checks: i32,
        projection_outputs: *const u32,
        projection_output_count: usize,
        trace: *mut f32,
        trace_count: usize,
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
    fn supports_choices(&self) -> bool {
        true
    }
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
                if let Some(ids) = &item.allowed_tokens {
                    ensure!(
                        !ids.is_empty()
                            && ids.len() <= 32
                            && ids
                                .iter()
                                .all(|t| *t < self.executor.architecture.vocab as u32),
                        "invalid legal token mask"
                    );
                }
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
                let ir = ModelStepIr::compile(
                    item.kind,
                    item.position,
                    item.tokens,
                    self.executor.architecture.vocab as u32,
                )?;
                let result = self.executor.execute(&old, ir).await?;
                let token = if item.prompt_complete {
                    Some(super::choices::select(
                        &result.logits,
                        item.allowed_tokens.as_deref(),
                    )?)
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
#[cfg(test)]
#[path = "qwen_tests.rs"]
mod tests;

#[path = "native_device.rs"]
mod device;
pub use device::{
    DeviceBlock, DeviceState, DeviceStats, DeviceStep, MtpState, MtpStep, QwenDeviceExecutor,
};

#[path = "native_device_scheduler.rs"]
mod device_scheduler;
pub use device_scheduler::{ScheduledDevice, StateMode};

#[path = "native_speculative.rs"]
mod speculative;
pub use speculative::{GreedyCursor, GreedyTransaction, MtpCursor, MtpTransaction, SuffixDraft};

/// Ownership held by each native command until its completion.
pub type CompletionGuard = Arc<dyn Send + Sync>;

#[path = "native_automaton.rs"]
mod automaton;
pub use automaton::{
    AutomatonCursor, AutomatonMtp, AutomatonMtpCursor, AutomatonMtpStep, AutomatonProposer,
    AutomatonStep, DraftProposer, ProposalContext, ProposalFuture, RetrievalProposer,
};

#[path = "native_mtp_scheduler.rs"]
mod mtp_scheduler;
pub use mtp_scheduler::{ScheduledMtp, SeedMetrics, SeedSnapshot};
