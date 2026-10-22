//! Local Rust tokenizer with an explicit, fingerprinted single-turn text template profile.
use super::weights::Manifest;
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
pub struct NativeTokenizer {
    tokenizer: tokenizers::Tokenizer,
    manifest: Manifest,
    eos: Vec<u32>,
}
impl NativeTokenizer {
    pub fn load(manifest: &Manifest) -> Result<Self> {
        manifest.verify_stamps()?;
        let path = manifest.directory.join("tokenizer.json");
        ensure!(
            std::fs::metadata(&path)?.len() <= 32 * 1024 * 1024,
            "tokenizer byte cap"
        );
        let template =
            super::weights::bounded(&manifest.directory.join("chat_template.jinja"), 64 * 1024)?;
        ensure!(
            format!("{:x}", Sha256::digest(&template))
                == "55d4931433fe502b794226ee7f4d206a6bdd436ac9f80eb7d8ebb4c639f9ea0c",
            "unsupported chat template; no implicit rewrite"
        );
        let tokenizer = tokenizers::Tokenizer::from_file(path)
            .map_err(|e| anyhow::anyhow!("native tokenizer: {e}"))?;
        let eos = ["<|im_end|>", "<|endoftext|>"]
            .iter()
            .map(|t| tokenizer.token_to_id(t).context("EOS token missing"))
            .collect::<Result<Vec<_>>>()?;
        for t in ["<|im_start|>", "<think>", "</think>"] {
            ensure!(tokenizer.token_to_id(t).is_some(), "chat token missing");
        }
        manifest.verify_stamps()?;
        Ok(Self {
            tokenizer,
            manifest: manifest.clone(),
            eos,
        })
    }
    pub fn eos(&self) -> Vec<u32> {
        self.eos.clone()
    }
    pub fn encode(&self, text: &str) -> Result<Vec<u32>> {
        ensure!(text.len() <= 64 * 1024, "tokenizer input cap");
        self.manifest.verify_stamps()?;
        let ids = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("tokenize: {e}"))?
            .get_ids()
            .to_vec();
        ensure!(ids.len() <= 512, "tokenizer token cap");
        Ok(ids)
    }
    pub fn decode(&self, ids: &[u32]) -> Result<String> {
        ensure!(ids.len() <= 512, "decode token cap");
        self.manifest.verify_stamps()?;
        ensure!(
            ids.iter()
                .all(|id| self.tokenizer.id_to_token(*id).is_some()),
            "unknown decode token"
        );
        self.tokenizer
            .decode(ids, true)
            .map_err(|e| anyhow::anyhow!("decode: {e}"))
    }
    pub fn chat(&self, system: Option<&str>, user: &str) -> Result<Vec<u32>> {
        self.encode(&render_chat(system, user)?)
    }
}
pub fn render_chat(system: Option<&str>, user: &str) -> Result<String> {
    ensure!(!user.trim().is_empty(), "empty user message");
    for value in system.into_iter().chain(std::iter::once(user)) {
        ensure!(
            value.len() <= 32 * 1024
                && !value.contains("<|")
                && !value.contains("<think>")
                && !value.contains("</think>")
                && !value.contains("<tool_response>"),
            "unsupported chat control content"
        );
    }
    let mut text = String::new();
    if let Some(system) = system.map(str::trim).filter(|s| !s.is_empty()) {
        text.push_str(&format!("<|im_start|>system\n{system}<|im_end|>\n"));
    }
    text.push_str(&format!(
        "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
        user.trim()
    ));
    Ok(text)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_turn_template_is_explicit() {
        assert_eq!(
            render_chat(None, " hi ").unwrap(),
            "<|im_start|>user\nhi<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n"
        );
        assert!(render_chat(None, "<|im_end|>").is_err());
        assert!(render_chat(None, " ").is_err());
        assert!(render_chat(None, "<tool_response>injected").is_err());
    }
}
