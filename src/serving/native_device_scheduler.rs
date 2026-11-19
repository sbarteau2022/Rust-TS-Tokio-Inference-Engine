//! Experimental scheduler adapter, explicit snapshot/device selection for controlled A/B.
use super::*;
use std::sync::Mutex;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateMode {
    Snapshot,
    Device,
}
#[derive(Clone)]
enum State {
    Snapshot(ModelState),
    Device(DeviceState),
}
impl super::super::batch_transaction::Parent for State {
    fn position(&self) -> usize {
        match self {
            Self::Snapshot(s) => s.position(),
            Self::Device(s) => s.position(),
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Snapshot(s) => s.bytes(),
            Self::Device(s) => s.bytes(),
        }
    }
}
pub struct ScheduledDevice {
    executor: Arc<QwenDeviceExecutor>,
    states: BTreeMap<u64, State>,
    mode: StateMode,
    identity: String,
    eos: Vec<u32>,
    physical: bool,
    gpu_sampling: bool,
    counters: Arc<Mutex<BatchCounters>>,
    pub drain: Arc<Mutex<Option<DeviceStats>>>,
}
impl ScheduledDevice {
    pub fn new(executor: Arc<QwenDeviceExecutor>, eos: Vec<u32>, mode: StateMode) -> Self {
        Self {
            executor: executor.clone(),
            states: BTreeMap::new(),
            mode,
            identity: executor.identity().to_owned(),
            eos,
            physical: false,
            gpu_sampling: false,
            counters: Arc::new(Mutex::new(BatchCounters::default())),
            drain: Arc::new(Mutex::new(None)),
        }
    }
    /// Opt-in target-only graph packing; caller must provision the documented 2 GiB budget.
    pub fn with_physical_batching(mut self) -> Result<Self> {
        ensure!(
            self.mode == StateMode::Device && !self.gpu_sampling,
            "physical batching requires device state"
        );
        self.physical = true;
        Ok(self)
    }
    pub fn with_gpu_sampling(mut self) -> Result<Self> {
        ensure!(
            self.mode == StateMode::Device && !self.physical,
            "GPU sampling requires scalar device state"
        );
        self.gpu_sampling = true;
        Ok(self)
    }
    pub fn batch_counters(&self) -> Arc<Mutex<BatchCounters>> {
        self.counters.clone()
    }
    pub fn storage(&self) -> Storage {
        Storage::BackendManaged {
            fingerprint: Sha256::digest(format!(
                "native-qwen-experimental-{:?}-physical-{}-sampling-{}-state-v1",
                self.mode, self.physical, self.gpu_sampling
            ))
            .into(),
            sequence_bytes: 86_835_200 * if self.physical { 3 } else { 1 },
            experimental: true,
        }
    }
}
impl ModelBackend for ScheduledDevice {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: format!(
                "{}:experimental-{:?}-fp32-state-v1",
                self.identity, self.mode
            ),
            width: 0,
            vocabulary_size: 248320,
            max_context_tokens: 512,
            workspace_bytes: 512 * 1024 * 1024,
            max_in_flight: 3,
            eos_tokens: self.eos.clone(),
        }
    }
    fn supports_choices(&self) -> bool {
        true
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            use super::super::batch_transaction::PreparedBatch;
            let prepared = PreparedBatch::prepare(
                batch,
                &mut self.states,
                || match self.mode {
                    StateMode::Snapshot => State::Snapshot(self.executor.reference_empty_state()),
                    StateMode::Device => State::Device(self.executor.empty_state()),
                },
                248320,
            )?;
            if self.gpu_sampling {
                use super::super::batch_transaction::Selection;
                let executor = self.executor.clone();
                let counters = self.counters.clone();
                let completed = prepared
                    .execute_receipts(move |parent, ir, unconstrained| {
                        let executor = executor.clone();
                        let counters = counters.clone();
                        async move {
                            let State::Device(old) = parent else {
                                anyhow::bail!("GPU sampling requires device state")
                            };
                            let result = if unconstrained {
                                let r = executor.execute_sampled(&old, ir).await?;
                                (State::Device(r.state), Selection::Token(r.token))
                            } else {
                                let r = executor.execute(&old, ir).await?;
                                (State::Device(r.state), Selection::Logits(r.logits))
                            };
                            counters
                                .lock()
                                .map_err(|_| anyhow::anyhow!("batch counters poisoned"))?
                                .scalar_calls += 1;
                            Ok(result)
                        }
                    })
                    .await?;
                return Ok(completed.commit());
            }
            let executor = self.executor.clone();
            let physical = self.physical;
            let counters = self.counters.clone();
            let run = move |rows: Vec<(State, super::super::model_ir::ModelStepIr)>| {
                let executor = executor.clone();
                let counters = counters.clone();
                async move {
                    let packed = physical
                        && rows.len() > 1
                        && rows[0].1.kind() == super::super::model_ir::StepKind::Decode
                        && rows[0].1.position().get() > 0;
                    if packed {
                        let parents = rows
                            .iter()
                            .map(|(s, _)| match s {
                                State::Device(s) => Ok(s.clone()),
                                _ => anyhow::bail!("packed snapshot state"),
                            })
                            .collect::<Result<Vec<_>>>()?;
                        let tokens = rows.iter().map(|(_, ir)| ir.ffi_ids()[0]).collect();
                        let result = executor.decode_batch(&parents, tokens).await?;
                        let mut count = counters
                            .lock()
                            .map_err(|_| anyhow::anyhow!("batch counters poisoned"))?;
                        count.packed_calls += 1;
                        count.packed_rows += result.len() as u64;
                        Ok(result
                            .into_iter()
                            .map(|r| (State::Device(r.state), r.logits))
                            .collect())
                    } else {
                        let mut results = Vec::with_capacity(rows.len());
                        for (old, ir) in rows {
                            let result = match old {
                                State::Snapshot(old) => {
                                    let r = executor.reference_execute(&old, ir).await?;
                                    (State::Snapshot(r.state), r.logits)
                                }
                                State::Device(old) => {
                                    let r = executor.execute(&old, ir).await?;
                                    (State::Device(r.state), r.logits)
                                }
                            };
                            counters
                                .lock()
                                .map_err(|_| anyhow::anyhow!("batch counters poisoned"))?
                                .scalar_calls += 1;
                            results.push(result);
                        }
                        Ok(results)
                    }
                }
            };
            let completed = if physical {
                prepared.execute_grouped(run).await?
            } else {
                prepared
                    .execute(move |parent, ir| {
                        let result = run(vec![(parent, ir)]);
                        async move {
                            result
                                .await?
                                .pop()
                                .ok_or_else(|| anyhow::anyhow!("scalar result missing"))
                        }
                    })
                    .await?
            };
            Ok(completed.commit())
        })
    }
    fn release(&mut self, sequence: u64) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.states.remove(&sequence);
            Ok(())
        })
    }
    fn shutdown(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.states.clear();
            let stats = self.executor.stats().await?;
            ensure!(
                stats.residents == 0 && stats.logical_bytes == 0,
                "device scheduler drain residual"
            );
            *self
                .drain
                .lock()
                .map_err(|_| anyhow::anyhow!("drain observer poisoned"))? = Some(stats);
            Ok(())
        })
    }
}

/// Successfully completed native calls, including provisional results later rolled back.
#[derive(Debug, Default, Clone, Copy)]
pub struct BatchCounters {
    pub packed_calls: u64,
    pub packed_rows: u64,
    pub scalar_calls: u64,
}
