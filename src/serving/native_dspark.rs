//! Strict Koopah drafter ingestion. Artifact validity does not qualify execution.
use super::weights::strict;
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub const TARGET_TAPS: [u64; 8] = [1, 6, 11, 16, 22, 27, 32, 37];
const MAX_HEADER: u64 = 1 << 20;
const MAX_CONFIG: u64 = 1 << 16;
/// Content-bound identity for ingestion, deliberately distinct from a proposer ID.
#[derive(Debug)]
pub struct DsparkArtifact {
    pub weights_path: PathBuf,
    pub config_path: PathBuf,
    pub weights_sha256: String,
    pub config_sha256: String,
    pub identity: String,
    pub payload_bytes: u64,
}
impl DsparkArtifact {
    /// Follows the HF snapshot's ordinary blob symlinks, then hashes full contents.
    pub fn read(directory: &Path) -> Result<Self> {
        let config_path = directory.join("config.json").canonicalize()?;
        let weights_path = directory.join("model.safetensors").canonicalize()?;
        let mut config_file = File::open(&config_path)?;
        let config_size = config_file.metadata()?.len();
        ensure!(
            config_file.metadata()?.is_file() && config_size <= MAX_CONFIG,
            "DSpark config bounds"
        );
        let mut config_bytes = Vec::new();
        (&mut config_file)
            .take(MAX_CONFIG + 1)
            .read_to_end(&mut config_bytes)?;
        ensure!(
            config_bytes.len() as u64 == config_size,
            "DSpark config changed during ingestion"
        );
        validate_config(&strict(&config_bytes)?)?;
        let mut weights_file = File::open(&weights_path)?;
        let before = weights_file.metadata()?;
        ensure!(before.is_file(), "DSpark weights must be a regular file");
        let mut prefix = [0; 8];
        weights_file.read_exact(&mut prefix)?;
        let header_len = u64::from_le_bytes(prefix);
        ensure!(
            (2..=MAX_HEADER).contains(&header_len),
            "DSpark header bounds"
        );
        let payload_bytes = before
            .len()
            .checked_sub(8 + header_len)
            .context("DSpark truncated header")?;
        let mut header_bytes = vec![0; header_len as usize];
        weights_file.read_exact(&mut header_bytes)?;
        validate_header(&strict(&header_bytes)?, payload_bytes)?;
        let mut digest = Sha256::new();
        digest.update(prefix);
        digest.update(&header_bytes);
        let mut buffer = vec![0; 1 << 20];
        let mut read_bytes = 0u64;
        loop {
            let n = weights_file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            digest.update(&buffer[..n]);
            read_bytes = read_bytes
                .checked_add(n as u64)
                .context("DSpark payload overflow")?;
            ensure!(
                read_bytes <= payload_bytes,
                "DSpark weights grew during ingestion"
            );
        }
        let after = weights_file.metadata()?;
        ensure!(
            read_bytes == payload_bytes
                && before.len() == after.len()
                && before.modified()? == after.modified()?,
            "DSpark weights changed during ingestion"
        );
        // Re-read the small config to detect changes while hashing the large payload.
        ensure!(
            std::fs::read(&config_path)? == config_bytes,
            "DSpark config changed during hashing"
        );
        let weights_sha256 = format!("{:x}", digest.finalize());
        let config_sha256 = format!("{:x}", Sha256::digest(&config_bytes));
        let identity = format!(
            "{:x}",
            Sha256::digest(format!(
                "koopah-dspark-bf16-v1\0{config_sha256}\0{weights_sha256}"
            ))
        );
        Ok(Self {
            weights_path,
            config_path,
            weights_sha256,
            config_sha256,
            identity,
            payload_bytes,
        })
    }
}
fn validate_config(c: &Value) -> Result<()> {
    ensure!(c.is_object(), "DSpark config object");
    let expected = serde_json::json!({
        "architectures":["Qwen3DSparkModel"], "model_type":"qwen3", "hidden_size":2048,
        "intermediate_size":6144, "num_hidden_layers":6, "num_target_layers":40,
        "num_attention_heads":32, "num_key_value_heads":8, "head_dim":128,
        "vocab_size":248320, "block_size":8, "markov_rank":256,
        "markov_head_type":"vanilla", "enable_confidence_head":true,
        "confidence_head_with_markov":true, "tie_word_embeddings":false,
        "attention_bias":false, "hidden_act":"silu", "dtype":"bfloat16",
        "rms_norm_eps":1e-6, "mask_token_id":248077,
        "target_layer_ids":TARGET_TAPS, "use_sliding_window":false,
        "layer_types":["full_attention","full_attention","full_attention","full_attention","full_attention","full_attention"],
        "max_position_embeddings":262144, "use_cache":true, "sliding_window":null,
        "rope_parameters":{"rope_theta":10000000.0,"rope_type":"default"},
        "dflash_config":{"target_layer_ids":TARGET_TAPS,"mask_token_id":248077}
    });
    for (name, value) in expected.as_object().unwrap() {
        ensure!(c.get(name) == Some(value), "DSpark config mismatch: {name}");
    }
    // Unknown descriptive metadata is allowed; alternate execution semantics aren't.
    let options = serde_json::json!({"is_causal":false,"dflash_query_causal":false,
        "attention_sink_bias":false,"dspark_bonus_anchor":false,
        "rope_is_neox_style":true,"final_logit_softcapping":null,
        "block_size_policy":"adaptive","runtime_block_size":8,
        "proposal_length":8,"draft_window_size":null,"rope_theta":10000000.0});
    for (name, expected) in options.as_object().unwrap() {
        if let Some(value) = c.get(name) {
            ensure!(
                value == expected,
                "DSpark unsupported execution option: {name}"
            );
        }
    }
    for (name, expected) in [("sample_from_anchor", true), ("attention_dropout", false)] {
        if let Some(value) = c.get(name) {
            let valid = if name == "attention_dropout" {
                value.as_f64() == Some(0.0)
            } else {
                value.as_bool() == Some(expected)
            };
            ensure!(valid, "DSpark unsupported execution option: {name}");
        }
    }
    Ok(())
}
fn contracts() -> BTreeMap<String, Vec<u64>> {
    let mut result = BTreeMap::new();
    for (name, shape) in [
        ("embed_tokens.weight", vec![248320, 2048]),
        ("lm_head.weight", vec![248320, 2048]),
        ("fc.weight", vec![2048, 16384]),
        ("hidden_norm.weight", vec![2048]),
        ("norm.weight", vec![2048]),
        ("markov_head.markov_w1.weight", vec![248320, 256]),
        ("markov_head.markov_w2.weight", vec![248320, 256]),
        ("confidence_head.proj.weight", vec![1, 2304]),
        ("confidence_head.proj.bias", vec![1]),
    ] {
        result.insert(name.into(), shape);
    }
    for layer in 0..6 {
        for (name, shape) in [
            ("input_layernorm.weight", vec![2048]),
            ("post_attention_layernorm.weight", vec![2048]),
            ("self_attn.q_norm.weight", vec![128]),
            ("self_attn.k_norm.weight", vec![128]),
            ("self_attn.q_proj.weight", vec![4096, 2048]),
            ("self_attn.k_proj.weight", vec![1024, 2048]),
            ("self_attn.v_proj.weight", vec![1024, 2048]),
            ("self_attn.o_proj.weight", vec![2048, 4096]),
            ("mlp.gate_proj.weight", vec![6144, 2048]),
            ("mlp.up_proj.weight", vec![6144, 2048]),
            ("mlp.down_proj.weight", vec![2048, 6144]),
        ] {
            result.insert(format!("layers.{layer}.{name}"), shape);
        }
    }
    result
}
fn validate_header(header: &Value, payload_bytes: u64) -> Result<()> {
    let header = header.as_object().context("DSpark header object")?;
    let expected = contracts();
    ensure!(
        header
            .keys()
            .filter(|s| s.as_str() != "__metadata__")
            .count()
            == expected.len(),
        "DSpark tensor count"
    );
    let mut ranges = Vec::new();
    for (name, shape) in expected {
        let tensor = header
            .get(&name)
            .with_context(|| format!("DSpark missing {name}"))?;
        ensure!(
            tensor["dtype"] == "BF16" && tensor["shape"] == serde_json::to_value(&shape)?,
            "DSpark tensor contract: {name}"
        );
        let offsets: Vec<u64> = serde_json::from_value(tensor["data_offsets"].clone())?;
        let bytes = shape
            .iter()
            .try_fold(2u64, |n, d| n.checked_mul(*d))
            .context("DSpark tensor overflow")?;
        ensure!(
            offsets.len() == 2
                && offsets[1].checked_sub(offsets[0]) == Some(bytes)
                && offsets[1] <= payload_bytes,
            "DSpark tensor range: {name}"
        );
        ranges.push((offsets[0], offsets[1]));
    }
    ranges.sort_unstable();
    let mut end = 0;
    for (start, next) in ranges {
        ensure!(start == end, "DSpark payload gap or overlap");
        end = next;
    }
    ensure!(end == payload_bytes, "DSpark trailing payload");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> (Value, u64) {
        let mut map = serde_json::Map::new();
        let mut end = 0;
        for (name, shape) in contracts() {
            let start = end;
            end += shape.iter().product::<u64>() * 2;
            map.insert(
                name,
                serde_json::json!({"dtype":"BF16","shape":shape,"data_offsets":[start,end]}),
            );
        }
        (Value::Object(map), end)
    }
    #[test]
    fn exact_geometry_and_payload() {
        let (h, n) = header();
        assert_eq!(h.as_object().unwrap().len(), 75);
        assert_eq!(n, 3_060_334_082);
        validate_header(&h, n).unwrap();
        assert!(validate_header(&h, n + 1).is_err());
        assert!(validate_header(&h, n - 1).is_err());
    }
    #[test]
    fn rejects_corrupt_geometry_and_ranges() {
        let (h, n) = header();
        for field in ["dtype", "shape", "data_offsets"] {
            let mut bad = h.clone();
            bad["fc.weight"][field] = match field {
                "dtype" => serde_json::json!("F32"),
                "shape" => serde_json::json!([2048, 4096]),
                _ => serde_json::json!([0, u64::MAX]),
            };
            assert!(validate_header(&bad, n).is_err());
        }
        let mut bad = h.clone();
        bad.as_object_mut().unwrap().remove("lm_head.weight");
        assert!(validate_header(&bad, n).is_err());
        let mut bad = h.clone();
        bad.as_object_mut()
            .unwrap()
            .insert("extra.weight".into(), serde_json::json!({}));
        assert!(validate_header(&bad, n).is_err());
        let mut bad = h.clone();
        let a = bad["fc.weight"]["data_offsets"].clone();
        bad["fc.weight"]["data_offsets"] = bad["embed_tokens.weight"]["data_offsets"].clone();
        bad["embed_tokens.weight"]["data_offsets"] = a;
        assert!(validate_header(&bad, n).is_err());
    }
    #[test]
    fn config_pins_backbone_heads_taps_and_anchor_semantics() {
        let c = strict(include_bytes!("../../fixtures/koopah-dspark-config.json")).unwrap();
        validate_config(&c).unwrap();
        for (field, value) in [
            ("num_hidden_layers", serde_json::json!(5)),
            (
                "target_layer_ids",
                serde_json::json!([0, 6, 11, 16, 22, 27, 32, 37]),
            ),
            ("confidence_head_with_markov", serde_json::json!(false)),
            ("tie_word_embeddings", serde_json::json!(true)),
            ("sample_from_anchor", serde_json::json!(false)),
            ("attention_dropout", serde_json::json!(0.1)),
        ] {
            let mut bad = c.clone();
            bad[field] = value;
            assert!(validate_config(&bad).is_err(), "{field}");
        }
        let mut explicit = c;
        explicit["sample_from_anchor"] = serde_json::json!(true);
        validate_config(&explicit).unwrap();
    }
    #[test]
    fn rejects_duplicate_metadata_keys() {
        assert!(strict(br#"{"dtype":"BF16","dtype":"F32"}"#).is_err());
    }
}
