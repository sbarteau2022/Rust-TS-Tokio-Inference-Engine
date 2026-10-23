//! Strict sidecar ingestion; separate storage preserves target normalization semantics.
use super::weights::{strict, Manifest, Quantization};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

pub struct MtpSidecar {
    pub(crate) path: PathBuf,
    pub(crate) identity: String,
    pub header_sha256: String,
    pub payload_bytes: u64,
}
impl MtpSidecar {
    pub fn read(manifest: &Manifest) -> Result<Self> {
        manifest.verify_stamps()?;
        let value = Self::parse(
            &manifest.directory.join("model-mtp.safetensors"),
            &manifest.identity,
            |module| manifest.module_quantization(module),
        )?;
        manifest.verify_stamps()?;
        Ok(value)
    }
    fn parse(path: &Path, identity: &str, quant: impl Fn(&str) -> Quantization) -> Result<Self> {
        ensure!(
            std::fs::symlink_metadata(path)?.is_file(),
            "MTP must be a regular local file"
        );
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();
        let mut prefix = [0; 8];
        file.read_exact(&mut prefix)?;
        let length = u64::from_le_bytes(prefix);
        ensure!(
            (2..=1024 * 1024).contains(&length) && length.checked_add(8).is_some_and(|n| n <= size),
            "MTP header bounds"
        );
        let mut bytes = vec![0; length as usize];
        file.read_exact(&mut bytes)?;
        let value = strict(&bytes)?;
        let header = value.as_object().context("MTP header object")?;
        let expected = contracts();
        ensure!(
            header
                .keys()
                .filter(|k| k.as_str() != "__metadata__")
                .count()
                == expected.len(),
            "MTP parameter count"
        );
        let mut ranges = Vec::new();
        for (name, dtype, shape) in expected {
            let entry = header
                .get(name)
                .with_context(|| format!("missing MTP parameter {name}"))?;
            let dims: Vec<i32> = serde_json::from_value(entry["shape"].clone())?;
            ensure!(dims == shape, "MTP shape mismatch: {name}");
            ensure!(
                entry["dtype"] == if dtype == 2 { "U32" } else { "BF16" },
                "MTP dtype mismatch: {name}"
            );
            let count = dims
                .iter()
                .try_fold(if dtype == 2 { 4u64 } else { 2u64 }, |a, b| {
                    a.checked_mul(*b as u64)
                })
                .context("MTP tensor overflow")?;
            let offsets: Vec<u64> = serde_json::from_value(entry["data_offsets"].clone())?;
            ensure!(
                offsets.len() == 2
                    && offsets[1].checked_sub(offsets[0]) == Some(count)
                    && offsets[1] <= size - 8 - length,
                "MTP tensor offsets: {name}"
            );
            ranges.push((offsets[0], offsets[1]));
            if dtype == 2 {
                let module = name
                    .strip_suffix(".weight")
                    .context("packed MTP weight name")?;
                let q = quant(module);
                ensure!(
                    [2, 3, 4, 5, 6, 8].contains(&q.bits) && [32, 64, 128].contains(&q.group),
                    "MTP quantization bounds"
                );
                let packed = *dims.last().context("MTP packed rank")? as u64 * 32;
                ensure!(packed.is_multiple_of(q.bits as u64), "MTP packed bit width");
                let width = packed / q.bits as u64;
                ensure!(width.is_multiple_of(q.group as u64), "MTP group width");
                let mut scales = dims[..dims.len() - 1].to_vec();
                scales.push((width / q.group as u64) as i32);
                for suffix in ["scales", "biases"] {
                    let companion = header
                        .get(&format!("{module}.{suffix}"))
                        .context("MTP quantization companion")?;
                    ensure!(
                        companion["shape"] == serde_json::to_value(&scales)?,
                        "MTP quantization geometry: {module}"
                    );
                }
            }
        }
        ranges.sort_unstable();
        let mut end = 0;
        for (start, next) in ranges {
            ensure!(start == end, "MTP payload gap or overlap");
            end = next;
        }
        ensure!(end == size - 8 - length, "MTP trailing payload");
        Ok(Self {
            path: path.to_owned(),
            identity: identity.to_owned(),
            header_sha256: format!("{:x}", Sha256::digest(bytes)),
            payload_bytes: end,
        })
    }
}
fn contracts() -> Vec<(&'static str, i32, &'static [i32])> {
    vec![
        ("language_model.mtp.fc.weight", 4, &[2048, 4096]),
        (
            "language_model.mtp.layers.0.input_layernorm.weight",
            4,
            &[2048],
        ),
        (
            "language_model.mtp.layers.0.mlp.gate.weight",
            4,
            &[256, 2048],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.down_proj.biases",
            4,
            &[2048, 4],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.down_proj.scales",
            4,
            &[2048, 4],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.down_proj.weight",
            2,
            &[2048, 128],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.gate_proj.biases",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.gate_proj.scales",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.gate_proj.weight",
            2,
            &[512, 512],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.up_proj.biases",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.up_proj.scales",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert.up_proj.weight",
            2,
            &[512, 512],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert_gate.biases",
            4,
            &[1, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert_gate.scales",
            4,
            &[1, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.shared_expert_gate.weight",
            2,
            &[1, 512],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.down_proj.biases",
            4,
            &[256, 2048, 8],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.down_proj.scales",
            4,
            &[256, 2048, 8],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.down_proj.weight",
            2,
            &[256, 2048, 64],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.gate_proj.biases",
            4,
            &[256, 512, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.gate_proj.scales",
            4,
            &[256, 512, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.gate_proj.weight",
            2,
            &[256, 512, 256],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.up_proj.biases",
            4,
            &[256, 512, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.up_proj.scales",
            4,
            &[256, 512, 32],
        ),
        (
            "language_model.mtp.layers.0.mlp.switch_mlp.up_proj.weight",
            2,
            &[256, 512, 256],
        ),
        (
            "language_model.mtp.layers.0.post_attention_layernorm.weight",
            4,
            &[2048],
        ),
        (
            "language_model.mtp.layers.0.self_attn.k_norm.weight",
            4,
            &[256],
        ),
        (
            "language_model.mtp.layers.0.self_attn.k_proj.biases",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.k_proj.scales",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.k_proj.weight",
            2,
            &[512, 320],
        ),
        (
            "language_model.mtp.layers.0.self_attn.o_proj.biases",
            4,
            &[2048, 64],
        ),
        (
            "language_model.mtp.layers.0.self_attn.o_proj.scales",
            4,
            &[2048, 64],
        ),
        (
            "language_model.mtp.layers.0.self_attn.o_proj.weight",
            2,
            &[2048, 512],
        ),
        (
            "language_model.mtp.layers.0.self_attn.q_norm.weight",
            4,
            &[256],
        ),
        (
            "language_model.mtp.layers.0.self_attn.q_proj.biases",
            4,
            &[8192, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.q_proj.scales",
            4,
            &[8192, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.q_proj.weight",
            2,
            &[8192, 320],
        ),
        (
            "language_model.mtp.layers.0.self_attn.v_proj.biases",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.v_proj.scales",
            4,
            &[512, 16],
        ),
        (
            "language_model.mtp.layers.0.self_attn.v_proj.weight",
            2,
            &[512, 384],
        ),
        ("language_model.mtp.norm.weight", 4, &[2048]),
        (
            "language_model.mtp.pre_fc_norm_embedding.weight",
            4,
            &[2048],
        ),
        ("language_model.mtp.pre_fc_norm_hidden.weight", 4, &[2048]),
    ]
}
/// Load into a distinct MLX store. This does not execute or qualify the head.
pub async fn inspect_selected() -> Result<()> {
    tokio::task::spawn_blocking(|| -> Result<()> {
        let directory=Path::new("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined");
        let manifest=Manifest::read(directory)?;
        let sidecar=MtpSidecar::read(&manifest)?;
        let _store=super::native_mlx::NativeWeights::open_mtp(&manifest,&sidecar)?;
        println!("MTP-INGESTION-ONLY tensors=42 payload_bytes={} header_sha256={} artifact={} execution_qualified=false",sidecar.payload_bytes,sidecar.header_sha256,manifest.identity);
        Ok(())
    }).await??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    fn quant(module: &str) -> Quantization {
        let (bits, group) =
            if module.ends_with("self_attn.k_proj") || module.ends_with("self_attn.q_proj") {
                (5, 128)
            } else if module.ends_with("self_attn.v_proj") {
                (6, 128)
            } else if module.contains("shared_expert.") {
                (8, 128)
            } else if module.ends_with("shared_expert_gate") {
                (8, 64)
            } else {
                (4, 64)
            };
        Quantization { bits, group }
    }
    fn header() -> serde_json::Value {
        let mut result = serde_json::Map::new();
        let mut offset = 0u64;
        for (name, dtype, shape) in contracts() {
            let bytes = shape
                .iter()
                .fold(if dtype == 2 { 4 } else { 2 }, |a, b| a * (*b as u64));
            result.insert(name.into(),serde_json::json!({"shape":shape,"dtype":if dtype==2 {"U32"}else{"BF16"},"data_offsets":[offset,offset+bytes]}));
            offset += bytes;
        }
        result.into()
    }
    fn fixture(value: &serde_json::Value) -> Fixture {
        let path = std::env::temp_dir().join(format!(
            "rust-mtp-{}-{}.safetensors",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        let bytes = serde_json::to_vec(value).unwrap();
        use std::io::Write;
        file.write_all(&(bytes.len() as u64).to_le_bytes()).unwrap();
        file.write_all(&bytes).unwrap();
        let payload = value
            .as_object()
            .unwrap()
            .values()
            .filter_map(|v| v["data_offsets"][1].as_u64())
            .max()
            .unwrap_or(0);
        file.set_len(8 + bytes.len() as u64 + payload).unwrap();
        Fixture(path)
    }
    #[test]
    fn complete_mixed_layout_passes_without_reading_payload() {
        let f = fixture(&header());
        let sidecar = MtpSidecar::parse(&f.0, "test", quant).unwrap();
        assert_eq!(sidecar.identity, "test");
        assert!(sidecar.payload_bytes > 400_000_000);
    }
    #[test]
    fn wrong_global_quantization_is_rejected() {
        let f = fixture(&header());
        assert!(MtpSidecar::parse(&f.0, "test", |_| Quantization { bits: 4, group: 64 }).is_err());
        for q in [
            Quantization { bits: 0, group: 64 },
            Quantization { bits: 5, group: 0 },
            Quantization { bits: 7, group: 64 },
        ] {
            assert!(MtpSidecar::parse(&f.0, "test", |_| q).is_err());
        }
    }
    #[test]
    fn every_required_parameter_is_enforced() {
        for (name, _, _) in contracts() {
            let mut h = header();
            h.as_object_mut().unwrap().remove(name);
            let f = fixture(&h);
            assert!(MtpSidecar::parse(&f.0, "test", quant).is_err(), "{name}");
        }
    }
    #[test]
    fn adversarial_shape_dtype_offsets_and_extras_fail() {
        let key = "language_model.mtp.fc.weight";
        for field in ["shape", "dtype", "data_offsets"] {
            let mut h = header();
            h[key][field] = match field {
                "shape" => serde_json::json!([4096, 2048]),
                "dtype" => serde_json::json!("F32"),
                _ => serde_json::json!([1, 16777217]),
            };
            let f = fixture(&h);
            assert!(MtpSidecar::parse(&f.0, "test", quant).is_err());
        }
        let mut h = header();
        h["unknown"] = h[key].clone();
        let f = fixture(&h);
        assert!(MtpSidecar::parse(&f.0, "test", quant).is_err());
        let f = fixture(&header());
        std::fs::OpenOptions::new()
            .write(true)
            .open(&f.0)
            .unwrap()
            .set_len(9)
            .unwrap();
        assert!(MtpSidecar::parse(&f.0, "test", quant).is_err());
    }
    #[test]
    fn trailing_payload_and_symlinks_fail() {
        let f = fixture(&header());
        let len = std::fs::metadata(&f.0).unwrap().len();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&f.0)
            .unwrap()
            .set_len(len + 1)
            .unwrap();
        assert!(MtpSidecar::parse(&f.0, "test", quant).is_err());
        let link = Fixture(f.0.with_extension("link"));
        std::os::unix::fs::symlink(&f.0, &link.0).unwrap();
        assert!(MtpSidecar::parse(&link.0, "test", quant).is_err());
    }
}
