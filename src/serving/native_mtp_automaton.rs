// Stateful native MTP workload under the existing Spec 15 controller.
#[derive(Clone)]
pub struct AutomatonMtpCursor {
    inner: super::MtpCursor,
    committed: Arc<Vec<u32>>,
    identity: Identity,
}
impl AutomatonMtpCursor {
    pub fn target(&self) -> &GreedyCursor { self.inner.target() }
    pub fn committed(&self) -> &[u32] { &self.committed }
}
pub struct AutomatonMtpStep {
    pub cursor: AutomatonMtpCursor,
    pub emitted: Vec<u32>,
    pub learned_proposed: usize,
    pub learned_accepted: usize,
    pub target_calls: usize,
    pub draft_calls: usize,
    pub admitted: bool,
    pub draft_key: Option<BlockKey>,
}
pub struct AutomatonMtp {
    controller: Controller,
    executor: Arc<QwenDeviceExecutor>,
    identity: Identity,
    id: ProposerId,
}
impl AutomatonMtp {
    pub fn register(controller: Controller, executor: Arc<QwenDeviceExecutor>, namespace: String) -> Result<Self> {
        let identity = Identity { model: executor.identity().into(), tokenizer: executor.identity().into(), quantization: "fp32-explicit-attention-regular-kv-mtp-maintenance-v1".into(), namespace };
        let id = controller.register_proposer("combined-mtp-mixed-v1".into(), identity.clone())?;
        Ok(Self { controller, executor, identity, id })
    }
    pub fn id(&self) -> ProposerId { self.id }
    pub async fn start(&self, prompt: Vec<u32>) -> Result<AutomatonMtpCursor> {
        prefix_key(&self.identity, &prompt)?;
        Ok(AutomatonMtpCursor { inner: self.executor.mtp_start(&prompt).await?, committed: Arc::new(prompt), identity: self.identity.clone() })
    }
    fn validate(&self, cursor: &AutomatonMtpCursor) -> Result<()> {
        ensure!(cursor.identity == self.identity && cursor.committed.len() == cursor.target().state().position() && !cursor.target().terminal(), "foreign/terminal MTP automaton cursor");
        Ok(())
    }
    fn output(cursor: &AutomatonMtpCursor, tx: super::MtpTransaction, admitted: bool, draft_key: Option<BlockKey>) -> AutomatonMtpStep {
        let mut committed = (*cursor.committed).clone();
        committed.extend_from_slice(&tx.emitted);
        AutomatonMtpStep { cursor: AutomatonMtpCursor { inner: tx.cursor, committed: Arc::new(committed), identity: cursor.identity.clone() }, emitted: tx.emitted, learned_proposed: tx.learned_proposed, learned_accepted: tx.learned_accepted, target_calls: tx.target_calls, draft_calls: tx.draft_calls, admitted, draft_key }
    }
    /// One target token plus explicitly charged draft-state maintenance.
    pub async fn no_draft(&self, cursor: &AutomatonMtpCursor, eos: &[u32]) -> Result<AutomatonMtpStep> {
        self.validate(cursor)?;
        ensure!(eos.len() <= 16, "MTP EOS bounds");
        let start = Instant::now();
        let tx = self.executor.mtp_decode_bounded(&cursor.inner, 1, 1, eos).await?;
        self.controller.record_no_draft(self.id, u64::try_from(start.elapsed().as_nanos())?.max(1), tx.emitted.len() as u64)?;
        Ok(Self::output(cursor, tx, false, None))
    }
    pub async fn next(&self, cursor: &AutomatonMtpCursor, width: usize, output_budget: usize, eos: &[u32]) -> Result<AutomatonMtpStep> {
        self.validate(cursor)?;
        ensure!((1..=8).contains(&width) && output_budget > 0 && eos.len() <= 16, "MTP automaton bounds");
        let maximum = width.min(output_budget).min(512usize.saturating_sub(cursor.target().state().position()));
        if maximum <= 1 { return self.no_draft(cursor, eos).await; }
        let start = Instant::now();
        let parent = prefix_key(&self.identity, &cursor.committed)?;
        let permit = match self.controller.request_draft(self.id, parent, maximum, true)? {
            DraftAdmission::Ready(p) => p,
            DraftAdmission::Benched => return self.no_draft(cursor, eos).await,
            DraftAdmission::Queued(t) => { self.controller.cancel_draft(t)?; return self.no_draft(cursor, eos).await; }
        };
        let running = Arc::new(Running(Mutex::new(Some(permit))));
        let guard: CompletionGuard = running.clone();
        let executor = self.executor.clone();
        let cursor = cursor.clone();
        let eos = eos.to_vec();
        let (reply, receive) = oneshot::channel();
        let (done, finished) = oneshot::channel();
        tokio::spawn(async move {
            let calls = Arc::new(AtomicU64::new(0));
            let mut proposed = 0;
            let mut known_prefix = 0;
            let result = async {
                let prepared = executor.mtp_prepare(&cursor.inner, maximum, output_budget, &eos, Some(guard.clone())).await?;
                let key = running.bind(prepared.tokens())?;
                known_prefix = 1;
                proposed = prepared.tokens().len() - 1;
                let tx = executor.mtp_commit(prepared, true, Some(calls.clone())).await?;
                Ok::<_, anyhow::Error>(Self::output(&cursor, tx, true, Some(key)))
            }.await;
            if known_prefix == 0 { let _ = running.bind(&[]); }
            let mut outcome = DraftOutcome { known_prefix, proposed: proposed as u64, accepted: 0, committed: 0, target_calls: calls.load(Ordering::Relaxed), cost_ns: u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX).max(1), failed: result.is_err() };
            if let Ok(tx) = &result { outcome.accepted = tx.learned_accepted as u64; outcome.committed = tx.emitted.len() as u64; }
            if reply.send(result).is_err() { outcome.committed = 0; }
            let _ = done.send(running.finish(outcome));
        });
        let result = receive.await.context("MTP proposer task stopped")?;
        finished.await.context("MTP accounting stopped")??;
        result
    }
}
