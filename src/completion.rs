//! A committed command owns its permit and retained resources independently of its waiter.
use crate::gpu::Timing;
use anyhow::{ensure, Result};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use tokio::sync::{oneshot, OwnedSemaphorePermit, Semaphore};
#[derive(Default)]
struct Counters {
    active: AtomicUsize,
    peak: AtomicUsize,
    acquired: AtomicUsize,
    released: AtomicUsize,
    submitted: AtomicUsize,
    completed: AtomicUsize,
    outstanding: AtomicUsize,
    peak_outstanding: AtomicUsize,
    errors: AtomicUsize,
}
#[derive(Clone)]
pub struct FlightGate {
    semaphore: Arc<Semaphore>,
    counters: Arc<Counters>,
    limit: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct FlightStats {
    pub limit: usize,
    pub active: usize,
    pub peak: usize,
    pub acquired: usize,
    pub released: usize,
    pub submitted: usize,
    pub completed: usize,
    pub outstanding: usize,
    pub peak_outstanding: usize,
    pub completion_errors: usize,
}
pub struct FlightPermit {
    _retained: Vec<Box<dyn Send>>,
    _permit: OwnedSemaphorePermit,
    gate: FlightGate,
}
impl FlightPermit {
    pub(crate) fn retain<T: Send + 'static>(&mut self, value: T) {
        self._retained.push(Box::new(value));
    }
}
impl Drop for FlightPermit {
    fn drop(&mut self) {
        self.gate.counters.active.fetch_sub(1, Ordering::SeqCst);
        self.gate.counters.released.fetch_add(1, Ordering::SeqCst);
    }
}
impl FlightGate {
    pub fn new(limit: usize) -> Result<Self> {
        ensure!((2..=3).contains(&limit), "in-flight limit must be 2 or 3");
        Ok(Self {
            semaphore: Arc::new(Semaphore::new(limit)),
            counters: Arc::default(),
            limit,
        })
    }
    fn wrap(&self, permit: OwnedSemaphorePermit) -> FlightPermit {
        let active = self.counters.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.counters.peak.fetch_max(active, Ordering::SeqCst);
        self.counters.acquired.fetch_add(1, Ordering::SeqCst);
        FlightPermit {
            _retained: Vec::new(),
            _permit: permit,
            gate: self.clone(),
        }
    }
    pub async fn acquire(&self) -> Result<FlightPermit> {
        Ok(self.wrap(self.semaphore.clone().acquire_owned().await?))
    }
    pub fn try_acquire(&self) -> Result<FlightPermit> {
        Ok(self.wrap(self.semaphore.clone().try_acquire_owned()?))
    }
    pub(crate) async fn drain(&self) {
        if let Ok(permits) = self
            .semaphore
            .clone()
            .acquire_many_owned(self.limit as u32)
            .await
        {
            drop(permits);
        }
    }
    pub fn stats(&self) -> FlightStats {
        let c = &self.counters;
        let read = |v: &AtomicUsize| v.load(Ordering::SeqCst);
        FlightStats {
            limit: self.limit,
            active: read(&c.active),
            peak: read(&c.peak),
            acquired: read(&c.acquired),
            released: read(&c.released),
            submitted: read(&c.submitted),
            completed: read(&c.completed),
            outstanding: read(&c.outstanding),
            peak_outstanding: read(&c.peak_outstanding),
            completion_errors: read(&c.errors),
        }
    }
    pub(crate) fn submitted(&self) {
        self.counters.submitted.fetch_add(1, Ordering::SeqCst);
        let active = self.counters.outstanding.fetch_add(1, Ordering::SeqCst) + 1;
        self.counters
            .peak_outstanding
            .fetch_max(active, Ordering::SeqCst);
    }
}
impl FlightPermit {
    pub(crate) fn belongs_to(&self, gate: &FlightGate) -> bool {
        Arc::ptr_eq(&self.gate.semaphore, &gate.semaphore)
    }
}
pub type CompletionResult = std::result::Result<Timing, String>;
struct State {
    permit: FlightPermit,
    buffers: Vec<metal::Buffer>,
    async_reply: oneshot::Sender<CompletionResult>,
    sync_reply: mpsc::SyncSender<CompletionResult>,
}
pub(crate) struct Bridge {
    state: Mutex<Option<State>>,
}
pub struct Submission {
    async_reply: oneshot::Receiver<CompletionResult>,
    sync_reply: mpsc::Receiver<CompletionResult>,
}
impl Submission {
    pub async fn wait(mut self) -> Result<Timing> {
        self.wait_ref().await
    }
    pub(crate) async fn wait_ref(&mut self) -> Result<Timing> {
        (&mut self.async_reply).await?.map_err(anyhow::Error::msg)
    }
    pub fn wait_blocking(self) -> Result<Timing> {
        self.sync_reply.recv()?.map_err(anyhow::Error::msg)
    }
}
impl Bridge {
    pub(crate) fn new(
        permit: FlightPermit,
        buffers: Vec<metal::Buffer>,
    ) -> (Arc<Self>, Submission) {
        let (async_reply, async_rx) = oneshot::channel();
        let (sync_reply, sync_rx) = mpsc::sync_channel(1);
        (
            Arc::new(Self {
                state: Mutex::new(Some(State {
                    permit,
                    buffers,
                    async_reply,
                    sync_reply,
                })),
            }),
            Submission {
                async_reply: async_rx,
                sync_reply: sync_rx,
            },
        )
    }
    // Consumed once even if invoked twice. No unwrap/panic crosses the Objective-C ABI.
    pub(crate) fn complete(&self, result: CompletionResult) {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(state) = state {
            let counters = &state.permit.gate.counters;
            counters.completed.fetch_add(1, Ordering::SeqCst);
            counters.outstanding.fetch_sub(1, Ordering::SeqCst);
            if result.is_err() {
                counters.errors.fetch_add(1, Ordering::SeqCst);
            }
            drop(state.buffers);
            drop(state.permit);
            let _ = state.sync_reply.try_send(result.clone());
            let _ = state.async_reply.send(result);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn failed_and_duplicate_completion_release_exactly_once() {
        let gate = FlightGate::new(2).unwrap();
        let permit = gate.acquire().await.unwrap();
        let (bridge, submission) = Bridge::new(permit, vec![]);
        gate.submitted();
        bridge.complete(Err("injected completion error".into()));
        bridge.complete(Err("duplicate".into()));
        assert!(submission.wait().await.is_err());
        let s = gate.stats();
        assert_eq!(s.active, 0);
        assert_eq!(s.released, 1);
        assert_eq!(s.completed, 1);
        assert_eq!(s.completion_errors, 1);
        assert_eq!(s.outstanding, 0);
    }
    #[tokio::test]
    async fn dropped_waiter_keeps_permit_until_callback() {
        let gate = FlightGate::new(2).unwrap();
        let (bridge, submission) = Bridge::new(gate.acquire().await.unwrap(), vec![]);
        gate.submitted();
        drop(submission);
        assert_eq!(gate.stats().active, 1);
        bridge.complete(Ok(Timing {
            dispatch_wall_ms: 1.,
            gpu_ms: 0.5,
        }));
        assert_eq!(gate.stats().active, 0);
        assert_eq!(gate.stats().completed, 1);
    }
}
