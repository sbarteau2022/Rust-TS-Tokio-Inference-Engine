//! Proposers are Spec 15 workloads; this adapter supplies no second controller.
use super::{CompletionGuard, GreedyCursor, GreedyTransaction, QwenDeviceExecutor, SuffixDraft};
use crate::cache::{
    prefix_key, BlockKey, Controller, DraftAdmission, DraftOutcome, DraftPermit, Identity,
    ProposerId,
};
use anyhow::{ensure, Context, Result};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use tokio::sync::oneshot;
pub struct ProposalContext {
    pub committed: Arc<Vec<u32>>,
    pub cursor: GreedyCursor,
    pub maximum: usize,
    /// Native proposer work MUST retain this through backend completion.
    pub completion: CompletionGuard,
}
pub type ProposalFuture<'a> = Pin<Box<dyn Future<Output = Result<Vec<u32>>> + Send + 'a>>;
pub trait DraftProposer: Send + Sync {
    fn name(&self) -> &str;
    fn propose(&self, context: ProposalContext) -> ProposalFuture<'_>;
}
pub struct RetrievalProposer;
impl DraftProposer for RetrievalProposer {
    fn name(&self) -> &str {
        "request-local-suffix-v1"
    }
    fn propose(&self, context: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async move {
            let mut suffix = SuffixDraft::new(4096)?;
            suffix.observe(&context.committed);
            suffix.propose(context.maximum)
        })
    }
}
#[derive(Clone)]
pub struct AutomatonCursor {
    greedy: GreedyCursor,
    committed: Arc<Vec<u32>>,
    identity: Identity,
}
impl AutomatonCursor {
    pub fn greedy(&self) -> &GreedyCursor {
        &self.greedy
    }
    pub fn committed(&self) -> &[u32] {
        &self.committed
    }
}
pub struct AutomatonStep {
    pub cursor: AutomatonCursor,
    pub emitted: Vec<u32>,
    pub proposed: usize,
    pub accepted: usize,
    pub target_calls: usize,
    pub proposer: Option<ProposerId>,
    pub draft_key: Option<BlockKey>,
}
struct Running(Mutex<Option<DraftPermit>>);
impl Running {
    fn bind(&self, tokens: &[u32]) -> Result<BlockKey> {
        self.0
            .lock()
            .map_err(|_| anyhow::anyhow!("draft lease poisoned"))?
            .as_ref()
            .context("finished draft lease")?
            .bind(tokens)
    }
    fn finish(&self, outcome: DraftOutcome) -> Result<()> {
        self.0
            .lock()
            .map_err(|_| anyhow::anyhow!("draft lease poisoned"))?
            .take()
            .context("finished draft lease")?
            .finish(outcome)
    }
}
/// One plugin registration under an existing automaton. Cursors of the same exact
/// identity may be arbitrated across several registrations without rewriting state.
pub struct AutomatonProposer {
    controller: Controller,
    executor: Arc<QwenDeviceExecutor>,
    plugin: Arc<dyn DraftProposer>,
    id: ProposerId,
    identity: Identity,
}
impl AutomatonProposer {
    pub fn register(
        controller: Controller,
        executor: Arc<QwenDeviceExecutor>,
        namespace: String,
        plugin: Arc<dyn DraftProposer>,
    ) -> Result<Self> {
        // The artifact digest includes tokenizer and chat-template bytes as well as
        // weights: deliberately over-specific, never cross-tokenizer sharing.
        let identity = Identity {
            model: executor.identity().into(),
            tokenizer: executor.identity().into(),
            quantization: "fp32-explicit-attention-regular-kv-v1".into(),
            namespace,
        };
        let id = controller.register_proposer(plugin.name().into(), identity.clone())?;
        Ok(Self {
            controller,
            executor,
            plugin,
            id,
            identity,
        })
    }
    pub fn id(&self) -> ProposerId {
        self.id
    }
    /// Arbitrate only compatible plugins under this Controller; no external policy.
    pub async fn next_best(
        candidates: &[&Self],
        cursor: &AutomatonCursor,
        output_budget: usize,
        eos: &[u32],
    ) -> Result<AutomatonStep> {
        let first = candidates.first().context("empty proposer set")?;
        ensure!(
            output_budget > 0 && eos.len() <= 16,
            "automaton output/EOS bounds"
        );
        for candidate in candidates {
            candidate.validate(cursor)?;
        }
        let ids: Vec<_> = candidates.iter().map(|c| c.id).collect();
        match first.controller.choose_proposer(&ids)? {
            Some(id) => {
                candidates
                    .iter()
                    .find(|c| c.id == id)
                    .unwrap()
                    .next(cursor, output_budget, eos)
                    .await
            }
            None => first.no_draft(cursor, eos).await,
        }
    }
    pub async fn start(&self, prompt: Vec<u32>) -> Result<AutomatonCursor> {
        ensure!(
            !prompt.is_empty() && prompt.len() < 512,
            "automaton prompt capacity"
        );
        prefix_key(&self.identity, &prompt)?;
        let mut chunks = prompt.chunks(8);
        let mut greedy = self
            .executor
            .greedy_start(chunks.next().unwrap().to_vec())
            .await?;
        for chunk in chunks {
            greedy = self.executor.greedy_extend(&greedy, chunk.to_vec()).await?;
        }
        Ok(AutomatonCursor {
            greedy,
            committed: Arc::new(prompt),
            identity: self.identity.clone(),
        })
    }
    fn validate(&self, cursor: &AutomatonCursor) -> Result<()> {
        ensure!(
            cursor.identity == self.identity
                && cursor.committed.len() == cursor.greedy.state().position()
                && !cursor.greedy.terminal(),
            "foreign/terminal automaton cursor"
        );
        Ok(())
    }
    fn output(
        cursor: &AutomatonCursor,
        t: GreedyTransaction,
        proposer: Option<ProposerId>,
        draft_key: Option<BlockKey>,
    ) -> AutomatonStep {
        let mut committed = (*cursor.committed).clone();
        committed.extend_from_slice(&t.emitted);
        AutomatonStep {
            cursor: AutomatonCursor {
                greedy: t.cursor,
                committed: Arc::new(committed),
                identity: cursor.identity.clone(),
            },
            emitted: t.emitted,
            proposed: t.proposed,
            accepted: t.accepted,
            target_calls: t.target_calls,
            proposer,
            draft_key,
        }
    }
    pub async fn no_draft(&self, cursor: &AutomatonCursor, eos: &[u32]) -> Result<AutomatonStep> {
        self.validate(cursor)?;
        ensure!(eos.len() <= 16, "automaton EOS bounds");
        let start = Instant::now();
        let t = self
            .executor
            .greedy_verify(&cursor.greedy, vec![], eos)
            .await?;
        let cost = u64::try_from(start.elapsed().as_nanos())?.max(1);
        self.controller
            .record_no_draft(self.id, cost, t.emitted.len() as u64)?;
        Ok(Self::output(cursor, t, None, None))
    }
    /// Queued metadata is explicitly cancelled here and the baseline continues;
    /// a future scheduler can retain/poll tickets using the Controller API.
    pub async fn next(
        &self,
        cursor: &AutomatonCursor,
        output_budget: usize,
        eos: &[u32],
    ) -> Result<AutomatonStep> {
        self.validate(cursor)?;
        ensure!(
            output_budget > 0 && eos.len() <= 16,
            "automaton output/EOS bounds"
        );
        let maximum = 8
            .min(output_budget.saturating_sub(1))
            .min(512usize.saturating_sub(cursor.greedy.state().position()));
        if maximum == 0 {
            return self.no_draft(cursor, eos).await;
        }
        let start = Instant::now();
        let parent = prefix_key(&self.identity, &cursor.committed)?;
        let permit = match self
            .controller
            .request_draft(self.id, parent, maximum, true)?
        {
            DraftAdmission::Ready(p) => p,
            DraftAdmission::Benched => return self.no_draft(cursor, eos).await,
            DraftAdmission::Queued(t) => {
                self.controller.cancel_draft(t)?;
                return self.no_draft(cursor, eos).await;
            }
        };
        let running = Arc::new(Running(Mutex::new(Some(permit))));
        let guard: CompletionGuard = running.clone();
        let executor = self.executor.clone();
        let plugin = self.plugin.clone();
        let cursor = cursor.clone();
        let eos = eos.to_vec();
        let id = self.id;
        let (reply, receive) = oneshot::channel();
        let (done, finished) = oneshot::channel();
        // Caller cancellation detaches this private job. Native commands also own
        // guard clones, so runtime abort cannot retire the lease before completion.
        tokio::spawn(async move {
            let calls = Arc::new(AtomicU64::new(0));
            let proposal = plugin
                .propose(ProposalContext {
                    committed: cursor.committed.clone(),
                    cursor: cursor.greedy.clone(),
                    maximum,
                    completion: guard.clone(),
                })
                .await;
            let (draft, key, error) = match proposal {
                Ok(draft) if draft.len() <= maximum => match running.bind(&draft) {
                    Ok(k) => (draft, Some(k), None),
                    Err(e) => (Vec::new(), None, Some(e)),
                },
                Ok(_) => (
                    Vec::new(),
                    None,
                    Some(anyhow::anyhow!("plugin exceeded draft budget")),
                ),
                Err(e) => (Vec::new(), None, Some(e)),
            };
            let attempted_proposed = draft.len();
            let result = if let Some(error) = error {
                let _ = running.bind(&[]);
                Err(error)
            } else {
                executor
                    .greedy_verify_accounted(
                        &cursor.greedy,
                        draft,
                        &eos,
                        Some(guard.clone()),
                        Some(calls.clone()),
                    )
                    .await
                    .map(|t| Self::output(&cursor, t, Some(id), key))
            };
            let mut outcome = match &result {
                Ok(t) => DraftOutcome {
                    known_prefix: 0,
                    proposed: t.proposed as u64,
                    accepted: t.accepted as u64,
                    committed: t.emitted.len() as u64,
                    target_calls: calls.load(Ordering::Relaxed),
                    cost_ns: 0,
                    failed: false,
                },
                Err(_) => DraftOutcome {
                    known_prefix: 0,
                    proposed: attempted_proposed as u64,
                    accepted: 0,
                    committed: 0,
                    target_calls: calls.load(Ordering::Relaxed),
                    cost_ns: 0,
                    failed: true,
                },
            };
            outcome.cost_ns = u64::try_from(start.elapsed().as_nanos())
                .unwrap_or(u64::MAX)
                .max(1);
            if reply.send(result).is_err() {
                outcome.committed = 0;
            }
            let _ = done.send(running.finish(outcome));
        });
        let result = receive.await.context("proposer task stopped")?;
        finished.await.context("proposer accounting stopped")??;
        result
    }
}

include!("native_mtp_automaton.rs");
