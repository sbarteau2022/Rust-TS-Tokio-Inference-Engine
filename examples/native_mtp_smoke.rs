//! Native mixed-precision MTP numerical, state, alignment and pressure gate.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mlx::Device,
    native_qwen::{ExecutionTuning, QwenDeviceExecutor},
};
use std::{future::Future, path::Path, task::Poll};
fn relative(a: &[f32], b: &[f32]) -> Result<f64> {
    ensure!(
        !a.is_empty() && a.len() == b.len() && a.iter().chain(b).all(|x| x.is_finite()),
        "comparison geometry"
    );
    let e = (a
        .iter()
        .zip(b)
        .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
        .sum::<f64>()
        / a.iter()
            .map(|a| (*a as f64).powi(2))
            .sum::<f64>()
            .max(1e-30))
    .sqrt();
    ensure!(e <= 1e-5, "MTP numerical drift {e}");
    Ok(e)
}
fn bits(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
}
fn argmax(x: &[f32]) -> u32 {
    x.iter()
        .enumerate()
        .max_by(|(i, a), (j, b)| a.total_cmp(b).then_with(|| j.cmp(i)))
        .unwrap()
        .0 as u32
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_smoke NEW_DIRECTORY")?;
    let graph = std::env::args().nth(2).unwrap_or_else(|| "eager".into());
    let group: u32 = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    ensure!(
        matches!(graph.as_str(), "eager" | "compiled-decode") && matches!(group, 1 | 4 | 10 | 40),
        "execution profile"
    );
    std::fs::create_dir(&out)?;
    let executor = QwenDeviceExecutor::open_mtp_configured(
        Device::Gpu,
        8,
        768 * 1024 * 1024,
        ExecutionTuning {
            layer_group: group,
            compiled_decode: graph == "compiled-decode",
            fused_recurrent: graph == "compiled-decode",
            ..ExecutionTuning::default()
        },
    )
    .await?;
    let mut cases = Vec::new();
    for count in 1usize..=8 {
        eprintln!("CASE count={count} target/hidden");
        let transfers_before = executor.stats().await?;
        let shifted = (2..=count as u32 + 1).collect::<Vec<_>>();
        let target = executor
            .hidden_block(&executor.empty_state(), (1..=count as u32).collect())
            .await?;
        let empty = executor.empty_mtp_state()?;
        ensure!(
            executor
                .mtp_prefill(&empty, &target.state, 1, vec![2])
                .await
                .is_err(),
            "misaligned source accepted"
        );
        ensure!(
            executor
                .mtp_prefill(&empty, &target.state, 0, vec![248320])
                .await
                .is_err(),
            "out of vocabulary accepted"
        );
        ensure!(
            executor.mtp_next(&empty, 2).await.is_err(),
            "empty autoregressive source accepted"
        );
        #[cfg(feature = "debug-validation")]
        let chunk = executor
            .mtp_prefill_debug(&empty, &target.state, 0, shifted.clone())
            .await?;
        #[cfg(not(feature = "debug-validation"))]
        let chunk = executor
            .mtp_prefill(&empty, &target.state, 0, shifted.clone())
            .await?;
        eprintln!("CASE count={count} chunk complete; serial comparison");
        let mut serial = empty.clone();
        let mut serial_logits = Vec::new();
        for (row, token) in shifted.into_iter().enumerate() {
            let step = executor
                .mtp_prefill(&serial, &target.state, row, vec![token])
                .await?;
            serial = step.state;
            serial_logits = step.logits;
        }
        ensure!(
            executor.stats().await?.exported_bytes == transfers_before.exported_bytes
                && executor.stats().await?.uploaded_bytes == transfers_before.uploaded_bytes,
            "MTP hot-path host transfer"
        );
        let logit_drift = relative(&chunk.logits, &serial_logits)
            .with_context(|| format!("count={count} chunk/serial logits"))?;
        ensure!(
            argmax(&chunk.logits) == argmax(&serial_logits),
            "MTP chunk/serial argmax"
        );
        let chunk_values = executor.mtp_snapshot(&chunk.state).await?;
        let serial_values = executor.mtp_snapshot(&serial).await?;
        let target_parent = executor.snapshot(&target.state).await?;
        let kv = 512 * 1024;
        ensure!(
            chunk_values.len() == kv + count * 2048 && serial_values.len() == kv + 2048,
            "fixed-capacity MTP snapshot geometry"
        );
        let kv_drift = relative(&chunk_values[..kv], &serial_values[..kv])
            .with_context(|| format!("count={count} entire fixed KV"))?;
        let hidden_drift = relative(
            &chunk_values[chunk_values.len() - 2048..],
            &serial_values[kv..],
        )
        .with_context(|| format!("count={count} final hidden"))?;
        let token = argmax(&chunk.logits);
        let a = executor.mtp_next(&chunk.state, token).await?;
        let b = executor.mtp_next(&chunk.state, token).await?;
        ensure!(
            bits(&a.logits, &b.logits)
                && bits(
                    &executor.mtp_snapshot(&a.state).await?,
                    &executor.mtp_snapshot(&b.state).await?
                ),
            "MTP branch reproducibility"
        );
        let parent = executor.mtp_snapshot(&chunk.state).await?;
        let mut pressure = Vec::new();
        while executor.stats().await?.residents < 8 {
            pressure.push(executor.mtp_next(&chunk.state, token).await?);
        }
        ensure!(
            executor.mtp_next(&chunk.state, token).await.is_err(),
            "combined pressure admission"
        );
        ensure!(
            bits(&parent, &executor.mtp_snapshot(&chunk.state).await?),
            "pressure corrupted parent"
        );
        drop(pressure);
        let before = executor.stats().await?.residents;
        {
            let mut pending = Box::pin(executor.mtp_next(&chunk.state, token));
            let waiting =
                std::future::poll_fn(|cx| Poll::Ready(pending.as_mut().poll(cx).is_pending()))
                    .await;
            ensure!(waiting, "expected queued native completion");
        }
        ensure!(
            executor.stats().await?.residents == before,
            "cancelled draft residency leaked"
        );
        let recovered = executor.mtp_next(&chunk.state, token).await?;
        ensure!(
            bits(&a.logits, &recovered.logits),
            "recovery changed branch"
        );
        ensure!(
            bits(&target_parent, &executor.snapshot(&target.state).await?),
            "MTP changed target parent"
        );
        cases.push(serde_json::json!({"count":count,"logit_relative_l2":logit_drift,"kv_relative_l2":kv_drift,"hidden_relative_l2":hidden_drift,"draft_token":token}));
        drop(recovered);
        drop(a);
        drop(b);
        drop(chunk);
        drop(serial);
        drop(target);
        ensure!(
            executor.stats().await?.residents == 0,
            "MTP final residency"
        );
    }
    // Exercise the separate draft cache through its full 512-position bound.
    let mut target = executor.empty_state();
    let mut draft = executor.empty_mtp_state()?;
    for _ in 0..64 {
        let next = executor.hidden_block(&target, vec![1; 8]).await?;
        target = next.state;
        draft = executor
            .mtp_prefill(&draft, &target, 0, vec![1; 8])
            .await?
            .state;
    }
    ensure!(
        draft.position() == 512 && executor.mtp_next(&draft, 1).await.is_err(),
        "MTP context bound"
    );
    drop(draft);
    drop(target);
    ensure!(executor.stats().await?.residents == 0, "MTP context drain");
    let stats = executor.shutdown().await?;
    let report = serde_json::json!({"pass":true,"graph_mode":graph,"layer_group":group,"scalar_debug_probes":cfg!(feature="debug-validation"),"cases":cases,"context":512,"transfer_accounting":"session ABI counters; explicit debug scalar snapshots are not counted","branch_bitwise":true,"combined_budget":true,"cancellation_drained":true,"logical_bytes_at_shutdown":stats.logical_bytes,"speculative_serving_qualified":false});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
