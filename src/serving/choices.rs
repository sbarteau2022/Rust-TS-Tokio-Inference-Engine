//! Explicit canonical token-path output language, independent of expected answers.
use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone)]
pub struct Choices {
    paths: Vec<Vec<u32>>,
    fingerprint: String,
}
impl Choices {
    pub fn new(mut paths: Vec<Vec<u32>>, vocab: u32, eos: &[u32], budget: usize) -> Result<Self> {
        ensure!(
            !eos.is_empty() && eos.iter().all(|t| *t < vocab),
            "choices require valid EOS"
        );
        ensure!((1..=16).contains(&paths.len()), "choice count 1..16");
        ensure!(
            paths.iter().all(|p| !p.is_empty()
                && p.len() <= 64
                && p.len() < budget
                && p.iter().all(|t| *t < vocab && !eos.contains(t))),
            "choice token/budget limits"
        );
        paths.sort();
        ensure!(!paths.windows(2).any(|p| p[0] == p[1]), "duplicate choice");
        let mut hash = Sha256::new();
        hash.update(b"canonical-token-choices-v1");
        hash.update(vocab.to_le_bytes());
        let mut terminators = eos.to_vec();
        terminators.sort();
        terminators.dedup();
        hash.update((terminators.len() as u32).to_le_bytes());
        for token in terminators {
            hash.update(token.to_le_bytes());
        }
        hash.update((paths.len() as u32).to_le_bytes());
        for p in &paths {
            hash.update((p.len() as u32).to_le_bytes());
            for t in p {
                hash.update(t.to_le_bytes());
            }
        }
        Ok(Self {
            paths,
            fingerprint: format!("{:x}", hash.finalize()),
        })
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
    pub fn allowed(&self, prefix: &[u32], eos: &[u32]) -> Vec<u32> {
        let mut next = std::collections::BTreeSet::new();
        for p in &self.paths {
            if p.starts_with(prefix) {
                if p.len() == prefix.len() {
                    next.extend(eos.iter().copied());
                } else {
                    next.insert(p[prefix.len()]);
                }
            }
        }
        next.into_iter().collect()
    }
}
/// Stable lowest-ID tie breaking, identical to unconstrained greedy selection.
pub fn select(logits: &[f32], allowed: Option<&[u32]>) -> Result<u32> {
    ensure!(
        !logits.is_empty() && logits.iter().all(|v| v.is_finite()),
        "invalid logits"
    );
    fn best(logits: &[f32], ids: impl Iterator<Item = u32>) -> Result<u32> {
        let mut winner: Option<u32> = None;
        for id in ids {
            ensure!((id as usize) < logits.len(), "invalid legal token");
            if winner.is_none_or(|old| {
                logits[id as usize].total_cmp(&logits[old as usize]).is_gt()
                    || (logits[id as usize].total_cmp(&logits[old as usize]).is_eq() && id < old)
            }) {
                winner = Some(id);
            }
        }
        winner.ok_or_else(|| anyhow::anyhow!("empty legal token set"))
    }
    match allowed {
        Some(ids) => best(logits, ids.iter().copied()),
        None => best(logits, 0..logits.len() as u32),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_and_terminal_prefixes_and_stable_selection() -> Result<()> {
        let c = Choices::new(vec![vec![2], vec![2, 3], vec![4, 5]], 10, &[9], 3)?;
        assert_eq!(c.allowed(&[], &[9]), vec![2, 4]);
        assert_eq!(c.allowed(&[2], &[9]), vec![3, 9]);
        assert_eq!(c.allowed(&[4, 5], &[9]), vec![9]);
        assert!(c.allowed(&[7], &[9]).is_empty());
        assert_eq!(select(&[1., 5., 5.], Some(&[2, 1]))?, 1);
        assert_eq!(select(&[100., 2., 3.], Some(&[1, 2]))?, 2);
        let reordered = Choices::new(vec![vec![4, 5], vec![2, 3], vec![2]], 10, &[9], 3)?;
        assert_eq!(c.fingerprint(), reordered.fingerprint());
        Ok(())
    }
    #[test]
    fn malformed_contracts_fail_closed() {
        for paths in [
            vec![],
            vec![vec![]],
            vec![vec![9]],
            vec![vec![10]],
            vec![vec![1], vec![1]],
            vec![vec![1]; 17],
            vec![vec![1; 65]],
        ] {
            assert!(Choices::new(paths, 10, &[9], 100).is_err());
        }
        assert!(Choices::new(vec![vec![1, 2]], 10, &[9], 2).is_err());
        assert!(Choices::new(vec![vec![1]], 10, &[], 3).is_err());
        assert!(select(&[f32::NAN], None).is_err());
        assert!(select(&[0.], Some(&[])).is_err());
        assert!(select(&[0.], Some(&[1])).is_err());
    }
}
