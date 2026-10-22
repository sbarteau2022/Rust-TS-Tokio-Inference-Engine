//! Opt-in offline layer/state capture; never on server admission or decoding hot paths.
use super::{
    model_ir::{ModelStepIr, StepKind},
    native_extended::relative,
    native_mlx::Device,
    native_qwen::QwenExecutor,
    native_tokenizer::NativeTokenizer,
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{io::Write, path::Path};

fn write_vector(directory: &Path, name: &str, values: &[f32]) -> Result<String> {
    ensure!(
        !values.is_empty() && values.iter().all(|v| v.is_finite()),
        "invalid capture values"
    );
    let data: Vec<_> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    std::fs::write(directory.join(name), &data)?;
    Ok(format!("{:x}", Sha256::digest(data)))
}
pub async fn run(directory: &Path) -> Result<()> {
    // Existing directories are rejected so interrupted/stale evidence is never reused.
    std::fs::create_dir(directory).context("new scratch directory required")?;
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let mut report = std::fs::File::create(directory.join("native.jsonl"))?;
    for id in ["s10", "r11"] {
        let row = include_str!("../../fixtures/native-domain-heldout.tsv")
            .lines()
            .find(|line| line.starts_with(&format!("{id}\t")))
            .context("frozen case missing")?;
        let fields: Vec<_> = row.split('\t').collect();
        ensure!(fields.len() == 5, "frozen fields");
        let ids = tokenizer.chat(None, fields[3])?;
        let mut state = executor.empty_state();
        for tokens in ids.chunks(8) {
            let position = state.position();
            let plain = executor.step(&state, tokens.to_vec()).await?;
            let traced = executor
                .execute(
                    &state,
                    ModelStepIr::compile(StepKind::Prefill, position, tokens.to_vec(), 248320)?
                        .with_trace(),
                )
                .await?;
            let logit_control = relative(&plain.logits, &traced.logits)?;
            let state_control = relative(
                plain.state.snapshot_values(),
                traced.state.snapshot_values(),
            )?;
            ensure!(
                logit_control <= 1e-6 && state_control <= 1e-6,
                "trace observer materially perturbs native execution"
            );
            let prefix = format!("{id}-{position}");
            let trace_hash =
                write_vector(directory, &format!("{prefix}-trace.f32le"), &traced.trace)?;
            let state_hash = write_vector(
                directory,
                &format!("{prefix}-state.f32le"),
                traced.state.snapshot_values(),
            )?;
            let logit_hash =
                write_vector(directory, &format!("{prefix}-logits.f32le"), &traced.logits)?;
            let record = serde_json::json!({"case":id,"prompt":fields[3],"prompt_ids":ids,
                "identity":executor.manifest().identity,"offset":position,"count":tokens.len(),
                "position":traced.state.position(),"trace_sha256":trace_hash,"state_sha256":state_hash,
                "logits_sha256":logit_hash,"trace_values":traced.trace.len(),
                "state_values":traced.state.snapshot_values().len(),"logit_control":logit_control,"state_control":state_control});
            writeln!(report, "{record}")?;
            report.flush()?;
            eprintln!("LAYER-CAPTURE case={id} offset={position} logit_control={logit_control} state_control={state_control}");
            state = plain.state;
        }
    }
    let resources = executor.shutdown().await?;
    ensure!(resources.leases == 0, "layer capture did not drain");
    std::fs::write(directory.join("drain.txt"), "leases=0 cases=2\n")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_rejects_empty_nonfinite_before_write() {
        for values in [vec![], vec![f32::NAN], vec![f32::INFINITY]] {
            assert!(
                write_vector(Path::new("/nonexistent-layer-reference"), "bad", &values).is_err()
            );
        }
    }
}
