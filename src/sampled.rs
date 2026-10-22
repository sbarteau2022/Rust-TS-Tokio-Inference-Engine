//! Bounded, post-receipt integer checking. A mismatch latches admission unhealthy.
use crate::{chain::ValidationInput, execute};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize)]
pub enum Selection {
    DeterministicBench,
    DynamicDraft,
}
#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize)]
pub struct ValidationConfig {
    pub selection: Selection,
    pub every: u64,
}
impl ValidationConfig {
    pub fn deterministic_bench() -> Self {
        Self {
            selection: Selection::DeterministicBench,
            every: 1000,
        }
    }
    pub fn validate(self) -> anyhow::Result<Self> {
        anyhow::ensure!(
            matches!(self.selection, Selection::DeterministicBench),
            "dynamic sampling policy remains an inactive draft"
        );
        anyhow::ensure!(
            (1..=1000).contains(&self.every),
            "deterministic cadence must be 1..1000"
        );
        Ok(self)
    }
}
#[derive(Default)]
struct State {
    receipts: AtomicU64,
    scheduled: AtomicU64,
    passed: AtomicU64,
    failed: AtomicU64,
    last_error: Mutex<Option<String>>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ValidationStats {
    pub policy_label: &'static str,
    pub selection: Selection,
    pub every: u64,
    pub production_sampling_claims: bool,
    pub successful_receipts: u64,
    pub scheduled: u64,
    pub passed: u64,
    pub failed: u64,
    pub last_error: Option<String>,
}
pub(crate) struct Sample {
    pub input: ValidationInput,
    pub actual: Vec<i32>,
}
#[derive(Clone)]
pub(crate) struct Validator {
    config: ValidationConfig,
    tx: mpsc::Sender<Sample>,
    state: Arc<State>,
}
impl Validator {
    #[cfg(test)]
    pub fn new(handle: &tokio::runtime::Handle) -> Self {
        Self::with_config(handle, ValidationConfig::deterministic_bench())
            .expect("valid bench configuration")
    }
    pub fn with_config(
        handle: &tokio::runtime::Handle,
        config: ValidationConfig,
    ) -> anyhow::Result<Self> {
        let config = config.validate()?;
        let (tx, mut rx) = mpsc::channel::<Sample>(3);
        let state = Arc::new(State::default());
        let worker = state.clone();
        handle.spawn(async move {
            while let Some(sample) = rx.recv().await {
                let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                    let expected =
                        crate::chain::reference(&sample.input.plans, &sample.input.inputs)?;
                    execute::compare(&sample.actual, &expected)?;
                    Ok(())
                })
                .await;
                let error = match result {
                    Ok(Ok(())) => None,
                    Ok(Err(e)) => Some(e.to_string()),
                    Err(e) => Some(e.to_string()),
                };
                if let Some(error) = error {
                    Self::fail(&worker, error);
                } else {
                    worker.passed.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        Ok(Self { tx, state, config })
    }
    fn fail(state: &State, error: String) {
        *state.last_error.lock().unwrap_or_else(|e| e.into_inner()) = Some(error.clone());
        state.failed.fetch_add(1, Ordering::SeqCst);
        eprintln!("background Q1.31 validation failed: {error}");
    }
    pub fn receipt_is_sampled(&self) -> bool {
        let count = self.state.receipts.fetch_add(1, Ordering::SeqCst) + 1;
        if count.is_multiple_of(self.config.every) {
            self.state.scheduled.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
    pub async fn enqueue(&self, sample: Sample) {
        if self.tx.send(sample).await.is_err() {
            Self::fail(&self.state, "sample worker unavailable".into());
        }
    }
    pub fn healthy(&self) -> bool {
        self.state.failed.load(Ordering::SeqCst) == 0
    }
    pub fn policy_label(&self) -> &'static str {
        "DETERMINISTIC-MODE"
    }
    pub fn stats(&self) -> ValidationStats {
        ValidationStats {
            policy_label: self.policy_label(),
            selection: self.config.selection,
            every: self.config.every,
            production_sampling_claims: false,
            successful_receipts: self.state.receipts.load(Ordering::SeqCst),
            scheduled: self.state.scheduled.load(Ordering::SeqCst),
            passed: self.state.passed.load(Ordering::SeqCst),
            failed: self.state.failed.load(Ordering::SeqCst),
            last_error: self
                .state
                .last_error
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        }
    }
    pub async fn drain(&self) -> anyhow::Result<ValidationStats> {
        loop {
            let stats = self.stats();
            if stats.passed + stats.failed == stats.scheduled {
                anyhow::ensure!(
                    stats.failed == 0,
                    "background validation failed: {:?}",
                    stats.last_error
                );
                return Ok(stats);
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn mismatch_latches_failure_and_drains() {
        let validator = Validator::new(&tokio::runtime::Handle::current());
        let plan = crate::ir::lower_trace(include_bytes!("../fixtures/elementwise.amvk")).unwrap();
        let inputs = execute::seeded_inputs(&plan, 42);
        for _ in 0..999 {
            assert!(!validator.receipt_is_sampled());
        }
        assert!(validator.receipt_is_sampled());
        validator
            .enqueue(Sample {
                input: ValidationInput {
                    plans: vec![plan],
                    inputs,
                },
                actual: vec![0],
            })
            .await;
        assert!(
            tokio::time::timeout(std::time::Duration::from_secs(5), validator.drain())
                .await
                .unwrap()
                .is_err()
        );
        assert_eq!(validator.stats().failed, 1);
        assert!(validator.stats().last_error.is_some());
    }
}
