use crate::{
    execute::{Inputs, Output},
    telemetry::Span,
};
use anyhow::{ensure, Result};
use serde::Serialize;
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};
use tokio::sync::{mpsc, oneshot};
static NEXT_JOB: AtomicU64 = AtomicU64::new(1);
pub struct ChainJob {
    pub traces: Vec<Vec<u8>>,
    pub inputs: Inputs,
}
pub struct Job {
    pub trace: Vec<u8>,
    pub inputs: Inputs,
}
#[derive(Debug, Serialize)]
pub struct Failure {
    pub stage: String,
    pub error: String,
}
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub validation_policy: &'static str,
    pub job_id: u64,
    pub worker_id: usize,
    pub flight: crate::completion::FlightStats,
    pub kernel: &'static str,
    pub shape: Option<[u32; 3]>,
    pub wall_ms: f64,
    pub queue_ms: f64,
    pub signpost_id: u64,
    pub signpost_enabled: bool,
    pub output: Option<Output>,
    pub failure: Option<Failure>,
}
struct Envelope {
    id: u64,
    job: ChainJob,
    lease: Option<crate::cache::StepLease>,
    submitted: Instant,
    reply: oneshot::Sender<Receipt>,
}
pub struct Runtime {
    tx: mpsc::Sender<Envelope>,
    gate: crate::completion::FlightGate,
    validator: crate::sampled::Validator,
}
impl Runtime {
    pub fn new(capacity: usize) -> Result<Self> {
        Self::with_in_flight(capacity, 3)
    }
    pub fn flight_stats(&self) -> crate::completion::FlightStats {
        self.gate.stats()
    }
    pub fn with_in_flight(capacity: usize, limit: usize) -> Result<Self> {
        Self::with_grid(capacity, limit, None)
    }
    pub fn with_grid(capacity: usize, limit: usize, threads: Option<u64>) -> Result<Self> {
        Self::with_sampling(
            capacity,
            limit,
            threads,
            crate::sampled::ValidationConfig::deterministic_bench(),
        )
    }
    pub fn with_sampling(
        capacity: usize,
        limit: usize,
        threads: Option<u64>,
        sampling: crate::sampled::ValidationConfig,
    ) -> Result<Self> {
        let sampling = sampling.validate()?;
        ensure!(
            threads.is_none_or(|n| [64, 128, 256, 512].contains(&n)),
            "unsupported threadgroup width"
        );
        ensure!((1..=32).contains(&capacity), "queue capacity outside 1..32");
        let gate = crate::completion::FlightGate::new(limit)?;
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow::anyhow!("Tokio runtime required"))?;
        let validator = crate::sampled::Validator::with_config(&handle, sampling)?;
        let (tx, rx) = mpsc::channel::<Envelope>(capacity);
        let receiver = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
        for worker_id in 0..limit {
            let validator = validator.clone();
            let receiver = receiver.clone();
            let gate = gate.clone();
            handle.spawn(async move {
                use crate::backend::{Config, MetalBackend};
                let mut backend: Option<MetalBackend> = None;
                loop {
                    let envelope = receiver.lock().await.recv().await;
                    let Some(envelope) = envelope else { break };
                    let start = Instant::now();
                    let mut span = Span::begin(envelope.id);
                    let mut kernel = "unresolved";
                    let mut shape = None;
                    let mut permit = match gate.acquire().await {
                        Ok(p) => p,
                        Err(_) => break,
                    };
                    if let Some(lease) = envelope.lease {
                        permit.retain(lease);
                    }
                    let config = Config {
                        threadgroup_threads: threads,
                        resident_bytes: Config::default().resident_bytes / limit,
                        idle_bytes: Config::default().idle_bytes / limit,
                        ..Config::default()
                    };
                    let job = envelope.job;
                    let prepare_gate = gate.clone();
                    let prepared = tokio::task::spawn_blocking(move || {
                        let mut stage = "decode";
                        let mut kind = "unresolved";
                        let mut dimensions = None;
                        let result = catch_unwind(AssertUnwindSafe(|| -> Result<_> {
                            let mut plans = Vec::new();
                            for bytes in &job.traces {
                                let trace = crate::wire::verified(bytes)?;
                                stage = "lower";
                                plans.push(crate::ir::lower_verified(trace)?);
                                stage = "decode";
                            }
                            let plan = plans.last().expect("nonempty admission");
                            kind = if plans.len() == 1 {
                                plan.kind.entry()
                            } else {
                                "chain"
                            };
                            dimensions = Some([plan.m, plan.k, plan.n]);
                            stage = "inputs";
                            crate::chain::validate(&plans, &job.inputs)?;
                            stage = "compile_dispatch_validate";
                            if backend.is_none() {
                                backend = Some(MetalBackend::with_gate(config, prepare_gate)?);
                            }
                            backend
                                .as_mut()
                                .expect("backend initialized")
                                .submit_chain_owned(plans, job.inputs, permit)
                        }));
                        let (pending, failure, panicked) = match result {
                            Ok(Ok(p)) => (Some(p), None, false),
                            Ok(Err(e)) => (
                                None,
                                Some(Failure {
                                    stage: stage.into(),
                                    error: e.to_string(),
                                }),
                                false,
                            ),
                            Err(_) => {
                                backend = None;
                                (
                                    None,
                                    Some(Failure {
                                        stage: stage.into(),
                                        error: "worker panic isolated; backend discarded".into(),
                                    }),
                                    true,
                                )
                            }
                        };
                        (backend, pending, failure, kind, dimensions, panicked)
                    })
                    .await;
                    let mut output = None;
                    let mut sample = None;
                    let mut failure;
                    match prepared {
                        Ok((returned, pending, error, kind, dimensions, panicked)) => {
                            backend = returned;
                            kernel = kind;
                            shape = dimensions;
                            failure = error;
                            if panicked {
                                gate.drain().await;
                            }
                            if let Some(pending) = pending {
                                let completed = pending.wait().await;
                                let finished = tokio::task::spawn_blocking(move || {
                                    let result = catch_unwind(AssertUnwindSafe(|| {
                                        backend
                                            .as_mut()
                                            .expect("backend initialized")
                                            .finish_with_validation_input(completed)
                                    }));
                                    match result {
                                        Ok(out) => (backend, out, false),
                                        Err(_) => (
                                            None,
                                            Err(anyhow::anyhow!(
                                                "validation panic isolated; backend discarded"
                                            )),
                                            true,
                                        ),
                                    }
                                })
                                .await;
                                match finished {
                                    Ok((returned, Ok((out, validation_input)), _)) => {
                                        if validator.receipt_is_sampled() {
                                            sample = Some(crate::sampled::Sample {
                                                input: validation_input,
                                                actual: out.values.clone(),
                                            });
                                        }
                                        backend = returned;
                                        output = Some(out);
                                    }
                                    Ok((returned, Err(e), panicked)) => {
                                        backend = returned;
                                        failure = Some(Failure {
                                            stage: "compile_dispatch_validate".into(),
                                            error: e.to_string(),
                                        });
                                        if panicked {
                                            gate.drain().await;
                                        }
                                    }
                                    Err(e) => {
                                        backend = None;
                                        failure = Some(Failure {
                                            stage: "worker".into(),
                                            error: e.to_string(),
                                        });
                                        gate.drain().await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            backend = None;
                            failure = Some(Failure {
                                stage: "worker".into(),
                                error: e.to_string(),
                            });
                            gate.drain().await;
                        }
                    }
                    span.success = failure.is_none();
                    let receipt = Receipt {
                        validation_policy: validator.policy_label(),
                        job_id: envelope.id,
                        worker_id,
                        flight: gate.stats(),
                        kernel,
                        shape,
                        wall_ms: envelope.submitted.elapsed().as_secs_f64() * 1000.,
                        queue_ms: start.duration_since(envelope.submitted).as_secs_f64() * 1000.,
                        signpost_id: span.id,
                        signpost_enabled: span.enabled,
                        output,
                        failure,
                    };
                    drop(span);
                    let _ = envelope.reply.send(receipt);
                    if let Some(sample) = sample {
                        validator.enqueue(sample).await;
                    }
                }
            });
        }
        Ok(Self {
            tx,
            gate,
            validator,
        })
    }
    /// Bounded, nonblocking admission. Ownership freezes inputs until GPU completion.
    pub fn submit(&self, job: Job) -> Result<oneshot::Receiver<Receipt>> {
        self.submit_chain(ChainJob {
            traces: vec![job.trace],
            inputs: job.inputs,
        })
    }
    pub fn validation_stats(&self) -> crate::sampled::ValidationStats {
        self.validator.stats()
    }
    /// Call after awaiting all receipts to wait for their reserved background checks.
    pub async fn flush_validation(&self) -> Result<crate::sampled::ValidationStats> {
        self.validator.drain().await
    }
    pub fn submit_chain(&self, job: ChainJob) -> Result<oneshot::Receiver<Receipt>> {
        self.submit_work(job, None)
    }
    pub fn submit_cached(
        &self,
        job: ChainJob,
        lease: crate::cache::StepLease,
    ) -> Result<oneshot::Receiver<Receipt>> {
        self.submit_work(job, Some(lease))
    }
    fn submit_work(
        &self,
        job: ChainJob,
        lease: Option<crate::cache::StepLease>,
    ) -> Result<oneshot::Receiver<Receipt>> {
        ensure!(self.validator.healthy(), "background validation unhealthy");
        ensure!(
            (1..=16).contains(&job.traces.len()),
            "chain stage bound 1..16"
        );
        ensure!(
            job.traces
                .iter()
                .all(|bytes| (8..=65536).contains(&bytes.len())),
            "trace size budget"
        );
        ensure!(job.inputs.len() <= 32, "input count budget");
        let count: usize = job.inputs.values().map(Vec::len).sum();
        ensure!(count <= 1048576, "admission input budget (4 MiB)");
        let (reply, rx) = oneshot::channel();
        let id = NEXT_JOB.fetch_add(1, Ordering::Relaxed);
        self.tx
            .try_send(Envelope {
                id,
                job,
                lease,
                submitted: Instant::now(),
                reply,
            })
            .map_err(|e| anyhow::anyhow!("admission failed: {e}"))?;
        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn background_mismatch_rejects_new_admission() {
        let runtime = Runtime::new(8).unwrap();
        let trace = include_bytes!("../fixtures/elementwise.amvk").to_vec();
        let plan = crate::ir::lower_trace(&trace).unwrap();
        let inputs = crate::execute::seeded_inputs(&plan, 42);
        for _ in 0..1000 {
            runtime.validator.receipt_is_sampled();
        }
        runtime
            .validator
            .enqueue(crate::sampled::Sample {
                input: crate::chain::ValidationInput {
                    plans: vec![plan],
                    inputs: inputs.clone(),
                },
                actual: vec![0],
            })
            .await;
        assert!(tokio::time::timeout(
            std::time::Duration::from_secs(5),
            runtime.flush_validation()
        )
        .await
        .unwrap()
        .is_err());
        assert!(runtime.submit(Job { trace, inputs }).is_err());
    }
}
