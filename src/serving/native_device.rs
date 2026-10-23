//! Opt-in, worker-confined evaluated MLX state; snapshot serving remains the default.
use super::*;
use std::future::Future;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StateId {
    owner: u64,
    slot: u64,
    generation: u64,
    position: u64,
    bytes: u64,
}
struct Lifetime {
    id: StateId,
    releases: mpsc::UnboundedSender<StateId>,
}
impl Drop for Lifetime {
    fn drop(&mut self) {
        // One nonblocking record per native version, independent of the work queue.
        let _ = self.releases.send(self.id);
    }
}
#[derive(Clone)]
pub struct DeviceState {
    owner: u64,
    token: Option<Arc<Lifetime>>,
    hidden_rows: usize,
}
impl DeviceState {
    /// Rows of post-normalized hidden activations owned by this exact version.
    pub fn hidden_rows(&self) -> usize {
        self.hidden_rows
    }
    pub fn position(&self) -> usize {
        self.token.as_ref().map_or(0, |t| t.id.position as usize)
    }
    pub fn bytes(&self) -> usize {
        self.token.as_ref().map_or(0, |t| t.id.bytes as usize)
    }
}
pub struct DeviceBlock {
    pub state: DeviceState,
    /// Position-major target logits; row i predicts the token after input i.
    pub logits: Vec<Vec<f32>>,
}
pub struct DeviceStep {
    pub state: DeviceState,
    pub logits: Vec<f32>,
    pub trace: Vec<f32>,
}
/// Host intervals; eval wait includes dispatch and is not GPU-only time.
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct ExecutionMetrics {
    pub calls: u64,
    pub completed: u64,
    pub graph_ns: u64,
    pub wait_ns: u64,
    pub validation_export_ns: u64,
    pub wall_ns: u64,
    pub completion_groups: u64,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct DeviceStats {
    pub residents: u64,
    /// Conservative sum of full logical state sizes, not unique GPU allocation bytes.
    pub logical_bytes: u64,
    pub exported_bytes: u64,
    pub uploaded_bytes: u64,
    pub steps: u64,
    /// Process-wide MLX allocator bytes, includes weights and other executors.
    pub mlx_active_bytes: u64,
    pub mlx_peak_bytes: u64,
    pub invalidated_at_shutdown: u64,
    pub execution: ExecutionMetrics,
    pub compiled_entries: u64,
    pub compiled_misses: u64,
    pub compiled_calls: u64,
}
#[path = "native_mtp_device.rs"]
mod mtp;
pub use mtp::{MtpState, MtpStep};
enum Work {
    DecodeBatch {
        parents: Vec<DeviceState>,
        tokens: Vec<u32>,
        reply: oneshot::Sender<Result<Vec<DeviceStep>>>,
        _permit: OwnedSemaphorePermit,
    },
    Mtp(mtp::MtpWork),
    MtpSnapshot(MtpState, oneshot::Sender<Result<Vec<f32>>>),
    Reference(
        ModelState,
        ModelStepIr,
        oneshot::Sender<Result<ModelStep>>,
        OwnedSemaphorePermit,
    ),
    Step {
        state: DeviceState,
        ir: ModelStepIr,
        all_logits: bool,
        capture_hidden: bool,
        _completion: Option<CompletionGuard>,
        reply: oneshot::Sender<Result<DeviceStep>>,
        _permit: OwnedSemaphorePermit,
    },
    Snapshot(DeviceState, bool, oneshot::Sender<Result<Vec<f32>>>),
    Stats(oneshot::Sender<Result<DeviceStats>>),
}
struct Session(*mut c_void);
impl Drop for Session {
    fn drop(&mut self) {
        unsafe { rs_qwen_device_destroy(self.0) };
    }
}
impl Session {
    fn release(&self, id: &StateId) -> Result<()> {
        let mut e = [0; 1024];
        native_status(
            unsafe { rs_qwen_device_release(self.0, id, e.as_mut_ptr(), e.len()) },
            &e,
        )
    }
    fn stats(&self) -> Result<DeviceStats> {
        let mut v = [0; 17];
        let mut e = [0; 1024];
        native_status(
            unsafe {
                rs_qwen_device_stats(self.0, v.as_mut_ptr(), v.len(), e.as_mut_ptr(), e.len())
            },
            &e,
        )?;
        Ok(DeviceStats {
            residents: v[0],
            logical_bytes: v[1],
            exported_bytes: v[2],
            uploaded_bytes: v[3],
            steps: v[4],
            mlx_active_bytes: v[5],
            mlx_peak_bytes: v[6],
            invalidated_at_shutdown: 0,
            compiled_entries: v[14],
            compiled_misses: v[15],
            compiled_calls: v[16],
            execution: ExecutionMetrics {
                calls: v[7],
                completed: v[8],
                graph_ns: v[9],
                wait_ns: v[10],
                validation_export_ns: v[11],
                wall_ns: v[12],
                completion_groups: v[13],
            },
        })
    }
}
pub struct QwenDeviceExecutor {
    tx: Option<mpsc::Sender<Work>>,
    worker: Option<JoinHandle<Result<DeviceStats>>>,
    closed: Arc<AtomicBool>,
    permit: Arc<Semaphore>,
    owner: u64,
    architecture: Architecture,
    identity: String,
    mtp_enabled: bool,
}
impl QwenDeviceExecutor {
    pub async fn open(device: Device, slots: usize, logical_byte_limit: u64) -> Result<Self> {
        Self::open_grouped(device, slots, logical_byte_limit, 1).await
    }
    /// Experimental completion grouping; arithmetic and full state checks are unchanged.
    pub async fn open_grouped(
        device: Device,
        slots: usize,
        logical_byte_limit: u64,
        layer_group: u32,
    ) -> Result<Self> {
        Self::open_tuned(device, slots, logical_byte_limit, layer_group, false).await
    }
    /// Experimental native recurrent primitive; snapshot control remains unchanged.
    pub async fn open_tuned(
        device: Device,
        slots: usize,
        logical_byte_limit: u64,
        layer_group: u32,
        fused_recurrent: bool,
    ) -> Result<Self> {
        Self::open_configured(
            device,
            slots,
            logical_byte_limit,
            ExecutionTuning {
                layer_group,
                fused_recurrent,
                execution_timing: false,
                compiled_decode: false,
            },
        )
        .await
    }
    pub async fn open_configured(
        device: Device,
        slots: usize,
        logical_byte_limit: u64,
        tuning: ExecutionTuning,
    ) -> Result<Self> {
        ensure!(
            !tuning.compiled_decode
                || (device == Device::Gpu && std::env::var_os("MLX_DISABLE_COMPILE").is_none()),
            "compiled decode requires GPU and no MLX_DISABLE_COMPILE"
        );
        let ExecutionTuning {
            layer_group,
            fused_recurrent,
            ..
        } = tuning;
        ensure!(
            !fused_recurrent || device == Device::Gpu,
            "fused recurrent requires GPU"
        );
        ensure!(
            matches!(layer_group, 1 | 4 | 10 | 40),
            "layer group must be 1|4|10|40"
        );
        ensure!(
            (1..=64).contains(&slots)
                && logical_byte_limit > 0
                && logical_byte_limit <= 8 * 1024 * 1024 * 1024,
            "device resident budget bounds"
        );
        tokio::task::spawn_blocking(move || {
            Self::start(device, slots, logical_byte_limit, false, tuning)
        })
        .await?
    }
    fn start(
        device: Device,
        slots: usize,
        limit: u64,
        mtp_enabled: bool,
        tuning: ExecutionTuning,
    ) -> Result<Self> {
        let manifest = Manifest::read(Path::new(
            include_str!("../../fixtures/native-model.path").trim(),
        ))?;
        let mut architecture = Architecture::workhorse(&manifest)?;
        architecture.layer_group = tuning.layer_group as i32;
        architecture.fused_recurrent = i32::from(tuning.fused_recurrent);
        architecture.execution_timing = i32::from(tuning.execution_timing);
        architecture.compiled_decode = i32::from(tuning.compiled_decode);
        let identity = manifest.identity.clone();
        let program = HybridProgram::compile(40, 4)?;
        let owner = next_owner()?;
        let (tx, mut rx) = mpsc::channel(3);
        let (releases, mut retire) = mpsc::unbounded_channel();
        let worker_releases = releases.clone();
        let (mtp_releases, mut mtp_retire) = mpsc::unbounded_channel();
        let worker_mtp_releases = mtp_releases.clone();
        let (ready, received) = std::sync::mpsc::sync_channel(1);
        let closed = Arc::new(AtomicBool::new(false));
        let worker_closed = closed.clone();
        let worker = std::thread::Builder::new().name("rust-qwen-device".into()).spawn(move || {
            let result = (|| -> Result<DeviceStats> {
                let weights = NativeWeights::open(&manifest)?;
                for (name,q) in &manifest.overrides {
                    let name = CString::new(name.as_str())?;
                    let mut e = [0;1024];
                    native_status(unsafe { rs_qwen_quant(weights.0,name.as_ptr(),q.group,q.bits,e.as_mut_ptr(),e.len()) }, &e)?;
                }
                let mut e = [0;1024];
                let handle = unsafe { rs_qwen_device_create(weights.0,i32::from(device==Device::Gpu),&architecture,program.layers().as_ptr(),program.layers().len(),slots,limit,e.as_mut_ptr(),e.len()) };
                ensure!(!handle.is_null(), "device session: {}", unsafe { CStr::from_ptr(e.as_ptr()) }.to_string_lossy());
                let session = Session(handle);
                let mtp_weights=if mtp_enabled {
                    let layout=super::super::native_mtp::MtpSidecar::read(&manifest)?;
                    let store=NativeWeights::open_mtp(&manifest,&layout)?;
                    for (name,q) in &manifest.overrides {if name.starts_with("language_model.mtp.") {
                        let name=CString::new(name.as_str())?;
                        native_status(unsafe{rs_qwen_quant(store.0,name.as_ptr(),q.group,q.bits,e.as_mut_ptr(),e.len())},&e)?;
                    }}
                    native_status(unsafe{mtp::rs_qwen_device_mtp_attach(session.0,store.0,e.as_mut_ptr(),e.len())},&e)?;
                    Some(store)
                }else{None};
                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let _ = ready.send(Ok(()));
                let mut faulted=false;
                runtime.block_on(async {
                    loop {
                        // Drain releases before accepting a new reservation.
                        while let Ok(id) = retire.try_recv() { session.release(&id)?; }
                        while let Ok(id)=mtp_retire.try_recv(){session.release_mtp(&id)?;}
                        tokio::select! {
                            biased;
                            id = retire.recv() => { if let Some(id)=id { session.release(&id)?; } }
                            id=mtp_retire.recv()=>{if let Some(id)=id{session.release_mtp(&id)?;}}
                            work = rx.recv() => {
                                let Some(work)=work else { break; };
                                match work {
                                    Work::DecodeBatch { parents,tokens,reply,_permit } => {
                                        let usage=session.stats()?;
                                        let b=parents.len() as u64;
                                        let charge=b*b*architecture.state_len(parents[0].position()+1) as u64*4;
                                        let result=if faulted {Err(anyhow::anyhow!("device admission closed"))}
                                        else if usage.residents+b>slots as u64 || charge>limit.saturating_sub(usage.logical_bytes) {Err(anyhow::anyhow!("batch resident budget exhausted"))}
                                        else {let result=batch_forward(&session,&manifest,architecture,owner,&worker_releases,&parents,&tokens);if result.is_err(){faulted=true;worker_closed.store(true,Ordering::Release);}result};
                                        drop(_permit);let _=reply.send(result);
                                    }
                                    Work::MtpSnapshot(state,reply)=>{let _=reply.send(mtp::snapshot(&session,&state));}
                                    Work::Mtp(work)=>{
                                        let usage=session.stats()?;
                                        let result=if faulted {Err(anyhow::anyhow!("device admission closed"))}
                                        else if usage.residents>=slots as u64 || work.required()>limit.saturating_sub(usage.logical_bytes){Err(anyhow::anyhow!("MTP resident budget exhausted"))}
                                        else {let result=mtp::forward(&session,&manifest,owner,&worker_mtp_releases,&work);if result.is_err(){faulted=true;worker_closed.store(true,Ordering::Release);}result};
                                        work.finish(result);
                                    }
                                    Work::Step { state,ir,all_logits,capture_hidden,_completion,reply,_permit } => {
                                        let usage=session.stats()?;
                                        let required=((architecture.state_len(state.position()+ir.len())+if capture_hidden {ir.len()*architecture.hidden as usize}else{0})*4) as u64;
                                        let result=if faulted {
                                            Err(anyhow::anyhow!("device admission closed"))
                                        }else if usage.residents>=slots as u64 || required>limit.saturating_sub(usage.logical_bytes) {
                                            Err(anyhow::anyhow!("device resident budget exhausted"))
                                        }else {
                                            let result=device_forward(&session,&manifest,architecture,owner,&worker_releases,&state,(&ir,all_logits,capture_hidden));
                                            if result.is_err(){faulted=true;worker_closed.store(true,Ordering::Release);}
                                            result
                                        };
                                        // Budget rejection preserves admission; computation faults latch closed.
                                        drop(_permit);
                                        let _ = reply.send(result); // Dropped result queues retirement.
                                    }
                                    Work::Reference(state,ir,reply,permit) => {
                                        let result=forward(&weights,&manifest,architecture,device,&program,&ir,&state);
                                        drop(permit);let _=reply.send(result);
                                    }
                                    Work::Snapshot(state,hidden,reply) => {
                                        let result=(|| {
                                            let Some(token)=&state.token else { return Ok(Vec::new()); };
                                            let mut values=vec![0.;if hidden {state.hidden_rows*architecture.hidden as usize}else{architecture.state_len(state.position())}];let mut e=[0;1024];
                                            native_status(unsafe { if hidden {rs_qwen_device_hidden_snapshot(session.0,&token.id,values.as_mut_ptr(),values.len(),e.as_mut_ptr(),e.len())}else{rs_qwen_device_snapshot(session.0,&token.id,values.as_mut_ptr(),values.len(),e.as_mut_ptr(),e.len())} },&e)?;
                                            ensure!(values.iter().all(|v| v.is_finite()),"nonfinite exported state");Ok(values)
                                        })();let _=reply.send(result);
                                    }
                                    Work::Stats(reply) => { let _=reply.send(session.stats()); }
                                }
                            }
                        }
                    }
                    while let Ok(id)=retire.try_recv() { session.release(&id)?; }
                    while let Ok(id)=mtp_retire.try_recv(){session.release_mtp(&id)?;}
                    let mut invalidated=0;let mut e=[0;1024];
                    native_status(unsafe {rs_qwen_device_drain(session.0,&mut invalidated,e.as_mut_ptr(),e.len())},&e)?;
                    let mut stats=session.stats()?;stats.invalidated_at_shutdown=invalidated;
                    ensure!(stats.residents==0 && stats.logical_bytes==0,"native device drain residual");
                    drop(session); // All arrays retired before weights and cleanup.
                    drop(mtp_weights);
                    Ok(stats)
                })
            })();
            worker_closed.store(true,Ordering::Release);
            if let Err(ref error)=result { let _=ready.send(Err(anyhow::anyhow!("{error:#}"))); }
            unsafe { rs_mlx_cleanup() };
            result
        })?;
        if let Err(error) = received.recv().context("device initializer stopped")? {
            let _ = worker.join();
            return Err(error);
        }
        Ok(Self {
            tx: Some(tx),
            worker: Some(worker),
            closed,
            permit: Arc::new(Semaphore::new(3)),
            owner,
            architecture,
            identity,
            mtp_enabled,
        })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn empty_state(&self) -> DeviceState {
        DeviceState {
            owner: self.owner,
            token: None,
            hidden_rows: 0,
        }
    }
    fn validate(&self, state: &DeviceState) -> Result<()> {
        ensure!(
            !self.closed.load(Ordering::Acquire) && state.owner == self.owner,
            "closed or foreign device state"
        );
        Ok(())
    }
    pub async fn step(&self, state: &DeviceState, tokens: Vec<u32>) -> Result<DeviceStep> {
        self.execute(
            state,
            ModelStepIr::compile(
                StepKind::Prefill,
                state.position(),
                tokens,
                self.architecture.vocab as u32,
            )?,
        )
        .await
    }
    pub async fn execute(&self, state: &DeviceState, ir: ModelStepIr) -> Result<DeviceStep> {
        self.execute_mode(state, ir, false, false, None, None).await
    }
    /// Explicit prototype: one batch-shaped graph, equal-position target decode only.
    /// Each child conservatively charges the entire packed backing until its release.
    pub fn submit_decode_batch(
        &self,
        parents: &[DeviceState],
        tokens: Vec<u32>,
    ) -> Result<impl Future<Output = Result<Vec<DeviceStep>>> + Send + 'static> {
        ensure!(
            !self.closed.load(Ordering::Acquire),
            "device admission closed"
        );
        validate_batch(self.owner, parents, &tokens, self.architecture.vocab as u32)?;
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("device in-flight limit reached")?;
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::DecodeBatch {
                parents: parents.to_vec(),
                tokens,
                reply,
                _permit: permit,
            })
            .map_err(|_| anyhow::anyhow!("device work queue full/closed"))?;
        Ok(async move { received.await.context("device worker stopped")? })
    }
    pub async fn decode_batch(
        &self,
        parents: &[DeviceState],
        tokens: Vec<u32>,
    ) -> Result<Vec<DeviceStep>> {
        self.submit_decode_batch(parents, tokens)?.await
    }
    /// Experimental verifier ABI; no candidate is committed by this call.
    pub async fn verify_block(&self, state: &DeviceState, tokens: Vec<u32>) -> Result<DeviceBlock> {
        self.verify_block_guarded(state, tokens, None, None).await
    }
    pub(super) async fn verify_block_guarded(
        &self,
        state: &DeviceState,
        tokens: Vec<u32>,
        completion: Option<CompletionGuard>,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<DeviceBlock> {
        let ir = ModelStepIr::compile(
            StepKind::Prefill,
            state.position(),
            tokens,
            self.architecture.vocab as u32,
        )?;
        let result = self
            .execute_mode(state, ir, true, false, completion, calls)
            .await?;
        Ok(DeviceBlock {
            state: result.state,
            logits: result
                .logits
                .chunks_exact(self.architecture.vocab as usize)
                .map(<[f32]>::to_vec)
                .collect(),
        })
    }
    /// Opt-in MTP input boundary: captures every post-norm row with unified state.
    pub async fn hidden_block(&self, state: &DeviceState, tokens: Vec<u32>) -> Result<DeviceBlock> {
        self.hidden_block_guarded(state, tokens, None, None).await
    }
    pub(super) async fn hidden_block_guarded(
        &self,
        state: &DeviceState,
        tokens: Vec<u32>,
        completion: Option<CompletionGuard>,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<DeviceBlock> {
        let ir = ModelStepIr::compile(
            StepKind::Prefill,
            state.position(),
            tokens,
            self.architecture.vocab as u32,
        )?;
        let result = self
            .execute_mode(state, ir, true, true, completion, calls)
            .await?;
        Ok(DeviceBlock {
            state: result.state,
            logits: result
                .logits
                .chunks_exact(self.architecture.vocab as usize)
                .map(<[f32]>::to_vec)
                .collect(),
        })
    }
    async fn execute_mode(
        &self,
        state: &DeviceState,
        ir: ModelStepIr,
        all_logits: bool,
        capture_hidden: bool,
        completion: Option<CompletionGuard>,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<DeviceStep> {
        self.validate(state)?;
        ensure!(
            ir.position().get() == state.position()
                && ir.vocabulary() == self.architecture.vocab as u32
                && ir.len() <= 8
                && state.position() + ir.len() <= 512,
            "device IR geometry"
        );
        ensure!(
            ir.diagnostic_flags() == 0,
            "device scalar probes require snapshot reference"
        );
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("device in-flight limit reached")?;
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Step {
                state: state.clone(),
                ir,
                all_logits,
                capture_hidden,
                _completion: completion,
                reply,
                _permit: permit,
            })
            .map_err(|_| anyhow::anyhow!("device work queue full/closed"))?;
        if let Some(calls) = calls {
            calls.fetch_add(1, Ordering::Relaxed);
        }
        received.await.context("device worker stopped")?
    }
    pub(super) async fn step_guarded(
        &self,
        state: &DeviceState,
        tokens: Vec<u32>,
        completion: Option<CompletionGuard>,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<DeviceStep> {
        let ir = ModelStepIr::compile(
            StepKind::Prefill,
            state.position(),
            tokens,
            self.architecture.vocab as u32,
        )?;
        self.execute_mode(state, ir, false, false, completion, calls)
            .await
    }
    /// Offline A/B only: share immutable weights with the CPU-snapshot reference.
    pub fn reference_empty_state(&self) -> ModelState {
        ModelState {
            owner: self.owner,
            offset: 0,
            values: Arc::new(Vec::new()),
        }
    }
    pub async fn reference_execute(
        &self,
        state: &ModelState,
        ir: ModelStepIr,
    ) -> Result<ModelStep> {
        ensure!(
            !self.closed.load(Ordering::Acquire)
                && state.owner == self.owner
                && ir.position().get() == state.offset
                && ir.vocabulary() == self.architecture.vocab as u32
                && ir.len() <= 8
                && state.offset + ir.len() <= 512,
            "reference IR ownership/geometry"
        );
        ensure!(
            (state.offset == 0 && state.values.is_empty())
                || (state.offset > 0
                    && state.values.len() == self.architecture.state_len(state.offset)),
            "reference state shape"
        );
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("reference in-flight limit reached")?;
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Reference(state.clone(), ir, reply, permit))
            .map_err(|_| anyhow::anyhow!("reference queue full/closed"))?;
        received.await.context("reference worker stopped")?
    }
    pub fn reference_values<'a>(&self, state: &'a ModelState) -> Result<&'a [f32]> {
        ensure!(state.owner == self.owner, "foreign reference state");
        Ok(&state.values)
    }
    pub async fn hidden_snapshot(&self, state: &DeviceState) -> Result<Vec<f32>> {
        ensure!(state.hidden_rows > 0, "state has no hidden activations");
        self.snapshot_mode(state, true).await
    }
    pub async fn snapshot(&self, state: &DeviceState) -> Result<Vec<f32>> {
        self.snapshot_mode(state, false).await
    }
    async fn snapshot_mode(&self, state: &DeviceState, hidden: bool) -> Result<Vec<f32>> {
        self.validate(state)?;
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Snapshot(state.clone(), hidden, reply))
            .map_err(|_| anyhow::anyhow!("device work queue full/closed"))?;
        received.await.context("snapshot worker stopped")?
    }
    pub async fn stats(&self) -> Result<DeviceStats> {
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Stats(reply))
            .map_err(|_| anyhow::anyhow!("device work queue full/closed"))?;
        received.await.context("stats worker stopped")?
    }
    pub async fn shutdown(mut self) -> Result<DeviceStats> {
        self.closed.store(true, Ordering::Release);
        self.tx.take();
        let worker = self.worker.take().context("device already stopped")?;
        tokio::task::spawn_blocking(move || {
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("device worker panicked"))?
        })
        .await?
    }
}
impl Drop for QwenDeviceExecutor {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Release);
        self.tx.take();
        self.worker.take();
    }
}
fn device_forward(
    session: &Session,
    manifest: &Manifest,
    c: Architecture,
    owner: u64,
    releases: &mpsc::UnboundedSender<StateId>,
    state: &DeviceState,
    mode: (&ModelStepIr, bool, bool),
) -> Result<DeviceStep> {
    let (ir, all_logits, capture_hidden) = mode;
    manifest.verify_stamps()?;
    let mut id = StateId::default();
    let mut logits = vec![0.; c.vocab as usize * if all_logits { ir.len() } else { 1 }];
    let mut trace = if ir.trace_enabled() {
        vec![0.; c.layers as usize * 5 * ir.len() * c.hidden as usize]
    } else {
        Vec::new()
    };
    let mut e = [0; 1024];
    native_status(
        unsafe {
            if capture_hidden {
                rs_qwen_device_hidden_block(
                    session.0,
                    state.token.as_ref().map_or(std::ptr::null(), |t| &t.id),
                    state.position(),
                    ir.ffi_ids().as_ptr(),
                    ir.len(),
                    &mut id,
                    logits.as_mut_ptr(),
                    logits.len(),
                    e.as_mut_ptr(),
                    e.len(),
                )
            } else if all_logits {
                rs_qwen_device_block(
                    session.0,
                    state.token.as_ref().map_or(std::ptr::null(), |t| &t.id),
                    state.position(),
                    ir.ffi_ids().as_ptr(),
                    ir.len(),
                    &mut id,
                    logits.as_mut_ptr(),
                    logits.len(),
                    e.as_mut_ptr(),
                    e.len(),
                )
            } else {
                rs_qwen_device_step(
                    session.0,
                    state.token.as_ref().map_or(std::ptr::null(), |t| &t.id),
                    state.position(),
                    ir.ffi_ids().as_ptr(),
                    ir.len(),
                    &mut id,
                    logits.as_mut_ptr(),
                    logits.len(),
                    trace.as_mut_ptr(),
                    trace.len(),
                    e.as_mut_ptr(),
                    e.len(),
                )
            }
        },
        &e,
    )?;
    // Establish retirement ownership before any subsequent fallible check.
    let next = DeviceState {
        owner,
        hidden_rows: if capture_hidden { ir.len() } else { 0 },
        token: Some(Arc::new(Lifetime {
            id,
            releases: releases.clone(),
        })),
    };
    manifest.verify_stamps()?;
    ensure!(
        id.position == (state.position() + ir.len()) as u64
            && id.bytes
                == ((c.state_len(id.position as usize)
                    + if capture_hidden {
                        ir.len() * c.hidden as usize
                    } else {
                        0
                    })
                    * 4) as u64
            && logits.iter().chain(&trace).all(|v| v.is_finite()),
        "invalid device receipt"
    );
    Ok(DeviceStep {
        state: next,
        logits,
        trace,
    })
}
fn validate_batch(
    owner: u64,
    parents: &[DeviceState],
    tokens: &[u32],
    vocabulary: u32,
) -> Result<()> {
    ensure!(
        !parents.is_empty() && parents.len() <= 3 && parents.len() == tokens.len(),
        "batch row geometry"
    );
    let position = parents[0].position();
    ensure!((1..512).contains(&position), "batch decode position");
    ensure!(
        parents
            .iter()
            .zip(tokens)
            .all(|(p, t)| p.owner == owner && p.position() == position && *t < vocabulary),
        "batch parent owner/token geometry"
    );
    Ok(())
}

