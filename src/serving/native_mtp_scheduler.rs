//! Spec 15 MTP under the existing fair scheduler; verified output stays request-local.
use super::{AutomatonMtp, AutomatonMtpCursor, QwenDeviceExecutor};
use crate::serving::{model_ir::StepKind, scheduler::*};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

type DriverFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

trait MtpCursorState: Clone + Send + Sync + 'static {
    fn position(&self) -> usize;
    fn resident_bytes(&self) -> usize;
}

impl MtpCursorState for AutomatonMtpCursor {
    fn position(&self) -> usize {
        self.target().state().position()
    }
    fn resident_bytes(&self) -> usize {
        AutomatonMtpCursor::resident_bytes(self)
    }
}

struct DriverReceipt<C> {
    cursor: C,
    emitted: Vec<u32>,
}

trait MtpDriver: Send + Sync + 'static {
    type Cursor: MtpCursorState;
    fn seed(&self, cursor: &Self::Cursor) -> Result<u32>;
    fn start(&self, prompt: Vec<u32>) -> DriverFuture<'_, Self::Cursor>;
    fn no_draft<'a>(
        &'a self,
        cursor: &'a Self::Cursor,
        eos: &'a [u32],
    ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>>;
    fn next<'a>(
        &'a self,
        cursor: &'a Self::Cursor,
        width: usize,
        output_budget: usize,
        eos: &'a [u32],
    ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>>;
    fn verify_drained(&self) -> DriverFuture<'_, ()>;
}

struct NativeMtpDriver {
    executor: Arc<QwenDeviceExecutor>,
    lane: AutomatonMtp,
}

impl MtpDriver for NativeMtpDriver {
    type Cursor = AutomatonMtpCursor;

    fn seed(&self, cursor: &Self::Cursor) -> Result<u32> {
        cursor.target().next_greedy_token()
    }
    fn start(&self, prompt: Vec<u32>) -> DriverFuture<'_, Self::Cursor> {
        Box::pin(async move { self.lane.start(prompt).await })
    }

    fn no_draft<'a>(
        &'a self,
        cursor: &'a Self::Cursor,
        eos: &'a [u32],
    ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>> {
        Box::pin(async move {
            let tx = self.lane.no_draft(cursor, eos).await?;
            Ok(DriverReceipt {
                cursor: tx.cursor,
                emitted: tx.emitted,
            })
        })
    }

    fn next<'a>(
        &'a self,
        cursor: &'a Self::Cursor,
        width: usize,
        output_budget: usize,
        eos: &'a [u32],
    ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>> {
        Box::pin(async move {
            let tx = self.lane.next(cursor, width, output_budget, eos).await?;
            Ok(DriverReceipt {
                cursor: tx.cursor,
                emitted: tx.emitted,
            })
        })
    }

    fn verify_drained(&self) -> DriverFuture<'_, ()> {
        Box::pin(async move {
            let s = self.executor.stats().await?;
            ensure!(
                s.residents == 0 && s.logical_bytes == 0,
                "MTP scheduler native drain"
            );
            Ok(())
        })
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
enum SeedState {
    #[default]
    Ready,
    SeedPending(u32),
}
#[derive(Default)]
struct SeedCounts {
    receipts: AtomicU64,
    attempts: AtomicU64,
    completed: AtomicU64,
    pending_releases: AtomicU64,
}
/// Snapshot only after draining for a coherent final accounting cut.
#[derive(Clone, Default)]
pub struct SeedMetrics(Arc<SeedCounts>);
#[derive(Debug, serde::Serialize)]
pub struct SeedSnapshot {
    pub prefill_seed_receipts: u64,
    pub advance_attempts: u64,
    pub advance_completed: u64,
    pub pending_releases: u64,
}
impl SeedMetrics {
    pub fn snapshot(&self) -> SeedSnapshot {
        SeedSnapshot {
            prefill_seed_receipts: self.0.receipts.load(Ordering::SeqCst),
            advance_attempts: self.0.attempts.load(Ordering::SeqCst),
            advance_completed: self.0.completed.load(Ordering::SeqCst),
            pending_releases: self.0.pending_releases.load(Ordering::SeqCst),
        }
    }
}
struct Row<C> {
    seed: SeedState,
    prompt: Vec<u32>,
    cursor: Option<C>,
    pending: VecDeque<u32>,
    last: Option<u32>,
    consumed: usize,
}

