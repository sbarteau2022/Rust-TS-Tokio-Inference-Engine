//! Opt-in, worker-confined evaluated MLX state; snapshot serving remains the default.
use super::*;
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
}
impl DeviceState {
    pub fn position(&self) -> usize {
        self.token.as_ref().map_or(0, |t| t.id.position as usize)
    }
    pub fn bytes(&self) -> usize {
        self.token.as_ref().map_or(0, |t| t.id.bytes as usize)
    }
}
pub struct DeviceStep {
    pub state: DeviceState,
    pub logits: Vec<f32>,
    pub trace: Vec<f32>,
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
}
enum Work {
    Reference(
        ModelState,
        ModelStepIr,
        oneshot::Sender<Result<ModelStep>>,
        OwnedSemaphorePermit,
    ),
    Step {
        state: DeviceState,
        ir: ModelStepIr,
        reply: oneshot::Sender<Result<DeviceStep>>,
        _permit: OwnedSemaphorePermit,
    },
    Snapshot(DeviceState, oneshot::Sender<Result<Vec<f32>>>),
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
        let mut v = [0; 7];
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
}
impl QwenDeviceExecutor {
    pub async fn open(device: Device, slots: usize, logical_byte_limit: u64) -> Result<Self> {
        ensure!(
            (1..=64).contains(&slots)
                && logical_byte_limit > 0
                && logical_byte_limit <= 8 * 1024 * 1024 * 1024,
            "device resident budget bounds"
        );
        tokio::task::spawn_blocking(move || Self::start(device, slots, logical_byte_limit)).await?
    }
    fn start(device: Device, slots: usize, limit: u64) -> Result<Self> {
        let manifest = Manifest::read(Path::new(
            include_str!("../../fixtures/native-model.path").trim(),
        ))?;
        let architecture = Architecture::workhorse(&manifest)?;
        let identity = manifest.identity.clone();
        let program = HybridProgram::compile(40, 4)?;
        let owner = next_owner()?;
        let (tx, mut rx) = mpsc::channel(3);
        let (releases, mut retire) = mpsc::unbounded_channel();
        let worker_releases = releases.clone();
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
                let runtime = tokio::runtime::Builder::new_current_thread().build()?;
                let _ = ready.send(Ok(()));
                let mut faulted=false;
                runtime.block_on(async {
                    loop {
                        // Drain releases before accepting a new reservation.
                        while let Ok(id) = retire.try_recv() { session.release(&id)?; }
                        tokio::select! {
                            biased;
                            id = retire.recv() => { if let Some(id)=id { session.release(&id)?; } }
                            work = rx.recv() => {
                                let Some(work)=work else { break; };
                                match work {
                                    Work::Step { state,ir,reply,_permit } => {
                                        let usage=session.stats()?;
                                        let required=(architecture.state_len(state.position()+ir.len())*4) as u64;
                                        let result=if faulted {
                                            Err(anyhow::anyhow!("device admission closed"))
                                        }else if usage.residents>=slots as u64 || required>limit.saturating_sub(usage.logical_bytes) {
                                            Err(anyhow::anyhow!("device resident budget exhausted"))
                                        }else {
                                            let result=device_forward(&session,&manifest,architecture,owner,&worker_releases,&state,&ir);
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
                                    Work::Snapshot(state,reply) => {
                                        let result=(|| {
                                            let Some(token)=&state.token else { return Ok(Vec::new()); };
                                            let mut values=vec![0.;state.bytes()/4];let mut e=[0;1024];
                                            native_status(unsafe { rs_qwen_device_snapshot(session.0,&token.id,values.as_mut_ptr(),values.len(),e.as_mut_ptr(),e.len()) },&e)?;
                                            ensure!(values.iter().all(|v| v.is_finite()),"nonfinite exported state");Ok(values)
                                        })();let _=reply.send(result);
                                    }
                                    Work::Stats(reply) => { let _=reply.send(session.stats()); }
                                }
                            }
                        }
                    }
                    while let Ok(id)=retire.try_recv() { session.release(&id)?; }
                    let mut invalidated=0;let mut e=[0;1024];
                    native_status(unsafe {rs_qwen_device_drain(session.0,&mut invalidated,e.as_mut_ptr(),e.len())},&e)?;
                    let mut stats=session.stats()?;stats.invalidated_at_shutdown=invalidated;
                    ensure!(stats.residents==0 && stats.logical_bytes==0,"native device drain residual");
                    drop(session); // All arrays retired before weights and cleanup.
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
        })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn empty_state(&self) -> DeviceState {
        DeviceState {
            owner: self.owner,
            token: None,
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
                reply,
                _permit: permit,
            })
            .map_err(|_| anyhow::anyhow!("device work queue full/closed"))?;
        received.await.context("device worker stopped")?
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
    pub async fn snapshot(&self, state: &DeviceState) -> Result<Vec<f32>> {
        self.validate(state)?;
        let (reply, received) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Snapshot(state.clone(), reply))
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
    ir: &ModelStepIr,
) -> Result<DeviceStep> {
    manifest.verify_stamps()?;
    let mut id = StateId::default();
    let mut logits = vec![0.; c.vocab as usize];
    let mut trace = if ir.trace_enabled() {
        vec![0.; c.layers as usize * 5 * ir.len() * c.hidden as usize]
    } else {
        Vec::new()
    };
    let mut e = [0; 1024];
    native_status(
        unsafe {
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
        },
        &e,
    )?;
    // Establish retirement ownership before any subsequent fallible check.
    let next = DeviceState {
        owner,
        token: Some(Arc::new(Lifetime {
            id,
            releases: releases.clone(),
        })),
    };
    manifest.verify_stamps()?;
    ensure!(
        id.position == (state.position() + ir.len()) as u64
            && id.bytes == (c.state_len(id.position as usize) * 4) as u64
            && logits.iter().chain(&trace).all(|v| v.is_finite()),
        "invalid device receipt"
    );
    Ok(DeviceStep {
        state: next,
        logits,
        trace,
    })
}
unsafe extern "C" {
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
    #[tokio::test]
    async fn invalid_budget_rejects_before_loading_weights() {
        for (slots, bytes) in [(0, 1), (65, 1), (1, 0), (1, u64::MAX)] {
            assert!(QwenDeviceExecutor::open(Device::Cpu, slots, bytes)
                .await
                .is_err());
        }
    }
}
