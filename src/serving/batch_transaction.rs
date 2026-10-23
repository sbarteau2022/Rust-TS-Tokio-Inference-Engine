//! Scalar transaction boundary for future physical batching. No tensor fusion occurs here.
use super::{
    model_ir::{ModelStepIr, StepKind},
    scheduler::{StepInput, StepOutput},
};
use anyhow::{ensure, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
};

/// The owning backend supplies leases; this trait does not grant native handle authority.
pub trait Parent: Clone {
    fn position(&self) -> usize;
    fn bytes(&self) -> usize;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub kind: StepKind,
    pub position: usize,
    pub tokens: usize,
}
#[derive(Debug)]
pub struct Cohort {
    pub geometry: Geometry,
    /// Original input indices, never GPU row counts or a claim of tensor packing.
    pub rows: Vec<usize>,
}
struct Row<P> {
    sequence: u64,
    parent: P,
    ir: ModelStepIr,
    complete: bool,
    allowed: Option<Vec<u32>>,
}
pub struct PreparedBatch<'a, P> {
    committed: &'a mut BTreeMap<u64, P>,
    rows: Vec<Row<P>>,
    cohorts: Vec<Cohort>,
    vocabulary: usize,
}
pub struct CompletedBatch<'a, P> {
    committed: &'a mut BTreeMap<u64, P>,
    states: Vec<(u64, P)>,
    outputs: Vec<StepOutput>,
}
impl<'a, P: Parent> PreparedBatch<'a, P> {
    /// Preparation invokes no executor. All rows belong to the caller's one backend identity.
    /// The borrow freezes the exact destination map through completion/commit:
    /// ```compile_fail
    /// use rust_server::serving::batch_transaction::{Parent, PreparedBatch};
    /// use std::collections::BTreeMap;
    /// #[derive(Clone)] struct P;
    /// impl Parent for P {
    ///     fn position(&self) -> usize { 0 }
    ///     fn bytes(&self) -> usize { 0 }
    /// }
    /// let mut states = BTreeMap::<u64, P>::new();
    /// let prepared = PreparedBatch::prepare(vec![], &mut states, || P, 16);
    /// states.clear(); // Cannot mutate a destination retained by the transaction.
    /// drop(prepared);
    /// ```
    pub fn prepare(
        batch: Vec<StepInput>,
        committed: &'a mut BTreeMap<u64, P>,
        empty: impl Fn() -> P,
        vocabulary: u32,
    ) -> Result<Self> {
        ensure!(!batch.is_empty() && batch.len() <= 3, "batch row bound");
        let new = batch
            .iter()
            .filter(|r| !committed.contains_key(&r.sequence))
            .count();
        ensure!(
            committed.len().checked_add(new).is_some_and(|n| n <= 3),
            "batch resident bound"
        );
        let mut seen = BTreeSet::new();
        let mut rows = Vec::with_capacity(batch.len());
        let mut cohorts: Vec<Cohort> = Vec::new();
        for item in batch {
            ensure!(
                seen.insert(item.sequence) && item.history.is_empty(),
                "batch duplicate/history"
            );
            ensure!(
                (1..=128).contains(&item.remaining_output_tokens),
                "batch output bound"
            );
            let parent = committed
                .get(&item.sequence)
                .cloned()
                .unwrap_or_else(&empty);
            ensure!(parent.position() == item.position, "batch parent position");
            if let Some(mask) = &item.allowed_tokens {
                ensure!(
                    !mask.is_empty() && mask.len() <= 32 && mask.iter().all(|t| *t < vocabulary),
                    "batch token mask"
                );
            }
            let ir = ModelStepIr::compile(item.kind, item.position, item.tokens, vocabulary)?;
            let geometry = Geometry {
                kind: ir.kind(),
                position: ir.position().get(),
                tokens: ir.len(),
            };
            let index = rows.len();
            if let Some(cohort) = cohorts.iter_mut().find(|g| g.geometry == geometry) {
                cohort.rows.push(index);
            } else {
                cohorts.push(Cohort {
                    geometry,
                    rows: vec![index],
                });
            }
            rows.push(Row {
                sequence: item.sequence,
                parent,
                ir,
                complete: item.prompt_complete,
                allowed: item.allowed_tokens,
            });
        }
        Ok(Self {
            committed,
            rows,
            cohorts,
            vocabulary: vocabulary as usize,
        })
    }
    pub fn cohorts(&self) -> &[Cohort] {
        &self.cohorts
    }
    /// Scalar input order is retained. A dropped/failed future exposes no CompletedBatch.
    /// The caller must retain completion ownership for native work already dispatched.
    pub async fn execute<F, Fut>(self, mut executor: F) -> Result<CompletedBatch<'a, P>>
    where
        F: FnMut(P, ModelStepIr) -> Fut,
        Fut: Future<Output = Result<(P, Vec<f32>)>>,
    {
        let mut states = Vec::with_capacity(self.rows.len());
        let mut outputs = Vec::with_capacity(self.rows.len());
        for row in self.rows {
            let expected = row.ir.position().get() + row.ir.len();
            let (state, logits) = executor(row.parent, row.ir).await?;
            ensure!(state.position() == expected, "batch result position");
            ensure!(logits.len() == self.vocabulary, "batch result logits");
            let next_token = if row.complete {
                Some(super::choices::select(&logits, row.allowed.as_deref())?)
            } else {
                ensure!(logits.iter().all(|v| v.is_finite()), "batch result logits");
                None
            };
            outputs.push(StepOutput {
                sequence: row.sequence,
                kv: Vec::new(),
                next_token,
                state_bytes: state.bytes(),
            });
            states.push((row.sequence, state));
        }
        Ok(CompletedBatch {
            committed: self.committed,
            states,
            outputs,
        })
    }
}
impl<P> CompletedBatch<'_, P> {
    /// Call under the backend's exclusive mutable borrow after successful full execution.
    /// No fallible model work or publication occurs between these map replacements.
    pub fn commit(self) -> Vec<StepOutput> {
        for (id, state) in self.states {
            self.committed.insert(id, state);
        }
        self.outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, Weak};
    #[derive(Clone)]
    struct Fake {
        position: usize,
        tag: u32,
        lease: Arc<()>,
    }
    impl Parent for Fake {
        fn position(&self) -> usize {
            self.position
        }
        fn bytes(&self) -> usize {
            16
        }
    }
    fn state(position: usize, tag: u32) -> Fake {
        Fake {
            position,
            tag,
            lease: Arc::new(()),
        }
    }
    fn parents() -> BTreeMap<u64, Fake> {
        [(1, state(2, 11)), (2, state(2, 22))].into()
    }
    fn row(sequence: u64) -> StepInput {
        StepInput {
            sequence,
            position: 2,
            kind: StepKind::Decode,
            tokens: vec![sequence as u32],
            remaining_output_tokens: 8,
            prompt_complete: true,
            allowed_tokens: None,
            history: Vec::new(),
        }
    }
    fn advance(old: Fake, ir: ModelStepIr) -> (Fake, Vec<f32>) {
        let next = state(
            old.position + ir.len(),
            old.tag + ir.ffi_ids().iter().sum::<u32>(),
        );
        let mut logits = vec![0.; 16];
        logits[(next.tag % 16) as usize] = 1.;
        (next, logits)
    }
    fn unchanged(map: &BTreeMap<u64, Fake>) {
        assert_eq!(map[&1].position, 2);
        assert_eq!(map[&1].tag, 11);
        assert_eq!(map[&2].position, 2);
        assert_eq!(map[&2].tag, 22);
        assert_eq!(Arc::strong_count(&map[&1].lease), 1);
        assert_eq!(Arc::strong_count(&map[&2].lease), 1);
    }
    #[test]
    fn invalid_batches_release_prepared_leases_before_any_execution() {
        for mode in 0..9 {
            let mut map = parents();
            let mut batch = vec![row(1), row(2)];
            match mode {
                0 => batch[1].sequence = 1,
                1 => batch[1].position = usize::MAX,
                2 => batch[1].tokens = vec![16],
                3 => batch[1].tokens = vec![],
                4 => batch[1].remaining_output_tokens = 0,
                5 => batch[1].remaining_output_tokens = 129,
                6 => batch[1].allowed_tokens = Some(vec![]),
                7 => batch[1].allowed_tokens = Some(vec![16]),
                _ => batch.extend([row(1), row(2)]),
            }
            assert!(PreparedBatch::prepare(batch, &mut map, || state(0, 0), 16).is_err());
            unchanged(&map);
        }
        let mut map = parents();
        map.insert(3, state(0, 0));
        assert!(PreparedBatch::prepare(vec![row(4)], &mut map, || state(0, 0), 16).is_err());
        assert!(PreparedBatch::prepare(vec![], &mut map, || state(0, 0), 16).is_err());
    }
    #[test]
    fn cohorts_describe_geometry_without_reordering_rows() -> Result<()> {
        let mut map = parents();
        let mut third = row(3);
        third.kind = StepKind::Prefill;
        third.position = 0;
        third.tokens = vec![3, 4];
        third.prompt_complete = false;
        let batch =
            PreparedBatch::prepare(vec![row(2), third, row(1)], &mut map, || state(0, 0), 16)?;
        assert_eq!(batch.cohorts().len(), 2);
        assert_eq!(batch.cohorts()[0].rows, vec![0, 2]);
        assert_eq!(batch.cohorts()[1].rows, vec![1]);
        assert_eq!(
            batch.rows.iter().map(|r| r.sequence).collect::<Vec<_>>(),
            vec![2, 3, 1]
        );
        Ok(())
    }
    #[tokio::test]
    async fn later_row_failure_drops_provisionals_and_preserves_all_parents() -> Result<()> {
        let mut map = parents();
        let leases: Arc<Mutex<Vec<Weak<()>>>> = Arc::default();
        let observed = leases.clone();
        let mut calls = 0;
        let batch = PreparedBatch::prepare(vec![row(1), row(2)], &mut map, || state(0, 0), 16)?;
        let result = batch
            .execute(move |old, ir| {
                calls += 1;
                let fail = calls == 2;
                let leases = observed.clone();
                async move {
                    if fail {
                        anyhow::bail!("injected later-row backend failure");
                    }
                    let (next, logits) = advance(old, ir);
                    leases.lock().unwrap().push(Arc::downgrade(&next.lease));
                    Ok((next, logits))
                }
            })
            .await;
        assert!(result.is_err());
        unchanged(&map);
        assert!(leases.lock().unwrap().iter().all(|w| w.upgrade().is_none()));
        Ok(())
    }
    #[tokio::test]
    async fn malformed_native_results_cannot_create_a_commit() -> Result<()> {
        for mode in 0..3 {
            let mut map = parents();
            let mut calls = 0;
            let batch = PreparedBatch::prepare(vec![row(1), row(2)], &mut map, || state(0, 0), 16)?;
            let result = batch
                .execute(move |old, ir| {
                    calls += 1;
                    let bad = calls == 2;
                    async move {
                        let (mut next, mut logits) = advance(old, ir);
                        if bad {
                            match mode {
                                0 => next.position += 1,
                                1 => logits[0] = f32::NAN,
                                _ => {
                                    logits.pop();
                                }
                            }
                        }
                        Ok((next, logits))
                    }
                })
                .await;
            assert!(result.is_err());
            unchanged(&map);
        }
        Ok(())
    }
    #[tokio::test]
    async fn interrupted_scalar_future_cannot_publish_a_partial_batch() -> Result<()> {
        let mut map = parents();
        let leases: Arc<Mutex<Vec<Weak<()>>>> = Arc::default();
        let observed = leases.clone();
        let mut calls = 0;
        let batch = PreparedBatch::prepare(vec![row(1), row(2)], &mut map, || state(0, 0), 16)?;
        let mut execution = Box::pin(batch.execute(move |old, ir| {
            calls += 1;
            let pause = calls == 2;
            let leases = observed.clone();
            async move {
                if pause {
                    std::future::pending::<()>().await;
                }
                let (next, logits) = advance(old, ir);
                leases.lock().unwrap().push(Arc::downgrade(&next.lease));
                Ok((next, logits))
            }
        }));
        std::future::poll_fn(|cx| {
            assert!(execution.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        assert_eq!(leases.lock().unwrap().len(), 1);
        drop(execution);
        unchanged(&map);
        assert!(leases.lock().unwrap().iter().all(|w| w.upgrade().is_none()));
        Ok(())
    }

    #[tokio::test]
    async fn dropping_completed_batch_is_rollback_not_commit() -> Result<()> {
        let mut map = parents();
        let leases: Arc<Mutex<Vec<Weak<()>>>> = Arc::default();
        let observed = leases.clone();
        let batch = PreparedBatch::prepare(vec![row(1), row(2)], &mut map, || state(0, 0), 16)?;
        let completed = batch
            .execute(move |old, ir| {
                let leases = observed.clone();
                async move {
                    let (next, logits) = advance(old, ir);
                    leases.lock().unwrap().push(Arc::downgrade(&next.lease));
                    Ok((next, logits))
                }
            })
            .await?;
        assert!(leases.lock().unwrap().iter().all(|w| w.upgrade().is_some()));
        drop(completed);
        unchanged(&map);
        assert!(leases.lock().unwrap().iter().all(|w| w.upgrade().is_none()));
        Ok(())
    }
    #[tokio::test]
    async fn permutations_commit_same_sequence_state_and_release_old_leases() -> Result<()> {
        let mut answers = Vec::new();
        for order in [[1, 2], [2, 1]] {
            let mut map = parents();
            let old: Vec<_> = map.values().map(|p| Arc::downgrade(&p.lease)).collect();
            let batch = PreparedBatch::prepare(
                order.into_iter().map(row).collect(),
                &mut map,
                || state(0, 0),
                16,
            )?;
            let complete = batch
                .execute(|old, ir| async move { Ok(advance(old, ir)) })
                .await?;
            assert!(old.iter().all(|w| w.upgrade().is_some()));
            let outputs = complete.commit();
            assert!(old.iter().all(|w| w.upgrade().is_none()));
            assert_eq!(
                outputs.iter().map(|o| o.sequence).collect::<Vec<_>>(),
                order
            );
            let mut answer: Vec<_> = outputs
                .iter()
                .map(|o| {
                    (
                        o.sequence,
                        o.next_token,
                        map[&o.sequence].position,
                        map[&o.sequence].tag,
                    )
                })
                .collect();
            answer.sort();
            answers.push(answer);
        }
        assert_eq!(answers[0], answers[1]);
        Ok(())
    }
}
