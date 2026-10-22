use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub max_queued: usize,
    pub max_sequences: usize,
    pub max_batch_tokens: usize,
    pub prefill_chunk_tokens: usize,
    pub max_context_tokens: usize,
    pub max_new_tokens: usize,
    pub model_memory_bytes: usize,
    pub max_in_flight: usize,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1,
            "unsupported serving configuration version"
        );
        ensure!(
            (1..=32).contains(&self.max_queued) && (1..=32).contains(&self.max_sequences),
            "queue/sequence limit outside 1..32"
        );
        ensure!(
            (1..=4096).contains(&self.max_batch_tokens)
                && (1..=self.max_batch_tokens).contains(&self.prefill_chunk_tokens),
            "invalid batch/chunk token limits"
        );
        ensure!(
            (1..=4096).contains(&self.max_context_tokens)
                && (1..=4096).contains(&self.max_new_tokens),
            "context/generation limits outside 1..4096"
        );
        ensure!(
            (1..=8usize * 1024 * 1024 * 1024).contains(&self.model_memory_bytes),
            "experimental model KV/workspace budget outside 1..8 GiB"
        );
        ensure!(
            [2, 3].contains(&self.max_in_flight),
            "in-flight limit must be 2 or 3"
        );
        Ok(())
    }
    pub fn fingerprint(&self) -> Result<String> {
        self.validate()?;
        Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(self)?)))
    }
}
