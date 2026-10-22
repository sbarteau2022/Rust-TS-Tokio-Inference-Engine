//! Experimental model-facing control plane. No implicit model or production qualification.
pub mod completion_ledger;
pub mod config;
pub mod kv;
pub mod scheduler;

use anyhow::{ensure, Result};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Debug)]
pub struct ByteBudget(Arc<BudgetInner>);
#[derive(Debug)]
struct BudgetInner {
    limit: usize,
    used: AtomicUsize,
    peak: AtomicUsize,
}
#[derive(Debug)]
pub struct Charge {
    budget: ByteBudget,
    bytes: usize,
}
impl ByteBudget {
    pub fn new(limit: usize) -> Result<Self> {
        ensure!(limit > 0, "zero memory budget");
        Ok(Self(Arc::new(BudgetInner {
            limit,
            used: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        })))
    }
    pub fn reserve(&self, bytes: usize) -> Result<Charge> {
        let mut old = self.used();
        loop {
            let next = old
                .checked_add(bytes)
                .filter(|v| *v <= self.0.limit)
                .ok_or_else(|| anyhow::anyhow!("model memory budget exhausted"))?;
            match self
                .0
                .used
                .compare_exchange_weak(old, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Ordering::Relaxed);
                    return Ok(Charge {
                        budget: self.clone(),
                        bytes,
                    });
                }
                Err(actual) => old = actual,
            }
        }
    }
    pub fn used(&self) -> usize {
        self.0.used.load(Ordering::Acquire)
    }
    pub fn peak(&self) -> usize {
        self.0.peak.load(Ordering::Relaxed)
    }
    pub fn limit(&self) -> usize {
        self.0.limit
    }
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.budget.0.used.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

#[cfg(feature = "native-mlx")]
pub mod native_mlx;
#[cfg(feature = "native-mlx")]
pub mod native_qwen;
#[cfg(feature = "native-mlx")]
pub mod native_state;
#[cfg(feature = "native-mlx")]
pub mod native_tokenizer;
#[cfg(feature = "legacy-python")]
pub mod rapid;
#[cfg(feature = "native-mlx")]
pub mod weights;
