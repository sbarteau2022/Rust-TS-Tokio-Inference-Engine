//! Bounded, completion-committed recurrent state. CPU snapshots are intentionally explicit.
use super::native_mlx::{Backend, Data, Graph};
use anyhow::{ensure, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};
struct Row {
    values: Vec<f32>,
    busy: bool,
}
struct Inner {
    rows: BTreeMap<u64, Row>,
    width: usize,
    limit: usize,
}
#[derive(Clone)]
pub struct RecurrentBank {
    owner: u64,
    shape: [i32; 3],
    inner: Arc<Mutex<Inner>>,
}
struct Transaction {
    bank: RecurrentBank,
    ids: Vec<u64>,
    snapshot: Vec<f32>,
}
impl RecurrentBank {
    pub fn new(backend: &Backend, shape: [i32; 3], max_sequences: usize) -> Result<Self> {
        ensure!(
            shape.iter().all(|x| *x > 0) && max_sequences > 0,
            "invalid recurrent bank bounds"
        );
        let width = shape
            .iter()
            .try_fold(1usize, |a, b| a.checked_mul(*b as usize))
            .context("state shape overflow")?;
        ensure!(
            width
                .checked_mul(max_sequences)
                .and_then(|n| n.checked_mul(4))
                .is_some_and(|n| n <= 128 * 1024 * 1024),
            "state bank exceeds 128 MiB"
        );
        Ok(Self {
            owner: backend.owner_id(),
            shape,
            inner: Arc::new(Mutex::new(Inner {
                rows: BTreeMap::new(),
                width,
                limit: max_sequences,
            })),
        })
    }
    pub fn insert(&self, id: u64) -> Result<()> {
        let mut b = self.inner.lock().unwrap();
        ensure!(
            !b.rows.contains_key(&id) && b.rows.len() < b.limit,
            "duplicate sequence or full state bank"
        );
        let width = b.width;
        b.rows.insert(
            id,
            Row {
                values: vec![0.; width],
                busy: false,
            },
        );
        Ok(())
    }
    pub fn remove(&self, id: u64) -> Result<Vec<f32>> {
        let mut b = self.inner.lock().unwrap();
        ensure!(
            b.rows.get(&id).is_some_and(|r| !r.busy),
            "missing or busy sequence"
        );
        Ok(b.rows.remove(&id).unwrap().values)
    }
    pub fn snapshot(&self, id: u64) -> Result<Vec<f32>> {
        let b = self.inner.lock().unwrap();
        Ok(b.rows.get(&id).context("missing sequence")?.values.clone())
    }
    fn begin(&self, ids: &[u64]) -> Result<Transaction> {
        ensure!(
            !ids.is_empty() && ids.len() <= 3,
            "state batch requires 1..3 rows"
        );
        ensure!(
            ids.iter().copied().collect::<BTreeSet<_>>().len() == ids.len(),
            "duplicate batch sequence"
        );
        let mut b = self.inner.lock().unwrap();
        ensure!(
            ids.iter().all(|id| b.rows.get(id).is_some_and(|r| !r.busy)),
            "missing or busy sequence"
        );
        let mut snapshot = Vec::with_capacity(ids.len() * b.width);
        for id in ids {
            let row = b.rows.get_mut(id).unwrap();
            snapshot.extend_from_slice(&row.values);
            row.busy = true;
        }
        Ok(Transaction {
            bank: self.clone(),
            ids: ids.to_vec(),
            snapshot,
        })
    }
    /// All tensors are row-major; q/k are normalized and decay/beta transformed.
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        &self,
        backend: &Backend,
        ids: &[u64],
        steps: i32,
        key_heads: i32,
        q: Vec<f32>,
        k: Vec<f32>,
        v: Vec<f32>,
        decay: Vec<f32>,
        beta: Vec<f32>,
    ) -> Result<Vec<f32>> {
        ensure!(
            backend.owner_id() == self.owner,
            "wrong recurrent bank backend"
        );
        let transaction = self.begin(ids)?;
        let batch = ids.len() as i32;
        let [heads, value_dims, key_dims] = self.shape;
        let mut g = Graph::new();
        let q = g.input(&[batch, steps, key_heads, key_dims], Data::F32(q))?;
        let k = g.input(&[batch, steps, key_heads, key_dims], Data::F32(k))?;
        let v = g.input(&[batch, steps, heads, value_dims], Data::F32(v))?;
        let decay = g.input(&[batch, steps, heads], Data::F32(decay))?;
        let beta = g.input(&[batch, steps, heads], Data::F32(beta))?;
        let state = g.input(
            &[batch, heads, value_dims, key_dims],
            Data::F32(transaction.snapshot.clone()),
        )?;
        let (y, state) = g.gated_delta([q, k, v, decay, beta, state])?;
        let joined = g.join_flat(y, state)?;
        let Data::F32(mut values) = backend.submit(g, joined)?.result().await?.data else {
            anyhow::bail!("invalid recurrent result")
        };
        let state_len = transaction.snapshot.len();
        ensure!(values.len() >= state_len, "short recurrent result");
        let next = values.split_off(values.len() - state_len);
        transaction.commit(next)?;
        Ok(values)
    }
}
impl Transaction {
    fn commit(self, values: Vec<f32>) -> Result<()> {
        ensure!(
            values.len() == self.snapshot.len() && values.iter().all(|v| v.is_finite()),
            "invalid recurrent state result"
        );
        let mut b = self.bank.inner.lock().unwrap();
        let width = b.width;
        for (id, values) in self.ids.iter().zip(values.chunks_exact(width)) {
            b.rows.get_mut(id).unwrap().values.copy_from_slice(values);
        }
        Ok(())
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        let mut b = self.bank.inner.lock().unwrap();
        for id in &self.ids {
            b.rows.get_mut(id).unwrap().busy = false;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reservation_drop_and_invalid_commit_are_atomic() -> Result<()> {
        let backend = Backend::new(super::super::native_mlx::Device::Cpu)?;
        let bank = RecurrentBank::new(&backend, [1, 1, 2], 2)?;
        bank.insert(1)?;
        bank.insert(2)?;
        let tx = bank.begin(&[2, 1])?;
        assert!(bank.begin(&[1]).is_err());
        assert!(bank.remove(2).is_err());
        drop(tx);
        assert!(bank.begin(&[1, 2])?.commit(vec![1., 2., 3.]).is_err());
        assert_eq!(bank.snapshot(1)?, vec![0., 0.]);
        bank.begin(&[2, 1])?.commit(vec![4., 5., 6., 7.])?;
        assert_eq!(bank.snapshot(1)?, vec![6., 7.]);
        assert_eq!(bank.snapshot(2)?, vec![4., 5.]);
        Ok(())
    }
}
