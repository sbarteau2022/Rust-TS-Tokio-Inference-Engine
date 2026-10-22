//! Independent file-payload comparison with native loaded parameter bytes.
use super::{
    native_mlx::NativeWeights,
    weights::{strict, Manifest},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    ffi::{c_char, CStr, CString},
    fs::File,
    io::{Read, Seek, SeekFrom},
};
unsafe extern "C" {
    fn rs_mlx_weight_bytes(
        store: *mut std::ffi::c_void,
        name: *const c_char,
        shape: *const i32,
        rank: usize,
        dtype: i32,
        offset: usize,
        output: *mut u8,
        count: usize,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}
fn verify(manifest: &Manifest) -> Result<(usize, usize, String)> {
    manifest.verify_stamps()?;
    let weights = NativeWeights::open(manifest)?;
    let mut raw = vec![0u8; 4 * 1024 * 1024];
    let mut native = raw.clone();
    let mut tensors = 0;
    let mut bytes = 0;
    let mut aggregate = Sha256::new();
    for shard in &manifest.shards {
        let mut file = File::open(shard)?;
        let mut length = [0u8; 8];
        file.read_exact(&mut length)?;
        let length = u64::from_le_bytes(length);
        ensure!((2..=8 * 1024 * 1024).contains(&length), "audit header cap");
        let mut header = vec![0; length as usize];
        file.read_exact(&mut header)?;
        let header = strict(&header)?;
        for (name, info) in header.as_object().context("audit header object")? {
            if name == "__metadata__" {
                continue;
            }
            let parameter = manifest
                .parameters
                .get(name)
                .context("audit parameter missing")?;
            let start = info["data_offsets"][0].as_u64().context("audit offset")?;
            let end = info["data_offsets"][1].as_u64().context("audit end")?;
            ensure!(
                end.checked_sub(start) == Some(parameter.bytes as u64),
                "audit tensor size"
            );
            let offset = 8u64
                .checked_add(length)
                .and_then(|n| n.checked_add(start))
                .context("audit file offset overflow")?;
            file.seek(SeekFrom::Start(offset))?;
            let cname = CString::new(name.as_str())?;
            let mut sha = Sha256::new();
            for offset in (0..parameter.bytes).step_by(raw.len()) {
                let count = raw.len().min(parameter.bytes - offset);
                file.read_exact(&mut raw[..count])?;
                let mut error = [0 as c_char; 1024];
                let status = unsafe {
                    rs_mlx_weight_bytes(
                        weights.0,
                        cname.as_ptr(),
                        parameter.shape.as_ptr(),
                        parameter.shape.len(),
                        parameter.dtype,
                        offset,
                        native.as_mut_ptr(),
                        count,
                        error.as_mut_ptr(),
                        error.len(),
                    )
                };
                ensure!(
                    status == 0,
                    "native byte audit: {}",
                    unsafe { CStr::from_ptr(error.as_ptr()) }.to_string_lossy()
                );
                ensure!(
                    raw[..count] == native[..count],
                    "native checkpoint bytes differ: {name} at {offset}"
                );
                sha.update(&raw[..count]);
            }
            aggregate.update((name.len() as u64).to_le_bytes());
            aggregate.update(name.as_bytes());
            aggregate.update(sha.finalize());
            tensors += 1;
            bytes += parameter.bytes;
        }
    }
    ensure!(
        tensors == manifest.parameters.len() && bytes == manifest.payload_bytes,
        "audit inventory mismatch"
    );
    manifest.verify_stamps()?;
    Ok((tensors, bytes, format!("{:x}", aggregate.finalize())))
}
pub async fn run() -> Result<()> {
    let manifest = Manifest::read(std::path::Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let identity = manifest.identity.clone();
    let (tensors, bytes, digest) = tokio::task::spawn_blocking(move || verify(&manifest)).await??;
    println!("INGESTION identity={identity} tensors={tensors} bytes={bytes} aggregate_sha256={digest} native_bytes_shape_dtype=true chunk_bytes=4194304 full_model_reference=false");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_payload_all_dtypes_chunks_and_bounds() -> Result<()> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "native-ingestion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)?;
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(path.clone());
        let values: Vec<(&str, &str, Vec<u8>)> = vec![
            (
                "f32",
                "F32",
                [-1.25f32, 0., 1.75]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
            (
                "i32",
                "I32",
                [-1i32, 0, 42]
                    .into_iter()
                    .flat_map(i32::to_le_bytes)
                    .collect(),
            ),
            (
                "u32",
                "U32",
                [0xdeadbeefu32, 0, 42]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect(),
            ),
            (
                "f16",
                "F16",
                [0xbc00u16, 0, 0x3c00]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            (
                "bf16",
                "BF16",
                [0xbf80u16, 0, 0x3f80]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            ("chunked", "U32", vec![0xa5u8; 4 * 1024 * 1024 + 4]),
        ];
        let mut header = serde_json::Map::new();
        let mut mapping = serde_json::Map::new();
        let mut payload = Vec::new();
        for (name, dtype, data) in values {
            let item = if ["F16", "BF16"].contains(&dtype) {
                2
            } else {
                4
            };
            let start = payload.len();
            payload.extend(&data);
            header.insert(name.into(),serde_json::json!({"dtype":dtype,"shape":[data.len()/item],"data_offsets":[start,payload.len()]}));
            mapping.insert(name.into(), serde_json::json!("model.safetensors"));
        }
        let header = serde_json::to_vec(&header)?;
        let mut file = (header.len() as u64).to_le_bytes().to_vec();
        file.extend(header);
        file.extend(&payload);
        std::fs::write(path.join("model.safetensors"), file)?;
        std::fs::write(
            path.join("model.safetensors.index.json"),
            serde_json::to_vec(&serde_json::json!({"weight_map":mapping}))?,
        )?;
        std::fs::write(
            path.join("config.json"),
            br#"{"quantization":{"bits":4,"group_size":32}}"#,
        )?;
        let manifest = Manifest::read(&path)?;
        let (count, bytes, _) = verify(&manifest)?;
        assert_eq!(count, 6);
        assert_eq!(bytes, payload.len());
        let weights = NativeWeights::open(&manifest)?;
        let mut output = [0xa5u8; 4];
        let good = c"f32";
        for (name, shape, dtype, offset) in [
            (good, &[2i32][..], 0, 0),
            (good, &[3][..], 1, 0),
            (good, &[3][..], 0, usize::MAX),
            (good, &[3][..], 0, 11),
            (c"missing", &[3][..], 0, 0),
        ] {
            let mut error = [0 as c_char; 128];
            let result = unsafe {
                rs_mlx_weight_bytes(
                    weights.0,
                    name.as_ptr(),
                    shape.as_ptr(),
                    1,
                    dtype,
                    offset,
                    output.as_mut_ptr(),
                    output.len(),
                    error.as_mut_ptr(),
                    error.len(),
                )
            };
            assert_ne!(result, 0);
            assert_eq!(output, [0xa5; 4]);
        }
        Ok(())
    }
    #[test]
    fn malformed_export_rejects_before_store() {
        let name = c"test";
        let shape = [1];
        let mut out = [0u8; 4];
        for (rank, dtype, count) in [
            (0, 0, 4),
            (5, 0, 4),
            (1, 5, 4),
            (1, 0, 0),
            (1, 0, usize::MAX),
        ] {
            let mut error = [0 as c_char; 128];
            let status = unsafe {
                rs_mlx_weight_bytes(
                    std::ptr::dangling_mut(),
                    name.as_ptr(),
                    shape.as_ptr(),
                    rank,
                    dtype,
                    0,
                    out.as_mut_ptr(),
                    count,
                    error.as_mut_ptr(),
                    error.len(),
                )
            };
            assert_ne!(status, 0);
            assert_eq!(out, [0; 4]);
        }
    }
}
