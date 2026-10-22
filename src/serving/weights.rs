//! Immutable native checkpoint inventory. JSON is confined to standard on-disk metadata.
use anyhow::{ensure, Context, Result};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quantization {
    pub bits: i32,
    pub group: i32,
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub shape: Vec<i32>,
    pub dtype: i32,
    pub bytes: usize,
}
#[derive(Clone, Debug)]
pub struct Manifest {
    pub directory: PathBuf,
    pub identity: String,
    pub parameters: BTreeMap<String, Parameter>,
    pub shards: Vec<PathBuf>,
    pub payload_bytes: usize,
    pub quantization: Quantization,
    pub overrides: BTreeMap<String, Quantization>,
    pub mtp_present: bool,
    stamps: BTreeMap<PathBuf, (u64, SystemTime)>,
}
#[derive(Clone, Debug)]
pub struct ModelWeights {
    pub(crate) owner: u64,
    pub(crate) manifest: Arc<Manifest>,
}
struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = StrictValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("metadata without duplicate keys")
            }
            fn visit_bool<E: serde::de::Error>(
                self,
                v: bool,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| StrictValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite metadata"))
            }
            fn visit_str<E: serde::de::Error>(
                self,
                v: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::String(v.into())))
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = a.next_element::<StrictValue>()? {
                    values.push(v.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((k, v)) = a.next_entry::<String, StrictValue>()? {
                    if values.insert(k, v.0).is_some() {
                        return Err(serde::de::Error::custom("duplicate metadata key"));
                    }
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        d.deserialize_any(V)
    }
}
fn strict(data: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice::<StrictValue>(data)?.0)
}
struct UniqueMap(BTreeMap<String, Value>);
impl<'de> Deserialize<'de> for UniqueMap {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct MapVisitor;
        impl<'de> Visitor<'de> for MapVisitor {
            type Value = UniqueMap;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("unique metadata keys")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<UniqueMap, A::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = a.next_entry::<String, StrictValue>()? {
                    if values.insert(key, value.0).is_some() {
                        return Err(serde::de::Error::custom("duplicate metadata key"));
                    }
                }
                Ok(UniqueMap(values))
            }
        }
        d.deserialize_map(MapVisitor)
    }
}
fn bounded(path: &Path, limit: u64) -> Result<Vec<u8>> {
    ensure!(
        fs::symlink_metadata(path)?.is_file(),
        "metadata must be a regular local file"
    );
    let file = File::open(path)?;
    ensure!(file.metadata()?.len() <= limit, "metadata exceeds byte cap");
    let mut data = Vec::new();
    file.take(limit + 1).read_to_end(&mut data)?;
    ensure!(data.len() as u64 <= limit, "metadata grew past cap");
    Ok(data)
}
fn quant(value: &Value, default: Option<Quantization>) -> Result<Quantization> {
    let bits = value
        .get("bits")
        .and_then(Value::as_i64)
        .or(default.map(|v| v.bits as i64))
        .context("quantization bits missing")?;
    let group = value
        .get("group_size")
        .and_then(Value::as_i64)
        .or(default.map(|v| v.group as i64))
        .context("quantization group missing")?;
    ensure!(
        [2, 3, 4, 5, 6, 8].contains(&bits) && [32, 64, 128].contains(&group),
        "unsupported affine quantization"
    );
    ensure!(
        value
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("affine")
            == "affine",
        "unsupported quantization mode"
    );
    Ok(Quantization {
        bits: bits as i32,
        group: group as i32,
    })
}
fn stamps(directory: &Path) -> Result<BTreeMap<PathBuf, (u64, SystemTime)>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "symlink checkpoint artifact"
        );
        if metadata.is_file() {
            result.insert(path, (metadata.len(), metadata.modified()?));
        }
    }
    Ok(result)
}
impl Manifest {
    pub fn read(directory: &Path) -> Result<Self> {
        ensure!(
            directory.is_absolute(),
            "absolute checkpoint directory required"
        );
        let directory = directory.canonicalize()?;
        let initial_stamps = stamps(&directory)?;
        let config: Value = strict(&bounded(&directory.join("config.json"), 1024 * 1024)?)?;
        let q = config
            .get("quantization")
            .context("quantization configuration required")?;
        let quantization = quant(q, None)?;
        let mut overrides = BTreeMap::new();
        for (key, value) in q.as_object().context("quantization object required")? {
            if value.is_object() {
                overrides.insert(key.clone(), quant(value, Some(quantization))?);
            }
        }
        let index: Value = strict(&bounded(
            &directory.join("model.safetensors.index.json"),
            4 * 1024 * 1024,
        )?)?;
        let mapping = index
            .get("weight_map")
            .and_then(Value::as_object)
            .context("weight map required")?;
        ensure!(
            !mapping.is_empty() && mapping.len() <= 100000,
            "tensor count outside bounds"
        );
        let mut shards = std::collections::BTreeSet::new();
        for value in mapping.values() {
            let name = value.as_str().context("shard name must be text")?;
            ensure!(
                Path::new(name).file_name().and_then(|v| v.to_str()) == Some(name)
                    && name.ends_with(".safetensors"),
                "unsafe shard path"
            );
            shards.insert(directory.join(name));
        }
        ensure!(shards.len() <= 16, "too many shards");
        let mut parameters = BTreeMap::new();
        let mut payload_bytes = 0usize;
        for shard in &shards {
            ensure!(
                fs::symlink_metadata(shard)?.is_file(),
                "shard must be a regular file"
            );
            let mut file = File::open(shard)?;
            let size = file.metadata()?.len();
            let mut length = [0u8; 8];
            file.read_exact(&mut length)?;
            let length = u64::from_le_bytes(length);
            ensure!(
                (2..=8 * 1024 * 1024).contains(&length)
                    && length.checked_add(8).is_some_and(|n| n <= size),
                "safetensors header size"
            );
            let mut header = vec![0; length as usize];
            file.read_exact(&mut header)?;
            let values: UniqueMap = serde_json::from_slice(&header)?;
            let mut ranges = Vec::new();
            for (name, value) in values.0 {
                if name == "__metadata__" {
                    continue;
                }
                ensure!(
                    mapping.get(&name).and_then(Value::as_str)
                        == shard.file_name().and_then(|v| v.to_str()),
                    "tensor/index shard mismatch"
                );
                let shape: Vec<i32> = serde_json::from_value(
                    value.get("shape").context("tensor shape missing")?.clone(),
                )?;
                ensure!(
                    !shape.is_empty() && shape.len() <= 4 && shape.iter().all(|n| *n > 0),
                    "parameter shape bounds"
                );
                let (dtype, item): (i32, usize) = match value.get("dtype").and_then(Value::as_str) {
                    Some("F32") => (0, 4),
                    Some("I32") => (1, 4),
                    Some("U32") => (2, 4),
                    Some("F16") => (3, 2),
                    Some("BF16") => (4, 2),
                    _ => anyhow::bail!("unsupported parameter dtype"),
                };
                let bytes = shape
                    .iter()
                    .try_fold(item, |a, b| a.checked_mul(*b as usize))
                    .context("parameter size overflow")?;
                let offsets: Vec<u64> = serde_json::from_value(
                    value
                        .get("data_offsets")
                        .context("tensor offsets missing")?
                        .clone(),
                )?;
                ensure!(
                    offsets.len() == 2
                        && offsets[0] <= offsets[1]
                        && offsets[1] - offsets[0] == bytes as u64
                        && offsets[1] <= size - 8 - length,
                    "parameter offsets outside shard"
                );
                ranges.push((offsets[0], offsets[1]));
                payload_bytes = payload_bytes
                    .checked_add(bytes)
                    .filter(|n| *n <= 32 * 1024usize.pow(3))
                    .context("parameter payload exceeds 32 GiB")?;
                ensure!(
                    parameters
                        .insert(
                            name,
                            Parameter {
                                shape,
                                dtype,
                                bytes
                            }
                        )
                        .is_none(),
                    "duplicate parameter"
                );
            }
            ranges.sort_unstable();
            ensure!(
                ranges.windows(2).all(|r| r[0].1 <= r[1].0),
                "overlapping parameter offsets"
            );
            file.seek(SeekFrom::Start(0))?;
        }
        ensure!(
            parameters.len() == mapping.len(),
            "index contains missing parameters"
        );
        let mut hasher = Sha256::new();
        let mut hashed_stamps = BTreeMap::new();
        let mut files: Vec<_> = fs::read_dir(&directory)?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<std::io::Result<_>>()?;
        files.sort();
        for path in files {
            let metadata = fs::symlink_metadata(&path)?;
            if !metadata.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .context("non-UTF8 artifact name")?;
            hasher.update((name.len() as u64).to_le_bytes());
            hasher.update(name.as_bytes());
            hasher.update(metadata.len().to_le_bytes());
            let mut file = File::open(&path)?;
            let mut buffer = vec![0; 8 * 1024 * 1024];
            loop {
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                hasher.update(&buffer[..count]);
            }
            hashed_stamps.insert(path, (metadata.len(), metadata.modified()?));
        }
        ensure!(
            initial_stamps == hashed_stamps,
            "checkpoint changed while parsing or hashing"
        );
        let manifest = Self {
            directory: directory.clone(),
            identity: format!("{:x}", hasher.finalize()),
            parameters,
            shards: shards.into_iter().collect(),
            payload_bytes,
            quantization,
            overrides,
            mtp_present: directory.join("model-mtp.safetensors").is_file(),
            stamps: initial_stamps,
        };
        manifest.verify_stamps()?;
        Ok(manifest)
    }
    pub fn verify_stamps(&self) -> Result<()> {
        ensure!(
            stamps(&self.directory)? == self.stamps,
            "checkpoint artifact set changed"
        );
        for (path, (size, time)) in &self.stamps {
            let metadata = fs::symlink_metadata(path)?;
            ensure!(
                metadata.is_file() && metadata.len() == *size && metadata.modified()? == *time,
                "checkpoint changed since fingerprinting"
            );
        }
        Ok(())
    }
    pub fn module_quantization(&self, module: &str) -> Quantization {
        self.overrides
            .get(module)
            .copied()
            .unwrap_or(self.quantization)
    }
}
impl ModelWeights {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}
