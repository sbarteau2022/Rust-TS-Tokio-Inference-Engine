//! MTP state handles are private to the same native worker as target state.
use super::*;
struct MtpLifetime {
    id: StateId,
    releases: mpsc::UnboundedSender<StateId>,
}
impl Drop for MtpLifetime {
    fn drop(&mut self) {
        let _ = self.releases.send(self.id);
    }
}
#[derive(Clone)]
pub struct MtpState {
    owner: u64,
    token: Option<Arc<MtpLifetime>>,
    rows: usize,
}
impl MtpState {
    pub(super) fn is_owned_by(&self, owner: u64) -> bool {
        self.owner == owner
    }
    pub fn position(&self) -> usize {
        self.token.as_ref().map_or(0, |s| s.id.position as usize)
    }
    pub fn bytes(&self) -> usize {
        self.token.as_ref().map_or(0, |s| s.id.bytes as usize)
    }
}
#[derive(Clone)]
pub struct MtpStep {
    pub state: MtpState,
    pub logits: Vec<f32>,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RawMtpTxResult {
    pub target_state: StateId,
    pub draft_state: StateId,
    pub emitted: [u32; 9],
    pub emitted_count: u32,
    pub learned_proposed: u32,
    pub learned_accepted: u32,
    pub target_calls: u32,
    pub draft_calls: u32,
    pub reused_target_tokens: u32,
    pub replayed_target_tokens: u32,
    pub target_forward_tokens: u32,
    pub terminal: u32,
}
pub(in crate::serving::native_qwen) struct FusedMtpReceipt {
    pub target_state: DeviceState,
    pub target_logits: Vec<f32>,
    pub draft_step: MtpStep,
    pub emitted: Vec<u32>,
    pub learned_proposed: usize,
    pub learned_accepted: usize,
    pub target_calls: usize,
    pub draft_calls: usize,
    pub reused_target_tokens: usize,
    pub replayed_target_tokens: usize,
    pub target_forward_tokens: usize,
    pub terminal: bool,
}
pub(super) struct MtpFusedWork {
    pub target_previous: DeviceState,
    pub draft_previous: MtpState,
    pub seed_token: u32,
    pub first_draft_token: u32,
    pub first_draft_margin: f32,
    pub width: usize,
    pub output_budget: usize,
    pub eos: Vec<u32>,
    pub reuse: bool,
    pub prepared: Option<Vec<u32>>,
    pub calls: Option<Arc<std::sync::atomic::AtomicU64>>,
    pub reply: oneshot::Sender<Result<FusedMtpReceipt>>,
    pub _permit: OwnedSemaphorePermit,
    pub _completion: Option<CompletionGuard>,
}
enum Source {
    Target(DeviceState),
    Draft(MtpState),
}
pub(super) struct MtpWork {
    previous: MtpState,
    source: Source,
    row: usize,
    tokens: Vec<u32>,
    context_cap: usize,
    diagnostics: bool,
    pub(super) reply: oneshot::Sender<Result<MtpStep>>,
    _permit: OwnedSemaphorePermit,
    _completion: Option<CompletionGuard>,
}
pub(super) fn kv_capacity(context: usize, end_pos: usize) -> usize {
    if context <= 512 {
        context
    } else {
        let active = end_pos.max(1);
        let paged = active.div_ceil(256) * 256;
        context.min(1024).min(paged)
    }
}
pub(super) fn bytes_for(context: usize, end_pos: usize, count: usize) -> usize {
    kv_capacity(context, end_pos) * 4096 + count * 8192
}
#[cfg(test)]
fn bytes(count: usize) -> usize {
    // Fixed-cap 512 default charge for 512-context compatibility.
    bytes_for(512, count, count)
}
impl MtpWork {
    pub(super) fn finish(self, result: Result<MtpStep>) {
        let Self { reply, _permit, .. } = self;
        drop(_permit);
        let _ = reply.send(result);
    }
    pub(super) fn required(&self) -> u64 {
        let end_pos = self.previous.position() + self.tokens.len();
        bytes_for(self.context_cap, end_pos, self.tokens.len()) as u64
    }
}
impl QwenDeviceExecutor {
    pub async fn open_mtp(device: Device, slots: usize, logical_byte_limit: u64) -> Result<Self> {
        let compiled_decode = false;
        Self::open_mtp_configured(
            device,
            slots,
            logical_byte_limit,
            ExecutionTuning {
                layer_group: 40,
                fused_recurrent: device == Device::Gpu,
                execution_timing: false,
                compiled_decode,
                operator_profile: false,
                fused_attention: false,
                batched_reference_attention: false,
                expert_kernel: 0,
                validation_mode: 2,
                activation_precision: 1,
            },
        )
        .await
    }
    pub async fn open_mtp_configured(
        device: Device,
        slots: usize,
        logical_byte_limit: u64,
        tuning: ExecutionTuning,
    ) -> Result<Self> {
        ensure!(
            (1..=64).contains(&slots)
                && logical_byte_limit > 0
                && logical_byte_limit <= 8 * 1024 * 1024 * 1024,
            "MTP resident budget bounds"
        );
        Self::open_mtp_with_context(device, slots, logical_byte_limit, 512, tuning).await
    }
    /// Opens an MTP-enabled device executor with configurable context up to 256K (`262,144` tokens).
    pub async fn open_mtp_with_context(
        device: Device,
        slots: usize,
        logical_byte_limit: u64,
        context_tokens: usize,
        tuning: ExecutionTuning,
    ) -> Result<Self> {
        ensure!(
            tuning.activation_precision == 1,
            "MTP requires the FP32 profile"
        );
        ensure!(
            (0..=2).contains(&tuning.expert_kernel) && (0..=2).contains(&tuning.validation_mode),
            "invalid MTP execution tuning"
        );
        ensure!(
            device == Device::Gpu,
            "native MTP CPU profile is not qualified"
        );
        ensure!(
            !tuning.operator_profile,
            "operator profiling requires target scalar eager execution"
        );
        ensure!(
            !tuning.compiled_decode || std::env::var_os("MLX_DISABLE_COMPILE").is_none(),
            "compiled decode requires no MLX_DISABLE_COMPILE"
        );
        ensure!(
            matches!(tuning.layer_group, 1 | 4 | 10 | 40),
            "layer group must be 1|4|10|40"
        );
        ensure!(
            (512..=262_144).contains(&context_tokens) && context_tokens.is_multiple_of(256),
            "MTP context bounds (512..=262144, 256-token page aligned)"
        );
        ensure!(
            (1..=64).contains(&slots)
                && logical_byte_limit > 0
                && logical_byte_limit <= 32 * 1024 * 1024 * 1024,
            "MTP resident budget bounds"
        );
        tokio::task::spawn_blocking(move || {
            Self::start_with_context(
                device,
                slots,
                logical_byte_limit,
                context_tokens as i32,
                true,
                tuning,
            )
        })
        .await?
    }
    pub async fn mtp_snapshot(&self, state: &MtpState) -> Result<Vec<f32>> {
        ensure!(
            !self.closed.load(Ordering::Acquire) && self.mtp_enabled && state.owner == self.owner,
            "foreign or disabled MTP snapshot"
        );
        let (reply, receive) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::MtpSnapshot(state.clone(), reply))
            .map_err(|_| anyhow::anyhow!("MTP snapshot queue full/closed"))?;
        receive.await.context("MTP snapshot worker stopped")?
    }
    pub fn empty_mtp_state(&self) -> Result<MtpState> {
        ensure!(self.mtp_enabled, "native MTP is not enabled");
        Ok(MtpState {
            owner: self.owner,
            token: None,
            rows: 0,
        })
    }
    /// Shifted token i is paired with target hidden row `row+i`.
    pub async fn mtp_prefill(
        &self,
        previous: &MtpState,
        hidden: &DeviceState,
        row: usize,
        tokens: Vec<u32>,
    ) -> Result<MtpStep> {
        self.mtp_prefill_guarded(previous, hidden, row, tokens, None)
            .await
    }
    pub(in crate::serving::native_qwen) async fn mtp_prefill_guarded(
        &self,
        previous: &MtpState,
        hidden: &DeviceState,
        row: usize,
        tokens: Vec<u32>,
        completion: Option<CompletionGuard>,
    ) -> Result<MtpStep> {
        self.validate(hidden)?;
        ensure!(
            row <= hidden.hidden_rows
                && tokens.len() <= hidden.hidden_rows - row
                && hidden.position() - hidden.hidden_rows + row == previous.position(),
            "MTP target hidden alignment"
        );
        self.mtp_execute(
            previous,
            Source::Target(hidden.clone()),
            row,
            tokens,
            completion,
            false,
        )
        .await
    }
    #[cfg(feature = "debug-validation")]
    pub async fn mtp_prefill_debug(
        &self,
        previous: &MtpState,
        hidden: &DeviceState,
        row: usize,
        tokens: Vec<u32>,
    ) -> Result<MtpStep> {
        self.validate(hidden)?;
        ensure!(
            row <= hidden.hidden_rows
                && tokens.len() <= hidden.hidden_rows - row
                && hidden.position() - hidden.hidden_rows + row == previous.position(),
            "MTP target hidden alignment"
        );
        self.mtp_execute(
            previous,
            Source::Target(hidden.clone()),
            row,
            tokens,
            None,
            true,
        )
        .await
    }
    pub async fn mtp_next(&self, previous: &MtpState, token: u32) -> Result<MtpStep> {
        self.mtp_next_guarded(previous, token, None).await
    }
    pub(in crate::serving::native_qwen) async fn mtp_next_guarded(
        &self,
        previous: &MtpState,
        token: u32,
        completion: Option<CompletionGuard>,
    ) -> Result<MtpStep> {
        ensure!(previous.rows > 0, "MTP autoregressive source empty");
        self.mtp_execute(
            previous,
            Source::Draft(previous.clone()),
            previous.rows - 1,
            vec![token],
            completion,
            false,
        )
        .await
    }
    async fn mtp_execute(
        &self,
        previous: &MtpState,
        source: Source,
        row: usize,
        tokens: Vec<u32>,
        completion: Option<CompletionGuard>,
        diagnostics: bool,
    ) -> Result<MtpStep> {
        let context_cap = self.architecture.context as usize;
        ensure!(
            self.mtp_enabled
                && !self.closed.load(Ordering::Acquire)
                && previous.owner == self.owner,
            "MTP disabled, closed or foreign state"
        );
        ensure!(
            (1..=8).contains(&tokens.len())
                && previous.position() + tokens.len() <= context_cap
                && tokens.iter().all(|t| *t < self.architecture.vocab as u32),
            "MTP token/context bounds"
        );
        let permit = self
            .permit
            .clone()
            .try_acquire_owned()
            .context("MTP in-flight limit reached")?;
        let (reply, receive) = oneshot::channel();
        self.tx
            .as_ref()
            .context("device stopped")?
            .try_send(Work::Mtp(MtpWork {
                previous: previous.clone(),
                source,
                row,
                tokens,
                context_cap,
                diagnostics,
                reply,
                _permit: permit,
                _completion: completion,
            }))
            .map_err(|_| anyhow::anyhow!("MTP queue full/closed"))?;
        receive.await.context("MTP worker stopped")?
    }
}
impl Session {
    pub(super) fn release_mtp(&self, id: &StateId) -> Result<()> {
        let mut e = [0; 1024];
        native_status(
            unsafe { rs_qwen_device_mtp_release(self.0, id, e.as_mut_ptr(), e.len()) },
            &e,
        )
    }
}
pub(super) fn forward(
    session: &Session,
    manifest: &Manifest,
    owner: u64,
    releases: &mpsc::UnboundedSender<StateId>,
    work: &MtpWork,
) -> Result<MtpStep> {
    manifest.verify_stamps()?;
    let (source, kind) = match &work.source {
        Source::Target(s) => (s.token.as_ref().context("missing target hidden")?.id, 0),
        Source::Draft(s) => (s.token.as_ref().context("missing draft hidden")?.id, 1),
    };
    let mut id = StateId::default();
    let mut logits = vec![0.; 248320];
    let mut e = [0; 1024];
    native_status(
        unsafe {
            rs_qwen_device_mtp_step(
                session.0,
                work.previous
                    .token
                    .as_ref()
                    .map_or(std::ptr::null(), |s| &s.id),
                &source,
                kind,
                work.row,
                work.tokens.as_ptr(),
                work.tokens.len(),
                &mut id,
                logits.as_mut_ptr(),
                logits.len(),
                i32::from(work.diagnostics),
                e.as_mut_ptr(),
                e.len(),
            )
        },
        &e,
    )?;
    let state = MtpState {
        owner,
        rows: work.tokens.len(),
        token: Some(Arc::new(MtpLifetime {
            id,
            releases: releases.clone(),
        })),
    };
    manifest.verify_stamps()?;
    ensure!(
        id.position == (work.previous.position() + work.tokens.len()) as u64
            && id.bytes == work.required()
            && logits.iter().all(|x| x.is_finite()),
        "MTP receipt geometry/finite"
    );
    Ok(MtpStep { state, logits })
}
pub(super) fn forward_fused(
    session: &Session,
    manifest: &Manifest,
    owner: u64,
    target_releases: &mpsc::UnboundedSender<StateId>,
    mtp_releases: &mpsc::UnboundedSender<StateId>,
    work: &MtpFusedWork,
) -> Result<FusedMtpReceipt> {
    manifest.verify_stamps()?;
    let target_prev = &work
        .target_previous
        .token
        .as_ref()
        .context("missing target parent state")?
        .id;
    let draft_prev = &work
        .draft_previous
        .token
        .as_ref()
        .context("missing draft parent state")?
        .id;
    let mut raw = RawMtpTxResult::default();
    let mut target_logits = vec![0f32; 248_320];
    let mut draft_logits = vec![0f32; 248_320];
    let mut e = [0; 1024];
    let status = unsafe {
        rs_qwen_device_mtp_transaction_prepared(
            session.0,
            target_prev,
            draft_prev,
            work.seed_token,
            work.first_draft_token,
            work.first_draft_margin,
            work.width,
            work.output_budget,
            if work.eos.is_empty() {
                std::ptr::null()
            } else {
                work.eos.as_ptr()
            },
            work.eos.len(),
            i32::from(work.reuse),
            &mut raw,
            target_logits.as_mut_ptr(),
            target_logits.len(),
            draft_logits.as_mut_ptr(),
            draft_logits.len(),
            e.as_mut_ptr(),
            e.len(),
            work.prepared
                .as_ref()
                .map_or(std::ptr::null(), |p| p.as_ptr()),
            work.prepared.as_ref().map_or(0, Vec::len),
        )
    };
    if let Some(calls) = &work.calls {
        calls.fetch_add(u64::from(raw.target_calls), Ordering::Relaxed);
    }
    native_status(status, &e)?;
    let emitted_count = raw.emitted_count as usize;
    let target_state = DeviceState {
        owner,
        hidden_rows: emitted_count,
        token: Some(Arc::new(Lifetime {
            id: raw.target_state,
            releases: target_releases.clone(),
        })),
    };
    let draft_state = MtpState {
        owner,
        rows: emitted_count,
        token: Some(Arc::new(MtpLifetime {
            id: raw.draft_state,
            releases: mtp_releases.clone(),
        })),
    };
    manifest.verify_stamps()?;
    ensure!(
        emitted_count > 0
            && emitted_count <= 9
            && raw.target_state.position == raw.draft_state.position
            && target_logits
                .iter()
                .chain(&draft_logits)
                .all(|x| x.is_finite()),
        "MTP fused receipt geometry/finite"
    );
    Ok(FusedMtpReceipt {
        target_state,
        target_logits,
        draft_step: MtpStep {
            state: draft_state,
            logits: draft_logits,
        },
        emitted: raw.emitted[..emitted_count].to_vec(),
        learned_proposed: raw.learned_proposed as usize,
        learned_accepted: raw.learned_accepted as usize,
        target_calls: raw.target_calls as usize,
        draft_calls: raw.draft_calls as usize,
        reused_target_tokens: raw.reused_target_tokens as usize,
        replayed_target_tokens: raw.replayed_target_tokens as usize,
        target_forward_tokens: raw.target_forward_tokens as usize,
        terminal: raw.terminal != 0,
    })
}
unsafe extern "C" {
    pub(super) fn rs_qwen_device_mtp_attach(
        session: *mut c_void,
        weights: *mut c_void,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_qwen_device_mtp_step(
        session: *mut c_void,
        previous: *const StateId,
        source: *const StateId,
        draft_source: i32,
        row: usize,
        tokens: *const u32,
        count: usize,
        output: *mut StateId,
        logits: *mut f32,
        logits_count: usize,
        diagnostics: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_qwen_device_mtp_release(
        session: *mut c_void,
        id: *const StateId,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn rs_qwen_device_mtp_transaction_prepared(
        session: *mut c_void,
        target_previous: *const StateId,
        draft_previous: *const StateId,
        seed_token: u32,
        first_draft_token: u32,
        first_draft_margin: f32,
        width: usize,
        output_budget: usize,
        eos: *const u32,
        eos_count: usize,
        reuse: i32,
        result: *mut RawMtpTxResult,
        target_logits: *mut f32,
        target_logits_count: usize,
        draft_logits: *mut f32,
        draft_logits_count: usize,
        error: *mut c_char,
        capacity: usize,
        prepared_tokens: *const u32,
        prepared_count: usize,
    ) -> i32;
}

pub(super) fn snapshot(session: &Session, state: &MtpState) -> Result<Vec<f32>> {
    let Some(token) = &state.token else {
        return Ok(Vec::new());
    };
    let mut values = vec![0.; state.bytes() / 4];
    let mut e = [0; 1024];
    native_status(
        unsafe {
            rs_qwen_device_mtp_snapshot(
                session.0,
                &token.id,
                values.as_mut_ptr(),
                values.len(),
                e.as_mut_ptr(),
                e.len(),
            )
        },
        &e,
    )?;
    ensure!(values.iter().all(|x| x.is_finite()), "MTP snapshot finite");
    Ok(values)
}
unsafe extern "C" {
    fn rs_qwen_device_mtp_snapshot(
        session: *mut c_void,
        id: *const StateId,
        output: *mut f32,
        count: usize,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn draft_work_owns_parent_guard_and_permit_after_receiver_cancellation() {
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let (retire, mut retired) = mpsc::unbounded_channel();
        let id = StateId {
            owner: 2,
            slot: 0,
            generation: 1,
            position: 1,
            bytes: 12288,
        };
        let parent = MtpState {
            owner: 7,
            rows: 1,
            token: Some(Arc::new(MtpLifetime {
                id,
                releases: retire,
            })),
        };
        let limiter = Arc::new(Semaphore::new(1));
        let permit = limiter.clone().acquire_owned().await.unwrap();
        let released = Arc::new(AtomicBool::new(false));
        let guard: CompletionGuard = Arc::new(Guard(released.clone()));
        let (reply, received) = oneshot::channel();
        let work = MtpWork {
            previous: parent.clone(),
            source: Source::Draft(parent.clone()),
            row: 0,
            tokens: vec![1],
            context_cap: 512,
            diagnostics: false,
            reply,
            _permit: permit,
            _completion: Some(guard),
        };
        drop(received);
        drop(parent);
        assert!(retired.try_recv().is_err());
        assert!(!released.load(Ordering::SeqCst));
        assert_eq!(limiter.available_permits(), 0);
        drop(work);
        assert_eq!(retired.try_recv().unwrap(), id);
        assert!(retired.try_recv().is_err());
        assert!(released.load(Ordering::SeqCst));
        assert_eq!(limiter.available_permits(), 1);
    }
    #[tokio::test]
    async fn unqualified_cpu_profile_fails_before_checkpoint_loading() {
        assert!(
            QwenDeviceExecutor::open_mtp(Device::Cpu, 8, 768 * 1024 * 1024)
                .await
                .is_err()
        );
    }
    #[test]
    fn fixed_cap_mtp_charge_matches_native_geometry() {
        for count in 1..=8 {
            assert_eq!(bytes(count), 512 * 2 * 2 * 256 * 4 + count * 2048 * 4);
        }
    }
    #[test]
    fn sliding_256k_mtp_charge_caps_at_1024_token_window() {
        assert_eq!(kv_capacity(262_144, 10), 256);
        assert_eq!(kv_capacity(262_144, 600), 768);
        assert_eq!(kv_capacity(262_144, 262_144), 1024);
    }
    #[test]
    fn draft_retirement_survives_worker_shutdown() {
        let (retire, receive) = mpsc::unbounded_channel();
        drop(receive);
        for generation in 1..=32_000 {
            let state = MtpState {
                owner: 7,
                rows: 1,
                token: Some(Arc::new(MtpLifetime {
                    id: StateId {
                        owner: 2,
                        slot: 0,
                        generation,
                        position: 1,
                        bytes: 12288,
                    },
                    releases: retire.clone(),
                })),
            };
            let reader = state.clone();
            drop(state);
            assert_eq!(reader.position(), 1);
            assert_eq!(reader.bytes(), 12288);
            drop(reader);
        }
    }
}
