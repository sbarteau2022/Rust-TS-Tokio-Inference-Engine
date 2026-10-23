//! Experimental greedy draft transactions. No production scheduler/default change.
use super::{CompletionGuard, DeviceState, QwenDeviceExecutor};
use anyhow::{ensure, Result};
use std::sync::{atomic::AtomicU64, Arc};

/// Only executor evaluation constructs this binding of state and next-token logits.
#[derive(Clone)]
pub struct GreedyCursor {
    state: DeviceState,
    logits: Arc<Vec<f32>>,
    terminal: bool,
}
impl GreedyCursor {
    pub fn state(&self) -> &DeviceState {
        &self.state
    }
    pub fn logits(&self) -> &[f32] {
        &self.logits
    }
    pub fn terminal(&self) -> bool {
        self.terminal
    }
}
pub struct GreedyTransaction {
    pub cursor: GreedyCursor,
    pub emitted: Vec<u32>,
    pub proposed: usize,
    pub accepted: usize,
    /// Includes the block verifier and correction/bonus replay calls.
    pub target_calls: usize,
    pub replayed_tokens: usize,
}
struct Plan {
    emitted: Vec<u32>,
    accepted: usize,
    terminal: bool,
}
/// Fixed tie-break: the lowest vocabulary index wins, matching native greedy select.
fn winner(logits: &[f32]) -> Result<u32> {
    ensure!(
        !logits.is_empty() && logits.len() <= 300_000 && logits.iter().all(|v| v.is_finite()),
        "greedy logits finite/shape"
    );
    let mut best = 0;
    for i in 1..logits.len() {
        if logits[i].total_cmp(&logits[best]).is_gt() {
            best = i;
        }
    }
    Ok(best as u32)
}
fn plan(
    initial: &[f32],
    draft: &[u32],
    rows: &[Vec<f32>],
    eos: &[u32],
    remaining: usize,
) -> Result<Plan> {
    let vocab = initial.len();
    winner(initial)?;
    ensure!(
        draft.len() <= 8
            && draft.len() <= remaining
            && remaining > 0
            && draft.iter().chain(eos).all(|&t| (t as usize) < vocab)
            && rows.len() == draft.len(),
        "greedy draft/rows/context geometry"
    );
    // Validate even the rejected suffix: a malformed native receipt never commits.
    for row in rows {
        ensure!(row.len() == vocab, "verifier row vocabulary");
        winner(row)?;
    }
    let mut emitted = Vec::new();
    let mut accepted = 0;
    let mut terminal = false;
    for i in 0..draft.len() {
        let target = winner(if i == 0 { initial } else { &rows[i - 1] })?;
        emitted.push(target);
        terminal = eos.contains(&target);
        if draft[i] != target || terminal {
            accepted += usize::from(draft[i] == target);
            return Ok(Plan {
                emitted,
                accepted,
                terminal,
            });
        }
        accepted += 1;
    }
    if emitted.len() < remaining {
        let bonus = winner(rows.last().map_or(initial, Vec::as_slice))?;
        emitted.push(bonus);
        terminal = eos.contains(&bonus);
    }
    Ok(Plan {
        emitted,
        accepted,
        terminal,
    })
}
impl QwenDeviceExecutor {
    /// Prefill one nonempty chunk. Use `greedy_extend` for subsequent prompt chunks.
    pub async fn greedy_start(&self, tokens: Vec<u32>) -> Result<GreedyCursor> {
        let result = self.step(&self.empty_state(), tokens).await?;
        winner(&result.logits)?;
        Ok(GreedyCursor {
            state: result.state,
            logits: Arc::new(result.logits),
            terminal: false,
        })
    }
    /// Prompt extension only; tokenization/chat templates stay at the input boundary.
    pub async fn greedy_extend(
        &self,
        cursor: &GreedyCursor,
        tokens: Vec<u32>,
    ) -> Result<GreedyCursor> {
        ensure!(!cursor.terminal, "cannot extend terminal cursor");
        let result = self.step(&cursor.state, tokens).await?;
        winner(&result.logits)?;
        Ok(GreedyCursor {
            state: result.state,
            logits: Arc::new(result.logits),
            terminal: false,
        })
    }
    /// Publish only after full block validation and all accepted-state replay completes.
    /// The input cursor remains immutable and valid on error or cancellation.
    pub async fn greedy_verify(
        &self,
        cursor: &GreedyCursor,
        draft: Vec<u32>,
        eos: &[u32],
    ) -> Result<GreedyTransaction> {
        self.greedy_verify_guarded(cursor, draft, eos, None).await
    }
    pub async fn greedy_verify_guarded(
        &self,
        cursor: &GreedyCursor,
        draft: Vec<u32>,
        eos: &[u32],
        completion: Option<CompletionGuard>,
    ) -> Result<GreedyTransaction> {
        self.greedy_verify_accounted(cursor, draft, eos, completion, None)
            .await
    }
    pub(super) async fn greedy_verify_accounted(
        &self,
        cursor: &GreedyCursor,
        draft: Vec<u32>,
        eos: &[u32],
        completion: Option<CompletionGuard>,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<GreedyTransaction> {
        ensure!(!cursor.terminal, "cannot decode terminal cursor");
        let remaining = 512usize.saturating_sub(cursor.state.position());
        ensure!(
            remaining > 0 && draft.len() <= 8 && draft.len() <= remaining,
            "greedy context/draft capacity"
        );
        let vocab = cursor.logits.len();
        ensure!(
            draft.iter().chain(eos).all(|&t| (t as usize) < vocab),
            "greedy token vocabulary"
        );
        // One immutable provisional version, never inserted into the committed cursor.
        let block = if draft.is_empty() {
            None
        } else {
            Some(
                self.verify_block_guarded(
                    &cursor.state,
                    draft.clone(),
                    completion.clone(),
                    calls.clone(),
                )
                .await?,
            )
        };
        let rows = block.as_ref().map_or(&[][..], |b| b.logits.as_slice());
        let decision = plan(&cursor.logits, &draft, rows, eos, remaining)?;
        let proposed = draft.len();
        let (state, logits, calls, replayed) = if decision.accepted == draft.len()
            && !draft.is_empty()
            && decision.emitted.len() == draft.len()
        {
            let block = block.expect("nonempty draft evaluated");
            (
                block.state,
                block.logits.last().expect("validated rows").clone(),
                1,
                0,
            )
        } else if decision.accepted == draft.len() && !draft.is_empty() {
            let block = block.expect("nonempty draft evaluated");
            let result = self
                .step_guarded(
                    &block.state,
                    vec![*decision.emitted.last().expect("bonus")],
                    completion.clone(),
                    calls.clone(),
                )
                .await?;
            (result.state, result.logits, 2, 0)
        } else {
            drop(block); // Reclaim rejected recurrent/convolution/KV state before replay.
            let replayed = decision.emitted.len();
            let result = self
                .step_guarded(
                    &cursor.state,
                    decision.emitted.clone(),
                    completion.clone(),
                    calls.clone(),
                )
                .await?;
            (
                result.state,
                result.logits,
                usize::from(proposed > 0) + 1,
                if proposed > 0 { replayed } else { 0 },
            )
        };
        winner(&logits)?;
        ensure!(
            state.position() == cursor.state.position() + decision.emitted.len(),
            "greedy atomic position"
        );
        Ok(GreedyTransaction {
            cursor: GreedyCursor {
                state,
                logits: Arc::new(logits),
                terminal: decision.terminal,
            },
            emitted: decision.emitted,
            proposed,
            accepted: decision.accepted,
            target_calls: calls,
            replayed_tokens: replayed,
        })
    }
}

/// Bounded, request-local token suffix retrieval. Feed committed IDs only.
/// No workspace/global corpus, tenant sharing or learned head is implicit here.
pub struct SuffixDraft {
    history: Vec<u32>,
    capacity: usize,
}
impl SuffixDraft {
    pub fn new(capacity: usize) -> Result<Self> {
        ensure!((16..=65_536).contains(&capacity), "suffix history capacity");
        Ok(Self {
            history: Vec::new(),
            capacity,
        })
    }
    pub fn observe(&mut self, committed: &[u32]) {
        if committed.len() >= self.capacity {
            self.history.clear();
            self.history
                .extend_from_slice(&committed[committed.len() - self.capacity..]);
        } else {
            let overflow = (self.history.len() + committed.len()).saturating_sub(self.capacity);
            self.history.drain(..overflow);
            self.history.extend_from_slice(committed);
        }
    }
    pub fn propose(&self, maximum: usize) -> Result<Vec<u32>> {
        ensure!(maximum <= 8, "suffix draft cap");
        let n = self.history.len();
        for width in (2..=16.min(n.saturating_sub(1))).rev() {
            for start in (0..n - width).rev() {
                if self.history[start..start + width] == self.history[n - width..] {
                    let end = (start + width + maximum).min(n);
                    return Ok(self.history[start + width..end].to_vec());
                }
            }
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(token: usize) -> Vec<f32> {
        let mut r = vec![-1.; 12];
        r[token] = 1.;
        r
    }
    #[test]
    fn every_acceptance_boundary_bonus_eos_context_and_ties() {
        for n in 1..=8 {
            for accepted in 0..=n {
                let mut draft = vec![1; n];
                if accepted < n {
                    draft[accepted] = 2;
                }
                let rows = vec![row(1); n];
                let p = plan(&row(1), &draft, &rows, &[11], 9).unwrap();
                assert_eq!(p.accepted, accepted);
                assert_eq!(p.emitted, vec![1; accepted + 1]);
                assert!(!p.terminal);
                for stop in 0..=accepted.min(n - 1) {
                    let mut rows = rows.clone();
                    let mut initial = row(1);
                    if stop == 0 {
                        initial = row(11);
                    } else {
                        rows[stop - 1] = row(11);
                    }
                    let p = plan(&initial, &draft, &rows, &[11], 9).unwrap();
                    assert!(p.terminal);
                    assert_eq!(p.emitted.len(), stop + 1);
                    assert_eq!(p.emitted[stop], 11);
                }
            }
            let p = plan(&row(1), &vec![1; n], &vec![row(1); n], &[], n).unwrap();
            assert_eq!(p.emitted.len(), n);
        }
        assert_eq!(winner(&[-0., 0., -1.]).unwrap(), 1);
        assert_eq!(plan(&row(1), &[], &[], &[], 1).unwrap().emitted, [1]);
        let p = plan(&row(11), &[11, 2], &[row(1), row(1)], &[11], 4).unwrap();
        assert!(p.terminal);
        assert_eq!(p.accepted, 1);
        assert_eq!(p.emitted, [11]);
    }
    #[test]
    fn tie_break_matches_existing_selector_under_pressure() {
        for n in 1..=512 {
            let values: Vec<f32> = (0..n)
                .map(|i| ((i * 17 + n * 31) % 23) as f32 - 11.)
                .collect();
            assert_eq!(
                winner(&values).unwrap(),
                super::super::super::choices::select(&values, None).unwrap()
            );
        }
        for values in [vec![-0., 0.], vec![0., -0.], vec![1., 1., 1.]] {
            assert_eq!(
                winner(&values).unwrap(),
                super::super::super::choices::select(&values, None).unwrap()
            );
        }
    }
    #[test]
    fn malformed_rejected_suffix_never_commits() {
        assert!(plan(&row(1), &[2], &[vec![f32::NAN; 12]], &[], 8).is_err());
        assert!(plan(&row(1), &[2], &[vec![f32::INFINITY; 12]], &[], 8).is_err());
        assert!(plan(&row(1), &[2], &[vec![0.; 11]], &[], 8).is_err());
        assert!(plan(&row(1), &[12], &[row(1)], &[], 8).is_err());
        assert!(plan(&row(1), &[], &[], &[12], 8).is_err());
        assert!(plan(&[], &[], &[], &[], 8).is_err());
        assert!(plan(&row(1), &[1; 9], &vec![row(1); 9], &[], 10).is_err());
        assert!(plan(&row(1), &[1], &[], &[], 8).is_err());
        assert!(plan(&row(1), &[1], &[row(1)], &[], 0).is_err());
    }
    #[test]
    fn suffix_is_bounded_committed_only_and_deterministic() {
        let mut d = SuffixDraft::new(16).unwrap();
        assert!(d.propose(8).unwrap().is_empty());
        d.observe(&[1, 2, 3, 4, 5, 1, 2]);
        assert_eq!(d.propose(3).unwrap(), [3, 4, 5]);
        assert_eq!(d.propose(3).unwrap(), [3, 4, 5]); // Proposals never alter history.
        assert!(d.propose(0).unwrap().is_empty());
        assert!(d.propose(9).is_err());
        d.observe(&[7; 100]);
        assert_eq!(d.history.len(), 16);
        d.observe(&[8, 9]);
        assert_eq!(d.history.len(), 16);
        assert!(SuffixDraft::new(0).is_err());
        assert!(SuffixDraft::new(65_537).is_err());
        for len in 0..100 {
            let mut d = SuffixDraft::new(16).unwrap();
            d.observe(&vec![1; len]);
            assert!(d.propose(8).unwrap().len() <= 8);
        }
    }
}
