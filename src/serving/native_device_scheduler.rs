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
impl State {
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
            drain: Arc::new(Mutex::new(None)),
        }
    }
    pub fn storage(&self) -> Storage {
        Storage::BackendManaged {
            fingerprint: Sha256::digest(format!(
                "native-qwen-experimental-{:?}-state-v1",
                self.mode
            ))
            .into(),
            sequence_bytes: 86_835_200,
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
            ensure!(
                !batch.is_empty() && batch.len() <= 3,
                "device scheduler row cap"
            );
            let mut seen = BTreeSet::new();
            let mut prepared = Vec::new();
            let new_count = batch
                .iter()
                .filter(|i| !self.states.contains_key(&i.sequence))
                .count();
            ensure!(
                self.states.len() + new_count <= 3,
                "device scheduler sequence cap"
            );
            for item in batch {
                ensure!(
                    seen.insert(item.sequence) && item.history.is_empty(),
                    "duplicate row/external history"
                );
                ensure!(
                    self.states.get(&item.sequence).map_or(0, State::position) == item.position,
                    "device scheduler position"
                );
                if let Some(mask) = &item.allowed_tokens {
                    ensure!(
                        !mask.is_empty() && mask.len() <= 32 && mask.iter().all(|t| *t < 248320),
                        "device scheduler token mask"
                    );
                }
                let ir = ModelStepIr::compile(item.kind, item.position, item.tokens, 248320)?;
                ensure!(
                    ir.len() <= 8 && item.position + ir.len() <= 512,
                    "device scheduler geometry"
                );
                prepared.push((item.sequence, item.prompt_complete, item.allowed_tokens, ir));
            }
            let mut next = Vec::new();
            let mut outputs = Vec::new();
            for (sequence, complete, allowed, ir) in prepared {
                let old = self
                    .states
                    .get(&sequence)
                    .cloned()
                    .unwrap_or_else(|| match self.mode {
                        StateMode::Snapshot => {
                            State::Snapshot(self.executor.reference_empty_state())
                        }
                        StateMode::Device => State::Device(self.executor.empty_state()),
                    });
                let (state, logits) = match old {
                    State::Snapshot(old) => {
                        let result = self.executor.reference_execute(&old, ir).await?;
                        (State::Snapshot(result.state), result.logits)
                    }
                    State::Device(old) => {
                        let result = self.executor.execute(&old, ir).await?;
                        (State::Device(result.state), result.logits)
                    }
                };
                let token = if complete {
                    Some(super::super::choices::select(&logits, allowed.as_deref())?)
                } else {
                    None
                };
                outputs.push(StepOutput {
                    sequence,
                    kv: Vec::new(),
                    next_token: token,
                    state_bytes: state.bytes(),
                });
                next.push((sequence, state));
            }
            for (id, state) in next {
                self.states.insert(id, state);
            }
            Ok(outputs)
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