impl<C: Clone> Clone for Row<C> {
    fn clone(&self) -> Self {
        Self {
            seed: self.seed,
            prompt: self.prompt.clone(),
            cursor: self.cursor.clone(),
            pending: self.pending.clone(),
            last: self.last,
            consumed: self.consumed,
        }
    }
}

impl<C> Default for Row<C> {
    fn default() -> Self {
        Self {
            seed: SeedState::Ready,
            prompt: Vec::new(),
            cursor: None,
            pending: VecDeque::new(),
            last: None,
            consumed: 0,
        }
    }
}

struct Core<D: MtpDriver> {
    started: bool,
    deferred_seed: bool,
    seed_metrics: SeedMetrics,
    driver: D,
    rows: BTreeMap<u64, Row<D::Cursor>>,
    width: usize,
    eos: Vec<u32>,
    identity: String,
}

impl<D: MtpDriver> Core<D> {
    fn validate_bounds(width: usize, eos: &[u32]) -> Result<()> {
        ensure!(
            (1..=8).contains(&width) && eos.len() <= 16 && eos.iter().all(|t| *t < 248320),
            "MTP scheduler width/EOS"
        );
        Ok(())
    }

    fn storage(&self) -> Storage {
        Storage::BackendManaged {
            fingerprint: Sha256::digest(self.identity.as_bytes()).into(),
            sequence_bytes: 128 * 1024 * 1024,
            experimental: true,
        }
    }

    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: self.identity.clone(),
            width: 0,
            vocabulary_size: 248320,
            max_context_tokens: 511,
            workspace_bytes: 512 * 1024 * 1024,
            max_in_flight: 3,
            eos_tokens: self.eos.clone(),
        }
    }

    async fn step(&mut self, batch: Vec<StepInput>) -> Result<Vec<StepOutput>> {
        ensure!(
            !batch.is_empty() && batch.len() <= 3,
            "MTP scheduler row bounds"
        );
        self.started = true;
        let mut seen = BTreeSet::new();
        let mut prepared = Vec::with_capacity(batch.len());
        let new = batch
            .iter()
            .filter(|i| !self.rows.contains_key(&i.sequence))
            .count();
        ensure!(self.rows.len() + new <= 3, "MTP scheduler lane cap");
        for item in batch {
            ensure!(
                seen.insert(item.sequence)
                    && item.history.is_empty()
                    && item.allowed_tokens.is_none(),
                "MTP duplicate/history/choices"
            );
            ensure!(
                (1..=128).contains(&item.remaining_output_tokens)
                    && (1..=8).contains(&item.tokens.len())
                    && item.tokens.iter().all(|t| *t < 248320),
                "MTP scheduler token/output geometry"
            );
            let row = self.rows.get(&item.sequence).cloned().unwrap_or_default();
            ensure!(
                item.position == row.consumed,
                "MTP scheduler virtual position"
            );
            if let SeedState::SeedPending(seed) = row.seed {
                ensure!(
                    self.deferred_seed
                        && row.pending.is_empty()
                        && row.last == Some(seed)
                        && row
                            .cursor
                            .as_ref()
                            .is_some_and(|c| c.position() == row.consumed),
                    "pending seed materialization invariant"
                );
            }
            match item.kind {
                StepKind::Prefill => {
                    ensure!(
                        row.cursor.is_none() && row.prompt.len() + item.tokens.len() <= 511,
                        "MTP prompt staging"
                    );
                    ensure!(
                        row.prompt.len() + item.tokens.len() + item.remaining_output_tokens <= 512,
                        "MTP consumed-output context"
                    );
                }
                StepKind::Decode => ensure!(
                    row.cursor.is_some()
                        && item.tokens.len() == 1
                        && row.last == item.tokens.first().copied()
                        && item.prompt_complete,
                    "MTP decoded history"
                ),
            }
            prepared.push((item, row));
        }
        let mut seed_receipts = 0;
        let mut output = Vec::with_capacity(prepared.len());
        let mut updates = Vec::with_capacity(prepared.len());
        for (item, mut row) in prepared {
            row.consumed += item.tokens.len();
            if item.kind == StepKind::Prefill {
                row.prompt.extend(item.tokens);
                if item.prompt_complete {
                    let cursor = self.driver.start(std::mem::take(&mut row.prompt)).await?;
                    if self.deferred_seed {
                        ensure!(
                            cursor.position() == row.consumed,
                            "prefill materialized position"
                        );
                        let seed = self.driver.seed(&cursor)?;
                        ensure!(seed < 248320, "prefill seed vocabulary");
                        row.pending.push_back(seed);
                        row.seed = SeedState::SeedPending(seed);
                        row.cursor = Some(cursor);
                        seed_receipts += 1;
                    } else {
                        let tx = self.driver.no_draft(&cursor, &self.eos).await?;
                        row.pending.extend(tx.emitted);
                        row.cursor = Some(tx.cursor);
                    }
                }
            } else if row.pending.is_empty() {
                if let SeedState::SeedPending(seed) = row.seed {
                    ensure!(
                        row.last == Some(seed) && !self.eos.contains(&seed),
                        "deferred seed input/EOS"
                    );
                    self.seed_metrics.0.attempts.fetch_add(1, Ordering::SeqCst);
                    let tx = self
                        .driver
                        .no_draft(
                            row.cursor.as_ref().context("deferred seed cursor")?,
                            &self.eos,
                        )
                        .await?;
                    ensure!(
                        tx.emitted == [seed] && tx.cursor.position() == row.consumed,
                        "deferred advance changed published seed/position"
                    );
                    self.seed_metrics.0.completed.fetch_add(1, Ordering::SeqCst);
                    row.cursor = Some(tx.cursor);
                    row.seed = SeedState::Ready;
                }
                let tx = self
                    .driver
                    .next(
                        row.cursor.as_ref().context("MTP cursor")?,
                        self.width,
                        item.remaining_output_tokens,
                        &self.eos,
                    )
                    .await?;
                row.pending.extend(tx.emitted);
                row.cursor = Some(tx.cursor);
            }
            let next_token = if item.prompt_complete {
                Some(
                    row.pending
                        .pop_front()
                        .context("empty MTP output receipt")?,
                )
            } else {
                None
            };
            if let Some(token) = next_token {
                row.last = Some(token);
            }
            ensure!(
                row.pending.len() < item.remaining_output_tokens,
                "MTP buffered output exceeded generation budget"
            );
            let bytes = row
                .cursor
                .as_ref()
                .map_or(0, MtpCursorState::resident_bytes);
            output.push(StepOutput {
                sequence: item.sequence,
                kv: Vec::new(),
                next_token,
                state_bytes: bytes,
            });
            updates.push((item.sequence, row));
        }
        self.seed_metrics
            .0
            .receipts
            .fetch_add(seed_receipts, Ordering::SeqCst);
        for (id, row) in updates {
            self.rows.insert(id, row);
        }
        Ok(output)
    }

    fn release(&mut self, sequence: u64) {
        if let Some(row) = self.rows.remove(&sequence) {
            if matches!(row.seed, SeedState::SeedPending(_)) {
                self.seed_metrics
                    .0
                    .pending_releases
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        for id in self.rows.keys().copied().collect::<Vec<_>>() {
            self.release(id);
        }
        self.driver.verify_drained().await
    }
}

pub struct ScheduledMtp(Core<NativeMtpDriver>, crate::cache::Controller, bool);

impl ScheduledMtp {
    /// Experimental opt-in; materialization of a published prefill seed is deferred.
    pub fn with_deferred_seed(mut self) -> Result<Self> {
        ensure!(
            !self.0.started && self.0.rows.is_empty() && !self.0.deferred_seed,
            "seed mode must be configured once before admission"
        );
        self.0.deferred_seed = true;
        self.0.identity.push_str("-seed-deferred-v1");
        Ok(self)
    }
    pub fn seed_metrics(&self) -> SeedMetrics {
        self.0.seed_metrics.clone()
    }
    /// Caller must exclusively own this draft-only metadata controller's audit clock.
    /// Payload-cache controllers are refused at every boundary.
    pub fn with_exclusive_metadata_audits(mut self) -> Self {
        self.2 = true;
        self
    }
}

impl ScheduledMtp {
    pub fn new(
        controller: crate::cache::Controller,
        executor: Arc<QwenDeviceExecutor>,
        eos: Vec<u32>,
        width: usize,
        namespace: String,
    ) -> Result<Self> {
        Core::<NativeMtpDriver>::validate_bounds(width, &eos)?;
        executor.empty_mtp_state()?;
        let identity = format!(
            "{}:experimental-spec15-mtp-width{width}-fp32-v1",
            executor.identity()
        );
        let lane = AutomatonMtp::register(controller.clone(), executor.clone(), namespace)?;
        Ok(Self(
            Core {
                started: false,
                deferred_seed: false,
                seed_metrics: SeedMetrics::default(),
                driver: NativeMtpDriver { executor, lane },
                rows: BTreeMap::new(),
                width,
                eos,
                identity,
            },
            controller,
            false,
        ))
    }

    pub fn proposer_id(&self) -> crate::cache::ProposerId {
        self.0.driver.lane.id()
    }

    pub fn storage(&self) -> Storage {
        self.0.storage()
    }
}

impl ModelBackend for ScheduledMtp {
    fn info(&self) -> BackendInfo {
        self.0.info()
    }

    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move { self.0.step(batch).await })
    }

    fn validated_boundary(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async move {
            if self.2 {
                let stats = self.1.stats()?;
                ensure!(
                    stats.open_branches == 0
                        && stats.resident_blocks == 0
                        && stats.reserved_blocks == 0,
                    "metadata audit cannot own payload cache plans"
                );
                let epoch = stats.epoch.checked_add(1).context("audit epoch overflow")?;
                let native = self.0.driver.executor.stats().await?;
                self.1.audit(
                    epoch,
                    &[],
                    native.logical_bytes,
                    crate::cache::Pressure::Normal,
                )?;
            }
            Ok(())
        })
    }

    fn release(&mut self, sequence: u64) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.0.release(sequence);
            Ok(())
        })
    }

    fn shutdown(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async move { self.0.shutdown().await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serving::config::Config;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Clone)]
    struct FakeCursor {
        history: Vec<u32>,
        bytes: usize,
        _lease: Arc<()>,
    }

    impl MtpCursorState for FakeCursor {
        fn position(&self) -> usize {
            self.history.len()
        }
        fn resident_bytes(&self) -> usize {
            self.bytes
        }
    }

    struct FakeDriver {
        starts: Arc<AtomicUsize>,
        no_drafts: Arc<AtomicUsize>,
        next_calls: Arc<AtomicUsize>,
        lease_root: Arc<()>,
        overshoot_budget: Arc<AtomicBool>,
        fail_advance: Arc<AtomicBool>,
        fail_advance_at: AtomicUsize,
        wrong_seed: Arc<AtomicBool>,
        wrong_position: Arc<AtomicBool>,
        advance_gate: Option<Arc<tokio::sync::Semaphore>>,
    }

    impl FakeDriver {
        fn new() -> Self {
            Self {
                starts: Arc::new(AtomicUsize::new(0)),
                no_drafts: Arc::new(AtomicUsize::new(0)),
                next_calls: Arc::new(AtomicUsize::new(0)),
                lease_root: Arc::new(()),
                overshoot_budget: Arc::new(AtomicBool::new(false)),
                fail_advance: Arc::new(AtomicBool::new(false)),
                fail_advance_at: AtomicUsize::new(usize::MAX),
                wrong_seed: Arc::new(AtomicBool::new(false)),
                wrong_position: Arc::new(AtomicBool::new(false)),
                advance_gate: None,
            }
        }

        fn cursor(&self, history: Vec<u32>) -> FakeCursor {
            let bytes = history.len() * 4096;
            FakeCursor {
                history,
                bytes,
                _lease: self.lease_root.clone(),
            }
        }
    }

    impl MtpDriver for FakeDriver {
        type Cursor = FakeCursor;

        fn seed(&self, cursor: &Self::Cursor) -> Result<u32> {
            Ok((cursor.history.iter().sum::<u32>() + 1) % 1000)
        }
        fn start(&self, prompt: Vec<u32>) -> DriverFuture<'_, Self::Cursor> {
            Box::pin(async move {
                ensure!(!prompt.is_empty() && prompt.len() < 512, "prompt cap");
                self.starts.fetch_add(1, Ordering::SeqCst);
                Ok(self.cursor(prompt))
            })
        }

        fn no_draft<'a>(
            &'a self,
            cursor: &'a Self::Cursor,
            _eos: &'a [u32],
        ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>> {
            Box::pin(async move {
                let call = self.no_drafts.fetch_add(1, Ordering::SeqCst) + 1;
                if let Some(gate) = &self.advance_gate {
                    gate.acquire().await?.forget();
                }
                ensure!(
                    !self.fail_advance.load(Ordering::SeqCst)
                        && call != self.fail_advance_at.load(Ordering::SeqCst),
                    "injected advance failure"
                );
                let mut h = cursor.history.clone();
                let tok = (h.iter().sum::<u32>() + 1) % 1000;
                if !self.wrong_position.load(Ordering::SeqCst) {
                    h.push(tok);
                }
                Ok(DriverReceipt {
                    cursor: self.cursor(h),
                    emitted: vec![if self.wrong_seed.load(Ordering::SeqCst) {
                        tok + 1
                    } else {
                        tok
                    }],
                })
            })
        }

        fn next<'a>(
            &'a self,
            cursor: &'a Self::Cursor,
            width: usize,
            output_budget: usize,
            eos: &'a [u32],
        ) -> DriverFuture<'a, DriverReceipt<Self::Cursor>> {
            Box::pin(async move {
                self.next_calls.fetch_add(1, Ordering::SeqCst);
                let count = if self.overshoot_budget.load(Ordering::SeqCst) {
                    output_budget + 1
                } else {
                    width.min(output_budget).max(1)
                };
                let mut h = cursor.history.clone();
                let mut emitted = Vec::with_capacity(count);
                for _ in 0..count {
                    let tok = (h.iter().sum::<u32>() + 1) % 1000;
                    h.push(tok);
                    emitted.push(tok);
                    if eos.contains(&tok) {
                        break;
                    }
                }
                Ok(DriverReceipt {
                    cursor: self.cursor(h),
                    emitted,
                })
            })
        }

        fn verify_drained(&self) -> DriverFuture<'_, ()> {
            Box::pin(async move {
                ensure!(
                    Arc::strong_count(&self.lease_root) == 1,
                    "fake MTP cursors leaked"
                );
                Ok(())
            })
        }
    }

    impl ModelBackend for Core<FakeDriver> {
        fn info(&self) -> BackendInfo {
            Core::info(self)
        }
        fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
            Box::pin(async move { Core::step(self, batch).await })
        }
        fn release(&mut self, sequence: u64) -> ReleaseFuture<'_> {
            Box::pin(async move {
                Core::release(self, sequence);
                Ok(())
            })
        }
        fn shutdown(&mut self) -> ReleaseFuture<'_> {
            Box::pin(async move { Core::shutdown(self).await })
        }
    }

    fn fake_core(width: usize) -> Core<FakeDriver> {
        Core {
            started: false,
            deferred_seed: false,
            seed_metrics: SeedMetrics::default(),
            driver: FakeDriver::new(),
            rows: BTreeMap::new(),
            width,
            eos: vec![999],
            identity: format!("FAKE-MTP-width{width}"),
        }
    }

    fn input(kind: StepKind, position: usize, tokens: Vec<u32>, remaining: usize) -> StepInput {
        StepInput {
            sequence: 1,
            kind,
            position,
            tokens,
            remaining_output_tokens: remaining,
            prompt_complete: true,
            history: vec![],
            allowed_tokens: None,
        }
    }
    #[tokio::test]
    async fn deferred_seed_matches_eager_without_duplicate_output_or_maintenance() -> Result<()> {
        for cap in [1, 2, 4, 8, 64] {
            let mut eager = fake_core(4);
            let mut deferred = fake_core(4);
            deferred.deferred_seed = true;
            let mut outputs = [Vec::new(), Vec::new()];
            for (index, core) in [&mut eager, &mut deferred].into_iter().enumerate() {
                let first = core
                    .step(vec![input(StepKind::Prefill, 0, vec![1, 2, 3], cap)])
                    .await?[0]
                    .next_token
                    .unwrap();
                outputs[index].push(first);
                assert_eq!(
                    core.driver.no_drafts.load(Ordering::SeqCst),
                    if index == 0 { 1 } else { 0 }
                );
                for n in 1..cap {
                    let last = *outputs[index].last().unwrap();
                    let next = core
                        .step(vec![input(
                            StepKind::Decode,
                            3 + n - 1,
                            vec![last],
                            cap - n,
                        )])
                        .await?[0]
                        .next_token
                        .unwrap();
                    outputs[index].push(next);
                }
            }
            assert_eq!(outputs[0], outputs[1]);
            if cap > 1 {
                assert_eq!(
                    eager.rows[&1].cursor.as_ref().unwrap().history,
                    deferred.rows[&1].cursor.as_ref().unwrap().history
                );
            }
            let stats = deferred.seed_metrics.snapshot();
            assert_eq!(stats.prefill_seed_receipts, 1);
            assert_eq!(stats.advance_completed, u64::from(cap > 1));
            assert_eq!(stats.advance_attempts, stats.advance_completed);
            eager.shutdown().await?;
            deferred.shutdown().await?;
            assert_eq!(
                deferred.seed_metrics.snapshot().pending_releases,
                u64::from(cap == 1)
            );
        }
        Ok(())
    }
    #[tokio::test]
    async fn deferred_seed_eos_and_failed_advancement_preserve_old_snapshot() -> Result<()> {
        let mut eos = fake_core(4);
        eos.deferred_seed = true;
        eos.eos = vec![7];
        assert_eq!(
            eos.step(vec![input(StepKind::Prefill, 0, vec![1, 2, 3], 8)])
                .await?[0]
                .next_token,
            Some(7)
        );
        assert!(eos
            .step(vec![input(StepKind::Decode, 3, vec![7], 7)])
            .await
            .is_err());
        eos.shutdown().await?;
        assert_eq!(eos.seed_metrics.snapshot().advance_attempts, 0);
        assert_eq!(eos.seed_metrics.snapshot().pending_releases, 1);
        for wrong in 0..3 {
            let mut core = fake_core(4);
            core.deferred_seed = true;
            core.step(vec![input(StepKind::Prefill, 0, vec![1, 2, 3], 8)])
                .await?;
            if wrong == 1 {
                core.driver.wrong_seed.store(true, Ordering::SeqCst)
            } else if wrong == 0 {
                core.driver.fail_advance.store(true, Ordering::SeqCst)
            } else {
                core.driver.wrong_position.store(true, Ordering::SeqCst)
            };
            assert!(core
                .step(vec![input(StepKind::Decode, 3, vec![7], 7)])
                .await
                .is_err());
            assert_eq!(core.rows[&1].consumed, 3);
            assert_eq!(core.rows[&1].seed, SeedState::SeedPending(7));
            assert_eq!(core.driver.next_calls.load(Ordering::SeqCst), 0);
            assert_eq!(core.seed_metrics.snapshot().advance_completed, 0);
            core.shutdown().await?;
        }
        Ok(())
    }
    #[tokio::test]
    async fn later_row_failure_keeps_all_old_pending_seed_snapshots() -> Result<()> {
        let mut core = fake_core(4);
        core.deferred_seed = true;
        let first = input(StepKind::Prefill, 0, vec![1, 2, 3], 8);
        let mut second = input(StepKind::Prefill, 0, vec![1, 2, 3], 8);
        second.sequence = 2;
        core.step(vec![first, second]).await?;
        core.driver.fail_advance_at.store(2, Ordering::SeqCst);
        let first = input(StepKind::Decode, 3, vec![7], 7);
        let mut second = input(StepKind::Decode, 3, vec![7], 7);
        second.sequence = 2;
        assert!(core.step(vec![first, second]).await.is_err());
        for row in core.rows.values() {
            assert_eq!(row.consumed, 3);
            assert_eq!(row.seed, SeedState::SeedPending(7));
            assert_eq!(row.cursor.as_ref().unwrap().history, vec![1, 2, 3]);
        }
        let metrics = core.seed_metrics.clone();
        assert_eq!(metrics.snapshot().advance_completed, 1);
        assert_eq!(metrics.snapshot().advance_attempts, 2);
        core.shutdown().await?;
        assert_eq!(metrics.snapshot().pending_releases, 2);
        Ok(())
    }

    #[tokio::test]
    async fn canceled_deferred_advance_remains_owned_until_completion() -> Result<()> {
        let mut core = fake_core(4);
        core.deferred_seed = true;
        let gate = Arc::new(tokio::sync::Semaphore::new(0));
        core.driver.advance_gate = Some(gate.clone());
        let calls = core.driver.no_drafts.clone();
        let metrics = core.seed_metrics.clone();
        let cfg = Config {
            version: 1,
            max_queued: 3,
            max_sequences: 3,
            max_batch_tokens: 24,
            prefill_chunk_tokens: 8,
            max_context_tokens: 511,
            max_new_tokens: 128,
            model_memory_bytes: 1024 * 1024 * 1024,
            max_in_flight: 3,
        };
        let engine = Engine::new(cfg, core.storage(), core)?;
        let mut request = engine.submit_stream(vec![1, 2, 3], 8, 1)?;
        assert_eq!(request.next_token().await.context("first seed")?.token, 7);
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        drop(request);
        assert_eq!(metrics.snapshot().advance_completed, 0);
        gate.add_permits(1);
        let drained = engine.shutdown().await?;
        assert_eq!(drained.cancelled, 1);
        assert_eq!(drained.memory_used, 0);
        assert_eq!(metrics.snapshot().advance_completed, 1);
        Ok(())
    }

    #[tokio::test]
    async fn buffered_speculative_tokens_drain_without_extra_driver_calls() -> Result<()> {
        let mut core = fake_core(4);
        let next_calls = core.driver.next_calls.clone();
        // Chunk 1 of prefill (not complete)
        let out1 = core
            .step(vec![StepInput {
                remaining_output_tokens: 5,
                kind: StepKind::Prefill,
                sequence: 1,
                position: 0,
                tokens: vec![2, 3],
                prompt_complete: false,
                history: vec![],
                allowed_tokens: None,
            }])
            .await?;
        assert_eq!(out1[0].next_token, None);
        assert_eq!(out1[0].state_bytes, 0);

        // Chunk 2 of prefill (complete -> runs start + no_draft -> emits 1st token)
        let out2 = core
            .step(vec![StepInput {
                remaining_output_tokens: 5,
                kind: StepKind::Prefill,
                sequence: 1,
                position: 2,
                tokens: vec![4],
                prompt_complete: true,
                history: vec![],
                allowed_tokens: None,
            }])
            .await?;
        let t0 = out2[0].next_token.unwrap();
        assert_eq!(t0, 10); // (2+3+4+1) = 10
        assert_eq!(next_calls.load(Ordering::SeqCst), 0);

        // Decode step 1: remaining_output_tokens = 4, width = 4 -> driver proposes 4 tokens,
        // 1 is popped now and 3 remain buffered in row.pending.
        let d1 = core
            .step(vec![StepInput {
                remaining_output_tokens: 4,
                kind: StepKind::Decode,
                sequence: 1,
                position: 3,
                tokens: vec![t0],
                prompt_complete: true,
                history: vec![],
                allowed_tokens: None,
            }])
            .await?;
        let t1 = d1[0].next_token.unwrap();
        assert_eq!(next_calls.load(Ordering::SeqCst), 1);

        // Decode steps 2, 3, 4 drain the buffered tokens with ZERO additional driver.next calls!
        let mut prev = t1;
        for step_idx in 0..3 {
            let d = core
                .step(vec![StepInput {
                    remaining_output_tokens: 3 - step_idx,
                    kind: StepKind::Decode,
                    sequence: 1,
                    position: 4 + step_idx,
                    tokens: vec![prev],
                    prompt_complete: true,
                    history: vec![],
                    allowed_tokens: None,
                }])
                .await?;
            prev = d[0].next_token.unwrap();
            assert_eq!(next_calls.load(Ordering::SeqCst), 1);
        }
        core.release(1);
        core.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn row_bounds_atomic_validation_and_budget_overshoot_fail_closed() -> Result<()> {
        assert!(Core::<FakeDriver>::validate_bounds(0, &[999]).is_err());
        assert!(Core::<FakeDriver>::validate_bounds(9, &[999]).is_err());
        assert!(Core::<FakeDriver>::validate_bounds(4, &[248320]).is_err());

        let mut core = fake_core(4);
        let mk = |seq, pos, tokens: Vec<u32>, rem| StepInput {
            remaining_output_tokens: rem,
            kind: StepKind::Prefill,
            sequence: seq,
            position: pos,
            tokens,
            prompt_complete: true,
            history: vec![],
            allowed_tokens: None,
        };
        assert!(core.step(vec![]).await.is_err());
        // Duplicate sequence in batch
        assert!(core
            .step(vec![mk(1, 0, vec![1], 4), mk(1, 0, vec![2], 4)])
            .await
            .is_err());
        // Choice mask rejected
        let mut with_choice = mk(1, 0, vec![1], 4);
        with_choice.allowed_tokens = Some(vec![1]);
        assert!(core.step(vec![with_choice]).await.is_err());
        // Early prefill chunk exceeding 512 consumed+output context rejected immediately
        let mut bad_prefill = mk(1, 0, vec![1; 8], 128);
        bad_prefill.prompt_complete = false;
        for i in 0..48 {
            let step = StepInput {
                remaining_output_tokens: 128,
                kind: StepKind::Prefill,
                sequence: 99,
                position: i * 8,
                tokens: vec![1; 8],
                prompt_complete: false,
                history: vec![],
                allowed_tokens: None,
            };
            if (i + 1) * 8 + 128 <= 512 {
                core.step(vec![step]).await?;
            } else {
                assert!(core.step(vec![step]).await.is_err());
                break;
            }
        }
        core.release(99);

        // Atomic batch rejection: if row 2 has an out-of-vocab token, row 1 is not committed
        assert!(core
            .step(vec![mk(1, 0, vec![1, 2], 4), mk(2, 0, vec![248320], 4)])
            .await
            .is_err());
        assert!(!core.rows.contains_key(&1));

        // Overshoot of remaining_output_tokens fails closed before committing
        core.step(vec![mk(1, 0, vec![2, 3], 4)]).await?;
        let last = core.rows[&1].last.unwrap();
        core.driver.overshoot_budget.store(true, Ordering::SeqCst);
        assert!(core
            .step(vec![StepInput {
                remaining_output_tokens: 2,
                kind: StepKind::Decode,
                sequence: 1,
                position: 2,
                tokens: vec![last],
                prompt_complete: true,
                history: vec![],
                allowed_tokens: None,
            }])
            .await
            .is_err());
        core.shutdown().await?;
        Ok(())
    }

    #[tokio::test]
    async fn engine_submit_and_stream_match_across_concurrent_mtp_lanes() -> Result<()> {
        let core1 = fake_core(1);
        let storage1 = core1.storage();
        let cfg = Config {
            version: 1,
            max_queued: 8,
            max_sequences: 3,
            max_batch_tokens: 16,
            prefill_chunk_tokens: 4,
            max_context_tokens: 511,
            max_new_tokens: 16,
            model_memory_bytes: 1024 * 1024 * 1024,
            max_in_flight: 3,
        };
        let engine1 = Engine::new(cfg.clone(), storage1, core1)?;
        let baseline = engine1.submit(vec![1, 2, 3, 4, 5, 6], 7)?.result().await?;
        assert_eq!(engine1.shutdown().await?.memory_used, 0);

        let core4 = fake_core(4);
        let storage4 = core4.storage();
        let engine4 = Engine::new(cfg, storage4, core4)?;
        let mut streamed = engine4.submit_stream(vec![1, 2, 3, 4, 5, 6], 7, 2)?;
        let batch_req = engine4.submit(vec![1, 2, 3, 4, 5, 6], 7)?;
        let canceled = engine4.submit_stream(vec![3, 4, 5], 8, 1)?;
        drop(canceled);

        let mut events = Vec::new();
        while let Some(ev) = streamed.next_token().await {
            events.push(ev.token);
        }
        let stream_gen = streamed.result().await?;
        let batch_gen = batch_req.result().await?;
        assert_eq!(stream_gen.tokens, baseline.tokens);
        assert_eq!(batch_gen.tokens, baseline.tokens);
        assert_eq!(events, baseline.tokens);
        let stats = engine4.shutdown().await?;
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.cancelled, 1);
        assert_eq!(stats.memory_used, 0);
        Ok(())
    }
}
