//! Bounded fair iteration scheduler. Backend futures must finish native work before returning.
use super::{
    choices::Choices,
    config::Config,
    kv::{Codec, Frame},
    model_ir::StepKind,
    ByteBudget, Charge,
};
use anyhow::{ensure, Result};
use std::{
    collections::{BTreeMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub enum Storage {
    Host(Codec),
    BackendManaged {
        fingerprint: [u8; 32],
        sequence_bytes: usize,
        experimental: bool,
    },
}
impl From<Codec> for Storage {
    fn from(c: Codec) -> Self {
        Self::Host(c)
    }
}
impl Storage {
    fn fingerprint(&self) -> [u8; 32] {
        match self {
            Self::Host(c) => c.fingerprint(),
            Self::BackendManaged { fingerprint, .. } => *fingerprint,
        }
    }
    fn experimental(&self) -> bool {
        match self {
            Self::Host(c) => c.experimental(),
            Self::BackendManaged { experimental, .. } => *experimental,
        }
    }
}
pub type ReleaseFuture<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
pub type BackendFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<StepOutput>>> + Send + 'a>>;
#[derive(Debug, Clone)]
pub struct BackendInfo {
    /// Must bind weights, tokenizer, adapters, positional configuration and numeric contract.
    pub identity: String,
    pub width: usize,
    pub vocabulary_size: u32,
    pub max_context_tokens: usize,
    /// Upper bound for transient step buffers, including encoded/decoded host staging.
    pub workspace_bytes: usize,
    pub max_in_flight: usize,
    pub eos_tokens: Vec<u32>,
}
pub trait ModelBackend: Send + 'static {
    fn info(&self) -> BackendInfo;
    fn supports_choices(&self) -> bool {
        false
    }
    /// History is immutable. State is committed by the host only after complete batch validation.
    /// The backend must retain native resources through completion, even if this future is dropped.
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_>;
    /// Discard per-sequence recurrent/temporary state; called after step completion.
    fn release(&mut self, sequence: u64) -> ReleaseFuture<'_>;
    /// Drain executor-owned work and release native parameter stores on orderly shutdown.
    fn shutdown(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}
#[derive(Debug)]
pub struct StepInput {
    pub kind: StepKind,
    pub sequence: u64,
    pub position: usize,
    pub tokens: Vec<u32>,
    pub prompt_complete: bool,
    pub history: Vec<Arc<Frame>>,
    pub allowed_tokens: Option<Vec<u32>>,
}
#[derive(Debug)]
pub struct KvValues {
    pub keys: Vec<i32>,
    pub values: Vec<i32>,
}
#[derive(Debug)]
pub struct StepOutput {
    pub sequence: u64,
    pub kv: Vec<KvValues>,
    pub next_token: Option<u32>,
    pub state_bytes: usize,
}
#[derive(Debug, serde::Serialize)]
pub struct Generation {
    pub sequence: u64,
    pub model_identity: String,
    pub codec_fingerprint: [u8; 32],
    pub config_fingerprint: String,
    pub output_contract_fingerprint: Option<String>,
    pub tokens: Vec<u32>,
    pub iterations: usize,
    /// Queue-inclusive monotonic offsets at validated token publication.
    pub token_times_ms: Vec<f64>,
    pub elapsed_ms: f64,
    pub experimental_codec: bool,
    pub finish_reason: String,
}
#[derive(Default)]
struct Counters {
    poisoned: AtomicBool,
    admitted: AtomicUsize,
    completed: AtomicUsize,
    failed: AtomicUsize,
    cancelled: AtomicUsize,
    iterations: AtomicUsize,
}
#[derive(Debug, serde::Serialize)]
pub struct Stats {
    pub admitted: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub iterations: usize,
    pub memory_used: usize,
    pub memory_peak: usize,
    pub admission_closed: bool,
}
struct Envelope {
    admitted_at: std::time::Instant,
    choices: Option<Choices>,
    id: u64,
    prompt: Vec<u32>,
    max_new: usize,
    cancelled: Arc<AtomicBool>,
    reply: oneshot::Sender<Result<Generation>>,
}
struct Sequence {
    token_times_ms: Vec<f64>,
    consumed: usize,
    managed_charge: Option<Charge>,
    envelope: Envelope,
    prefilled: usize,
    generated: Vec<u32>,
    history: Vec<Arc<Frame>>,
    iterations: usize,
}
pub struct Request {
    cancelled: Arc<AtomicBool>,
    reply: Option<oneshot::Receiver<Result<Generation>>>,
}
impl Request {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub async fn result(mut self) -> Result<Generation> {
        self.reply
            .take()
            .unwrap()
            .await
            .map_err(|_| anyhow::anyhow!("scheduler stopped before receipt"))?
    }
}
impl Drop for Request {
    fn drop(&mut self) {
        self.cancel();
    }
}
pub struct Engine {
    tx: Option<mpsc::Sender<Envelope>>,
    task: Option<tokio::task::JoinHandle<()>>,
    config: Config,
    supports_choices: bool,
    info: BackendInfo,
    next_id: AtomicU64,
    counters: Arc<Counters>,
    budget: ByteBudget,
}
impl Engine {
    pub fn new<B: ModelBackend>(
        config: Config,
        storage: impl Into<Storage>,
        backend: B,
    ) -> Result<Self> {
        let storage = storage.into();
        config.validate()?;
        let supports_choices = backend.supports_choices();
        let info = backend.info();
        ensure!(
            !info.identity.is_empty() && info.identity.len() <= 1024,
            "model identity required"
        );
        ensure!(
            match &storage {
                Storage::Host(c) => info.width == c.width(),
                Storage::BackendManaged { sequence_bytes, .. } =>
                    info.width == 0
                        && *sequence_bytes > 0
                        && *sequence_bytes <= config.model_memory_bytes,
            } && info.vocabulary_size > 0
                && info.max_context_tokens >= config.max_context_tokens,
            "backend geometry/capability mismatch"
        );
        ensure!(
            info.max_in_flight == config.max_in_flight,
            "backend native in-flight limit mismatch"
        );
        ensure!(
            info.workspace_bytes < config.model_memory_bytes,
            "workspace exceeds model memory budget"
        );
        ensure!(
            info.eos_tokens.len() <= 16
                && info.eos_tokens.iter().all(|t| *t < info.vocabulary_size),
            "invalid EOS tokens"
        );
        let budget = ByteBudget::new(config.model_memory_bytes)?;
        let counters = Arc::new(Counters::default());
        let (tx, rx) = mpsc::channel(config.max_queued);
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("Tokio runtime required"))?;
        let task = handle.spawn(run(
            config.clone(),
            storage,
            backend,
            rx,
            counters.clone(),
            budget.clone(),
        ));
        Ok(Self {
            tx: Some(tx),
            task: Some(task),
            config,
            supports_choices,
            info,
            next_id: AtomicU64::new(1),
            counters,
            budget,
        })
    }
    pub fn submit(&self, prompt: Vec<u32>, max_new: usize) -> Result<Request> {
        self.submit_inner(prompt, max_new, None)
    }
    pub fn submit_choices(
        &self,
        prompt: Vec<u32>,
        max_new: usize,
        paths: Vec<Vec<u32>>,
    ) -> Result<Request> {
        ensure!(
            self.supports_choices,
            "backend does not support output choices"
        );
        let choices = Choices::new(
            paths,
            self.info.vocabulary_size,
            &self.info.eos_tokens,
            max_new,
        )?;
        self.submit_inner(prompt, max_new, Some(choices))
    }
    fn submit_inner(
        &self,
        prompt: Vec<u32>,
        max_new: usize,
        choices: Option<Choices>,
    ) -> Result<Request> {
        ensure!(
            !self.counters.poisoned.load(Ordering::Acquire),
            "model admission closed after backend failure"
        );
        ensure!(
            !prompt.is_empty() && (1..=self.config.max_new_tokens).contains(&max_new),
            "prompt/generation limits"
        );
        ensure!(
            prompt
                .len()
                .checked_add(max_new - 1)
                .is_some_and(|n| n <= self.config.max_context_tokens),
            "context limit"
        );
        ensure!(
            prompt.iter().all(|t| *t < self.info.vocabulary_size),
            "token outside model vocabulary"
        );
        let id = self
            .next_id
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| anyhow::anyhow!("sequence identifier exhausted"))?;
        let (reply, rx) = oneshot::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        self.tx
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("scheduler closed"))?
            .try_send(Envelope {
                admitted_at: std::time::Instant::now(),
                choices,
                id,
                prompt,
                max_new,
                cancelled: cancelled.clone(),
                reply,
            })
            .map_err(|e| anyhow::anyhow!("scheduler admission: {e}"))?;
        self.counters.admitted.fetch_add(1, Ordering::Relaxed);
        Ok(Request {
            cancelled,
            reply: Some(rx),
        })
    }
    pub fn stats(&self) -> Stats {
        let mut result = stats(&self.counters, &self.budget);
        // A panicked worker closes admission too, even before a backend error latch was set.
        result.admission_closed |= self.tx.as_ref().is_some_and(|tx| tx.is_closed());
        result
    }
    pub async fn shutdown(mut self) -> Result<Stats> {
        self.tx.take();
        self.task.take().unwrap().await?;
        Ok(self.stats())
    }
}
fn stats(c: &Counters, b: &ByteBudget) -> Stats {
    Stats {
        admitted: c.admitted.load(Ordering::Relaxed),
        completed: c.completed.load(Ordering::Relaxed),
        failed: c.failed.load(Ordering::Relaxed),
        cancelled: c.cancelled.load(Ordering::Relaxed),
        iterations: c.iterations.load(Ordering::Relaxed),
        memory_used: b.used(),
        memory_peak: b.peak(),
        admission_closed: c.poisoned.load(Ordering::Acquire),
    }
}
struct Pending {
    sequence: Sequence,
    count: usize,
    prompt_complete: bool,
    charges: Vec<Charge>,
}
fn cancelled(s: &Sequence) -> bool {
    s.envelope.cancelled.load(Ordering::Acquire) || s.envelope.reply.is_closed()
}
async fn release<B: ModelBackend>(backend: &mut B, id: u64, c: &Counters) -> Result<()> {
    let result = backend.release(id).await;
    if result.is_err() {
        c.poisoned.store(true, Ordering::Release);
    }
    result
}
async fn fail<B: ModelBackend>(s: Sequence, error: &str, backend: &mut B, c: &Counters) {
    let cleanup = release(backend, s.envelope.id, c).await;
    c.failed.fetch_add(1, Ordering::Relaxed);
    let _ = s
        .envelope
        .reply
        .send(Err(anyhow::anyhow!("{error}; cleanup: {cleanup:?}")));
}
async fn run<B: ModelBackend>(
    config: Config,
    storage: Storage,
    mut backend: B,
    mut rx: mpsc::Receiver<Envelope>,
    c: Arc<Counters>,
    budget: ByteBudget,
) {
    let info = backend.info();
    let config_hash = config.fingerprint().expect("validated config");
    let mut active = VecDeque::new();
    loop {
        if c.poisoned.load(Ordering::Acquire) {
            rx.close();
            while let Some(s) = active.pop_front() {
                fail(s, "backend failure latch", &mut backend, &c).await;
            }
            while let Some(e) = rx.recv().await {
                c.failed.fetch_add(1, Ordering::Relaxed);
                let _ = e.reply.send(Err(anyhow::anyhow!("backend failure latch")));
            }
            break;
        }
        if active.is_empty() {
            match rx.recv().await {
                Some(e) => active.push_back(Sequence {
                    token_times_ms: Vec::new(),
                    envelope: e,
                    consumed: 0,
                    managed_charge: None,
                    prefilled: 0,
                    generated: Vec::new(),
                    history: Vec::new(),
                    iterations: 0,
                }),
                None => break,
            }
        }
        while active.len() < config.max_sequences {
            match rx.try_recv() {
                Ok(e) => active.push_back(Sequence {
                    token_times_ms: Vec::new(),
                    envelope: e,
                    consumed: 0,
                    managed_charge: None,
                    prefilled: 0,
                    generated: Vec::new(),
                    history: Vec::new(),
                    iterations: 0,
                }),
                Err(_) => break,
            }
        }
        let mut batch = Vec::new();
        let mut pending = Vec::new();
        let mut token_budget = config.max_batch_tokens;
        for _ in 0..active.len() {
            let mut s = active.pop_front().unwrap();
            if cancelled(&s) {
                let _ = release(&mut backend, s.envelope.id, &c).await;
                c.cancelled.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            if token_budget == 0 {
                active.push_back(s);
                continue;
            }
            let prefill = s.prefilled < s.envelope.prompt.len();
            let tokens = if prefill {
                s.envelope.prompt[s.prefilled..]
                    .iter()
                    .take(config.prefill_chunk_tokens.min(token_budget))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                vec![*s.generated.last().expect("decode after prefill")]
            };
            let count = tokens.len();
            if let Storage::BackendManaged { sequence_bytes, .. } = &storage {
                if s.managed_charge.is_none() {
                    match budget.reserve(*sequence_bytes) {
                        Ok(charge) => s.managed_charge = Some(charge),
                        Err(e) => {
                            fail(s, &e.to_string(), &mut backend, &c).await;
                            continue;
                        }
                    }
                }
            }
            let charges = match &storage {
                Storage::Host(codec) => (0..count)
                    .map(|_| codec.reserve_frame(&budget))
                    .collect::<Result<Vec<_>>>(),
                Storage::BackendManaged { .. } => Ok(Vec::new()),
            };
            let charges = match charges {
                Ok(v) => v,
                Err(e) => {
                    fail(s, &e.to_string(), &mut backend, &c).await;
                    continue;
                }
            };
            let prompt_complete = !prefill || s.prefilled + count == s.envelope.prompt.len();
            batch.push(StepInput {
                kind: if prefill {
                    StepKind::Prefill
                } else {
                    StepKind::Decode
                },
                sequence: s.envelope.id,
                position: s.consumed,
                tokens,
                prompt_complete,
                history: s.history.clone(),
                allowed_tokens: s
                    .envelope
                    .choices
                    .as_ref()
                    .map(|c| c.allowed(&s.generated, &info.eos_tokens)),
            });
            pending.push(Pending {
                sequence: s,
                count,
                prompt_complete,
                charges,
            });
            token_budget -= count;
        }
        if pending.is_empty() {
            tokio::task::yield_now().await;
            continue;
        }
        let workspace = match budget.reserve(info.workspace_bytes) {
            Ok(v) => v,
            Err(e) => {
                for p in pending {
                    fail(p.sequence, &e.to_string(), &mut backend, &c).await;
                }
                continue;
            }
        };
        c.iterations.fetch_add(1, Ordering::Relaxed);
        let result = backend.step(batch).await;
        // Native backend work must have completed here; cancellation never abandons this future.
        let validated = result.and_then(|outputs| {
            ensure!(outputs.len() == pending.len(), "backend batch cardinality");
            let mut by_id = BTreeMap::new();
            for o in outputs {
                ensure!(
                    by_id.insert(o.sequence, o).is_none(),
                    "duplicate backend sequence"
                );
            }
            for p in &pending {
                let o = by_id
                    .get(&p.sequence.envelope.id)
                    .ok_or_else(|| anyhow::anyhow!("foreign/missing backend sequence"))?;
                ensure!(
                    match &storage {
                        Storage::Host(codec) =>
                            o.state_bytes == 0
                                && o.kv.len() == p.count
                                && o.kv.iter().all(|v| v.keys.len() == codec.width()
                                    && v.values.len() == codec.width()),
                        Storage::BackendManaged { sequence_bytes, .. } =>
                            o.kv.is_empty() && o.state_bytes <= *sequence_bytes,
                    },
                    "backend KV shape"
                );
                ensure!(
                    o.next_token.is_some() == p.prompt_complete
                        && o.next_token.is_none_or(|t| t < info.vocabulary_size),
                    "backend token contract"
                );
                if let (Some(choices), Some(token)) = (&p.sequence.envelope.choices, o.next_token) {
                    ensure!(
                        choices
                            .allowed(&p.sequence.generated, &info.eos_tokens)
                            .contains(&token),
                        "backend output-choice violation"
                    );
                }
            }
            Ok(by_id)
        });
        match validated {
            Err(e) => {
                c.poisoned.store(true, Ordering::Release);
                for p in pending {
                    fail(p.sequence, &e.to_string(), &mut backend, &c).await;
                }
            }
            Ok(mut outputs) => {
                for p in pending {
                    let mut s = p.sequence;
                    if cancelled(&s) {
                        let _ = release(&mut backend, s.envelope.id, &c).await;
                        c.cancelled.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    let output = outputs.remove(&s.envelope.id).unwrap();
                    let frames = match &storage {
                        Storage::Host(codec) => output
                            .kv
                            .into_iter()
                            .zip(p.charges)
                            .map(|(v, charge)| codec.encode_reserved(&v.keys, &v.values, charge))
                            .collect::<Result<Vec<_>>>(),
                        Storage::BackendManaged { .. } => Ok(Vec::new()),
                    };
                    match frames {
                        Err(e) => fail(s, &e.to_string(), &mut backend, &c).await,
                        Ok(frames) => {
                            s.history.extend(frames);
                            s.consumed += p.count;
                            s.prefilled = (s.prefilled + p.count).min(s.envelope.prompt.len());
                            s.iterations += 1;
                            if let Some(token) = output.next_token {
                                s.generated.push(token);
                                s.token_times_ms
                                    .push(s.envelope.admitted_at.elapsed().as_secs_f64() * 1000.);
                            }
                            if s.generated.len() == s.envelope.max_new
                                || s.generated
                                    .last()
                                    .is_some_and(|t| info.eos_tokens.contains(t))
                            {
                                if let Err(e) = release(&mut backend, s.envelope.id, &c).await {
                                    c.failed.fetch_add(1, Ordering::Relaxed);
                                    let _ = s.envelope.reply.send(Err(e));
                                    continue;
                                }
                                let g = Generation {
                                    sequence: s.envelope.id,
                                    model_identity: info.identity.clone(),
                                    codec_fingerprint: storage.fingerprint(),
                                    config_fingerprint: config_hash.clone(),
                                    output_contract_fingerprint: s
                                        .envelope
                                        .choices
                                        .as_ref()
                                        .map(|c| c.fingerprint().to_owned()),
                                    tokens: s.generated.clone(),
                                    iterations: s.iterations,
                                    token_times_ms: s.token_times_ms,
                                    elapsed_ms: s.envelope.admitted_at.elapsed().as_secs_f64()
                                        * 1000.,
                                    experimental_codec: storage.experimental(),
                                    finish_reason: if s
                                        .generated
                                        .last()
                                        .is_some_and(|t| info.eos_tokens.contains(t))
                                    {
                                        "eos"
                                    } else {
                                        "length"
                                    }
                                    .into(),
                                };
                                // A concurrently dropped receiver is cancellation, not a delivered completion.
                                if s.envelope.reply.send(Ok(g)).is_ok() {
                                    c.completed.fetch_add(1, Ordering::Relaxed);
                                } else {
                                    c.cancelled.fetch_add(1, Ordering::Relaxed);
                                }
                            } else {
                                active.push_back(s);
                            }
                        }
                    }
                }
            }
        }
        drop(workspace);
        tokio::task::yield_now().await;
    }
    if backend.shutdown().await.is_err() {
        c.poisoned.store(true, Ordering::Release);
    }
}
