//! Read-only full-content artifact qualification; does not execute the drafter.
use anyhow::{ensure, Context, Result};
use rust_server::serving::native_dspark::{DsparkArtifact, TARGET_TAPS};
use std::path::Path;
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        args.len() == 2,
        "native_dspark_inventory SNAPSHOT NEW_REPORT.json"
    );
    ensure!(!Path::new(&args[1]).exists(), "report already exists");
    let artifact =
        DsparkArtifact::read(Path::new(&args[0])).context("DSpark artifact qualification")?;
    let report = serde_json::json!({
        "pass":true, "scope":"strict config/tensor inventory and complete file hashes; no native draft execution or model-quality qualification",
        "native_execution_qualified":false, "target_compatibility_qualified":false,
        "artifact_identity":artifact.identity, "weights_sha256":artifact.weights_sha256,
        "config_sha256":artifact.config_sha256, "payload_bytes":artifact.payload_bytes,
        "weights_path":artifact.weights_path, "config_path":artifact.config_path,
        "target_taps":TARGET_TAPS, "tensor_count":75, "backbone_layers":6,
        "sample_from_anchor":true, "markov_rank":256,
        "checkpoint_proposal_length":8, "normalized_max_verification_width":9,
        "default_runtime_verification_width":8, "default_block_size_policy":"adaptive",
        "target_tap_indexing_qualified":false,
    });
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    Ok(())
}
