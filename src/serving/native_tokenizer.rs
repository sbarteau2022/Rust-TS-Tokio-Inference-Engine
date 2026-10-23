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
        let config = super::weights::strict(&super::weights::bounded(
            &manifest.directory.join("config.json"),
            1024 * 1024,
        )?)?;
        let generation_path = manifest.directory.join("generation_config.json");
        let generation = if generation_path.try_exists()? {
            Some(super::weights::strict(&super::weights::bounded(
                &generation_path,
                64 * 1024,
            )?)?)
        } else {
            None
        };
        let eos = configured_eos(&config, generation.as_ref(), &tokenizer)?;
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
    pub fn text_stream(&self) -> Result<super::native_text_stream::NativeTextStream<'_>> {
        self.manifest.verify_stamps()?;
        Ok(
            super::native_text_stream::NativeTextStream::new(&self.tokenizer)
                .with_manifest(&self.manifest),
        )
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
    pub fn encode_choice(&self, text: &str) -> Result<Vec<u32>> {
        ensure!(
            !text.is_empty()
                && text.len() <= 1024
                && !text.contains("<|")
                && !text.contains("<think>")
                && !text.contains("</think>"),
            "invalid output choice text"
        );
        let ids = self.encode(text)?;
        ensure!(
            self.decode(&ids)? == text,
            "output choice must round-trip exactly"
        );
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
// The checkpoint's top-level generation policy wins over nested model metadata.
// A generation_config override is explicit; malformed overrides fail closed.
fn configured_eos(
    config: &serde_json::Value,
    generation: Option<&serde_json::Value>,
    tokenizer: &tokenizers::Tokenizer,
) -> Result<Vec<u32>> {
    ensure!(config.is_object(), "model configuration object required");
    if let Some(generation) = generation {
        ensure!(
            generation.is_object(),
            "generation configuration object required"
        );
    }
    let value = generation
        .and_then(|v| v.get("eos_token_id"))
        .filter(|v| !v.is_null())
        .or_else(|| config.get("eos_token_id"))
        .context("checkpoint EOS policy missing")?;
    let values = value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value.clone()]);
    ensure!((1..=16).contains(&values.len()), "EOS count 1..16");
    let mut eos = values
        .iter()
        .map(|v| {
            let id = u32::try_from(v.as_u64().context("EOS must be an unsigned integer")?)?;
            ensure!(tokenizer.id_to_token(id).is_some(), "unknown EOS token");
            Ok(id)
        })
        .collect::<Result<Vec<_>>>()?;
    eos.sort();
    eos.dedup();
    Ok(eos)
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
    fn checkpoint_eos_precedence_and_malformed_policy() -> Result<()> {
        use serde_json::json;
        let model = tokenizers::models::wordlevel::WordLevel::builder()
            .vocab(
                [("end".to_string(), 0), ("other".to_string(), 1)]
                    .into_iter()
                    .collect(),
            )
            .unk_token("end".into())
            .build()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let tokenizer = tokenizers::Tokenizer::new(model);
        let config = json!({"eos_token_id": 1, "text_config": {"eos_token_id": 0}});
        assert_eq!(configured_eos(&config, None, &tokenizer)?, vec![1]);
        assert_eq!(
            configured_eos(&config, Some(&json!({"eos_token_id": [1,0,1]})), &tokenizer)?,
            vec![0, 1]
        );
        assert_eq!(
            configured_eos(&config, Some(&json!({"eos_token_id": null})), &tokenizer)?,
            vec![1]
        );
        for value in [
            json!([]),
            json!(vec![0; 17]),
            json!(-1),
            json!(4294967296u64),
            json!(2),
            json!(1.5),
            json!("1"),
            json!(true),
            json!({}),
            json!([0, null]),
        ] {
            assert!(configured_eos(&json!({"eos_token_id":value}), None, &tokenizer).is_err());
            assert!(
                configured_eos(&config, Some(&json!({"eos_token_id":value})), &tokenizer).is_err()
            );
        }
        assert!(configured_eos(&json!({}), None, &tokenizer).is_err());
        assert!(configured_eos(&config, Some(&json!([])), &tokenizer).is_err());
        Ok(())
    }
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