#[cfg(test)]
mod batch_tests {
    use super::*;
    fn parent(owner: u64, position: u64) -> DeviceState {
        let (releases, _) = mpsc::unbounded_channel();
        DeviceState {
            owner,
            hidden_rows: 0,
            token: Some(Arc::new(Lifetime {
                id: StateId {
                    owner,
                    position,
                    slot: 0,
                    generation: 1,
                    bytes: 4,
                },
                releases,
            })),
        }
    }
    #[test]
    fn batch_descriptor_rejects_foreign_owner_geometry_and_vocabulary_before_submission() {
        assert!(validate_batch(1, &[parent(1, 8), parent(1, 8)], &[1, 2], 16).is_ok());
        for (parents, tokens) in [
            (vec![], vec![]),
            (vec![parent(1, 8)], vec![]),
            (vec![parent(2, 8)], vec![1]),
            (vec![parent(1, 0)], vec![1]),
            (vec![parent(1, 512)], vec![1]),
            (vec![parent(1, 8)], vec![16]),
            (vec![parent(1, 8), parent(1, 9)], vec![1, 2]),
            (vec![parent(1, 8); 4], vec![1; 4]),
        ] {
            assert!(validate_batch(1, &parents, &tokens, 16).is_err());
        }
    }
}

fn batch_forward(
    session: &Session,
    manifest: &Manifest,
    c: Architecture,
    owner: u64,
    releases: &mpsc::UnboundedSender<StateId>,
    parents: &[DeviceState],
    tokens: &[u32],
) -> Result<Vec<DeviceStep>> {
    manifest.verify_stamps()?;
    let old: Vec<_> = parents
        .iter()
        .map(|p| {
            p.token
                .as_ref()
                .expect("validated nonempty batch parent")
                .id
        })
        .collect();
    let mut ids = vec![StateId::default(); parents.len()];
    let mut logits = vec![0.; parents.len() * c.vocab as usize];
    let mut e = [0; 1024];
    native_status(
        unsafe {
            rs_qwen_device_decode_batch(
                session.0,
                old.as_ptr(),
                tokens.as_ptr(),
                parents.len(),
                ids.as_mut_ptr(),
                logits.as_mut_ptr(),
                logits.len(),
                e.as_mut_ptr(),
                e.len(),
            )
        },
        &e,
    )?;
    // Acquire retirement ownership for every child before any fallible receipt validation.
    let states: Vec<_> = ids
        .iter()
        .map(|id| DeviceState {
            owner,
            hidden_rows: 0,
            token: Some(Arc::new(Lifetime {
                id: *id,
                releases: releases.clone(),
            })),
        })
        .collect();
    manifest.verify_stamps()?;
    let position = parents[0].position() + 1;
    ensure!(
        ids.iter().all(|id| id.position == position as u64
            && id.bytes == (parents.len() * c.state_len(position) * 4) as u64)
            && logits.iter().all(|v| v.is_finite()),
        "invalid batch receipt"
    );
    Ok(states
        .into_iter()
        .zip(logits.chunks_exact(c.vocab as usize))
        .map(|(state, logits)| DeviceStep {
            state,
            logits: logits.to_vec(),
            trace: Vec::new(),
        })
        .collect())
}
unsafe extern "C" {
    fn rs_qwen_device_decode_batch(
        session: *mut c_void,
        parents: *const StateId,
        tokens: *const u32,
        batch: usize,
        outputs: *mut StateId,
        logits: *mut f32,
        logits_count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_create(
        weights: *mut c_void,
        gpu: i32,
        config: *const Architecture,
        layers: *const LayerIr,
        count: usize,
        slots: usize,
        bytes: u64,
        error: *mut c_char,
        cap: usize,
    ) -> *mut c_void;
    fn rs_qwen_device_step(
        session: *mut c_void,
        previous: *const StateId,
        position: usize,
        tokens: *const u32,
        count: usize,
        output: *mut StateId,
        logits: *mut f32,
        logits_count: usize,
        trace: *mut f32,
        trace_count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_block(
        session: *mut c_void,
        previous: *const StateId,
        position: usize,
        tokens: *const u32,
        count: usize,
        output: *mut StateId,
        logits: *mut f32,
        logits_count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_hidden_block(
        session: *mut c_void,
        previous: *const StateId,
        position: usize,
        tokens: *const u32,
        count: usize,
        output: *mut StateId,
        logits: *mut f32,
        logits_count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_hidden_snapshot(
        session: *mut c_void,
        id: *const StateId,
        output: *mut f32,
        count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_release(
        session: *mut c_void,
        id: *const StateId,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_snapshot(
        session: *mut c_void,
        id: *const StateId,
        output: *mut f32,
        count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_stats(
        session: *mut c_void,
        output: *mut u64,
        count: usize,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_drain(
        session: *mut c_void,
        invalidated: *mut u64,
        error: *mut c_char,
        cap: usize,
    ) -> i32;
    fn rs_qwen_device_destroy(session: *mut c_void);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_final_clone_retires_and_closed_worker_is_safe() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let id = StateId {
            owner: 7,
            slot: 2,
            generation: 9,
            position: 8,
            bytes: 64,
        };
        let state = DeviceState {
            owner: 3,
            hidden_rows: 0,
            token: Some(Arc::new(Lifetime { id, releases: tx })),
        };
        let command_reader = state.clone();
        let fork = state.clone();
        drop(state);
        drop(fork);
        assert!(rx.try_recv().is_err());
        drop(command_reader);
        assert_eq!(rx.try_recv().unwrap(), id);
        assert!(rx.try_recv().is_err());
        let (tx, rx) = mpsc::unbounded_channel();
        drop(rx);
        drop(Lifetime { id, releases: tx }); // Last ref may safely outlive worker shutdown.
    }
    #[test]
    fn release_records_survive_full_work_queue_and_dropped_receipt() {
        let (work, mut queued) = mpsc::channel::<u8>(3);
        for i in 0..3 {
            work.try_send(i).unwrap();
        }
        assert!(work.try_send(4).is_err());
        let (tx, mut rx) = mpsc::unbounded_channel();
        for generation in 1..=32_000 {
            let id = StateId {
                owner: 1,
                slot: generation % 64,
                generation,
                position: 1,
                bytes: 4,
            };
            let token = Arc::new(Lifetime {
                id,
                releases: tx.clone(),
            });
            let (reply, receive) = oneshot::channel();
            drop(receive);
            let _ = reply.send(token); // Unpublished output ownership retires once.
            assert_eq!(rx.try_recv().unwrap(), id);
        }
        assert!(rx.try_recv().is_err());
        for i in 0..3 {
            assert_eq!(queued.try_recv().unwrap(), i);
        }
    }
    #[test]
    fn native_command_retains_completion_guard_after_runtime_abort() {
        struct Marker(Arc<AtomicBool>);
        impl Drop for Marker {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let dropped = Arc::new(AtomicBool::new(false));
        let marker = dropped.clone();
        let (sent, received) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let permit = Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap();
                let (reply, receive) = oneshot::channel();
                let guard: CompletionGuard = Arc::new(Marker(marker));
                let work = Work::Step {
                    state: DeviceState {
                        owner: 1,
                        token: None,
                        hidden_rows: 0,
                    },
                    ir: ModelStepIr::compile(StepKind::Decode, 0, vec![1], 2).unwrap(),
                    all_logits: false,
                    capture_hidden: false,
                    _completion: Some(guard.clone()),
                    reply,
                    _permit: permit,
                };
                tokio::spawn(async move {
                    let _guard = guard;
                    let _ = receive.await;
                });
                sent.send(work)
                    .unwrap_or_else(|_| panic!("test worker disconnected"));
                tokio::task::yield_now().await;
            });
            drop(runtime); // Aborts waiter while the native command remains owned.
        })
        .join()
        .unwrap();
        let work = received.recv().unwrap();
        assert!(!dropped.load(Ordering::SeqCst));
        drop(work);
        assert!(dropped.load(Ordering::SeqCst));
    }
    #[tokio::test]
    async fn compiled_decode_rejects_cpu_before_loading_weights() {
        assert!(QwenDeviceExecutor::open_configured(
            Device::Cpu,
            1,
            1,
            ExecutionTuning {
                compiled_decode: true,
                ..ExecutionTuning::default()
            },
        )
        .await
        .is_err());
    }
    #[tokio::test]
    async fn invalid_budget_rejects_before_loading_weights() {
        for (slots, bytes) in [(0, 1), (65, 1), (1, 0), (1, u64::MAX)] {
            assert!(QwenDeviceExecutor::open(Device::Cpu, slots, bytes)
                .await
                .is_err());
        }
    }
}
