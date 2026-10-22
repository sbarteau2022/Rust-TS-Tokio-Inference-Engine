//! Atomic multidimensional admission. Leases belong to completed native work, not receipts.
use anyhow::{ensure, Result};
use std::sync::{Arc, Mutex};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Footprint {
    pub host_bytes: usize,
    pub tensor_bytes: usize,
    pub graph_nodes: usize,
}
#[derive(Clone, Debug)]
pub struct Ledger(Arc<Inner>);
#[derive(Debug)]
struct Inner {
    limits: Footprint,
    state: Mutex<State>,
}
#[derive(Debug, Default)]
struct State {
    used: Footprint,
    peak: Footprint,
    leases: usize,
}
#[derive(Debug)]
pub struct Lease {
    inner: Arc<Inner>,
    cost: Footprint,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub used: Footprint,
    pub peak: Footprint,
    pub leases: usize,
}
impl Ledger {
    pub fn new(limits: Footprint) -> Result<Self> {
        ensure!(
            limits.host_bytes > 0 && limits.tensor_bytes > 0 && limits.graph_nodes > 0,
            "ledger limits must be positive"
        );
        Ok(Self(Arc::new(Inner {
            limits,
            state: Mutex::new(State::default()),
        })))
    }
    pub fn reserve(&self, cost: Footprint) -> Result<Lease> {
        let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        let add = |used: usize, extra: usize, cap: usize| {
            used.checked_add(extra).filter(|sum| *sum <= cap)
        };
        let host = add(
            state.used.host_bytes,
            cost.host_bytes,
            self.0.limits.host_bytes,
        );
        let tensors = add(
            state.used.tensor_bytes,
            cost.tensor_bytes,
            self.0.limits.tensor_bytes,
        );
        let nodes = add(
            state.used.graph_nodes,
            cost.graph_nodes,
            self.0.limits.graph_nodes,
        );
        ensure!(
            host.is_some() && tensors.is_some() && nodes.is_some(),
            "joint completion budget exhausted"
        );
        state.used = Footprint {
            host_bytes: host.unwrap(),
            tensor_bytes: tensors.unwrap(),
            graph_nodes: nodes.unwrap(),
        };
        state.peak.host_bytes = state.peak.host_bytes.max(state.used.host_bytes);
        state.peak.tensor_bytes = state.peak.tensor_bytes.max(state.used.tensor_bytes);
        state.peak.graph_nodes = state.peak.graph_nodes.max(state.used.graph_nodes);
        state.leases += 1;
        Ok(Lease {
            inner: self.0.clone(),
            cost,
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        let state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        Snapshot {
            used: state.used,
            peak: state.peak,
            leases: state.leases,
        }
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state.used.host_bytes -= self.cost.host_bytes;
        state.used.tensor_bytes -= self.cost.tensor_bytes;
        state.used.graph_nodes -= self.cost.graph_nodes;
        state.leases -= 1;
    }
}
