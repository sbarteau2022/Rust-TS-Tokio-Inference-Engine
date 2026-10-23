//! Post-norm ownership, numerical contract, baseline preservation and pressure gate.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    model_ir::{ModelStepIr, StepKind},
    native_mlx::Device,
    native_qwen::QwenDeviceExecutor,
};
use std::{
    io::{Read, Seek, SeekFrom},
    path::Path,
};
fn norm_weights() -> Result<Vec<f32>> {
    let dir = Path::new(include_str!("../fixtures/native-model.path").trim());
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("model.safetensors.index.json"))?)?;
    let key = "language_model.model.norm.weight";
    let shard = index["weight_map"][key].as_str().context("norm shard")?;
    let mut file = std::fs::File::open(dir.join(shard))?;
    let mut n = [0; 8];
    file.read_exact(&mut n)?;
    let n = u64::from_le_bytes(n);
    ensure!(n <= 8 * 1024 * 1024, "header cap");
    let mut header = vec![0; n as usize];
    file.read_exact(&mut header)?;
    let h: serde_json::Value = serde_json::from_slice(&header)?;
    ensure!(
        h[key]["dtype"] == "BF16" && h[key]["shape"] == serde_json::json!([2048]),
        "norm layout"
    );
    let start = h[key]["data_offsets"][0].as_u64().context("norm offset")?;
    file.seek(SeekFrom::Start(8 + n + start))?;
    let mut bytes = vec![0; 4096];
    file.read_exact(&mut bytes)?;
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|x| f32::from_bits((u16::from_le_bytes([x[0], x[1]]) as u32) << 16))
        .collect())
}
fn bits_equal(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
}
#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("native_hidden_smoke NEW_DIRECTORY")?;
    std::fs::create_dir(&path)?;
    let weights = norm_weights()?;
    let executor = QwenDeviceExecutor::open(Device::Gpu, 4, 512 * 1024 * 1024).await?;
    let mut maximum = 0f64;
    let mut last_logit_maximum = 0f64;
    for count in [1, 2, 4, 8] {
        let empty = executor.empty_state();
        let tokens = (1..=count as u32).collect::<Vec<_>>();
        let ir = ModelStepIr::compile(StepKind::Prefill, 0, tokens.clone(), 248320)?.with_trace();
        let baseline = executor.execute(&empty, ir).await?;
        let all = executor.verify_block(&empty, tokens.clone()).await?;
        let hidden = executor.hidden_block(&empty, tokens).await?;
        ensure!(
            hidden.state.hidden_rows() == count
                && hidden.state.bytes() == baseline.state.bytes() + count * 2048 * 4,
            "hidden row/charge"
        );
        ensure!(
            all.logits.len() == count
                && all
                    .logits
                    .iter()
                    .zip(&hidden.logits)
                    .all(|(a, b)| bits_equal(a, b)),
            "logit preservation"
        );
        // Single-row vs batched vocabulary matmul uses different numerical kernels.
        let last = hidden.logits.last().unwrap();
        let relative = (baseline
            .logits
            .iter()
            .zip(last)
            .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
            .sum::<f64>()
            / baseline
                .logits
                .iter()
                .map(|a| (*a as f64).powi(2))
                .sum::<f64>()
                .max(1e-30))
        .sqrt();
        ensure!(
            relative <= 1e-5,
            "single-row/batched logit numerical gate {relative}"
        );
        last_logit_maximum = last_logit_maximum.max(relative);
        let state = executor.snapshot(&baseline.state).await?;
        ensure!(
            bits_equal(&state, &executor.snapshot(&hidden.state).await?),
            "unified state preservation"
        );
        let values = executor.hidden_snapshot(&hidden.state).await?;
        ensure!(
            values.len() == count * 2048 && values.iter().all(|x| x.is_finite()),
            "hidden geometry"
        );
        let x = &baseline.trace[(39 * 5 + 4) * count * 2048..40 * 5 * count * 2048];
        for (row, actual) in x
            .as_chunks::<2048>()
            .0
            .iter()
            .zip(values.as_chunks::<2048>().0.iter())
        {
            let scale =
                (row.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / 2048. + 1e-6).sqrt();
            let expected = row
                .iter()
                .zip(&weights)
                .map(|(x, w)| *x as f64 / scale * (*w as f64))
                .collect::<Vec<_>>();
            let error = (actual
                .iter()
                .zip(&expected)
                .map(|(a, b)| (*a as f64 - b).powi(2))
                .sum::<f64>()
                / expected.iter().map(|x| x * x).sum::<f64>().max(1e-30))
            .sqrt();
            ensure!(error <= 1e-5, "post-norm scalar contract {error}");
            maximum = maximum.max(error);
        }
        ensure!(
            executor.hidden_snapshot(&baseline.state).await.is_err(),
            "ordinary state must not fabricate hidden"
        );
        let a = executor.step(&baseline.state, vec![17]).await?;
        drop(all);
        let b = executor.step(&hidden.state, vec![17]).await?;
        ensure!(
            bits_equal(&a.logits, &b.logits)
                && bits_equal(
                    &executor.snapshot(&a.state).await?,
                    &executor.snapshot(&b.state).await?
                ),
            "continuation preservation"
        );
        // Four live versions saturate the table; rejection must retain the parent.
        ensure!(
            executor
                .hidden_block(&hidden.state, vec![19])
                .await
                .is_err(),
            "pressure rejection"
        );
        ensure!(
            bits_equal(&values, &executor.hidden_snapshot(&hidden.state).await?),
            "pressure changed hidden parent"
        );
        drop(a);
        drop(b);
        drop(baseline);
        drop(hidden);
        ensure!(executor.stats().await?.residents == 0, "residency leak");
    }
    let stats = executor.shutdown().await?;
    let result = serde_json::json!({"pass":true,"counts":[1,2,4,8],"all_position_logits_and_unified_state_bitwise":true,"single_row_batched_logit_max_relative_l2":last_logit_maximum,"post_norm_scalar_max_relative_l2":maximum,"hidden_transfer":"diagnostic snapshot only; production handles remain native","logical_bytes_at_shutdown":stats.logical_bytes,"mtp_forward_qualified":false});
    std::fs::write(
        Path::new(&path).join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    println!("{result}");
    Ok(())
}
