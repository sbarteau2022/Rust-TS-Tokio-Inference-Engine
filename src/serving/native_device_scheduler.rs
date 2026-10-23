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
            let executor = self.executor.clone();
            let completed = prepared
                .execute(move |old, ir| {
                    let executor = executor.clone();
                    async move {
                        match old {
                            State::Snapshot(old) => {
                                let result = executor.reference_execute(&old, ir).await?;
                                Ok((State::Snapshot(result.state), result.logits))
                            }
                            State::Device(old) => {
                                let result = executor.execute(&old, ir).await?;
                                Ok((State::Device(result.state), result.logits))
                            }
                        }
                    }
                })
                .await?;
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
