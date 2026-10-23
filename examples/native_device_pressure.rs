//! Hardware lifetime/cancellation/context pressure. No model-quality promotion.
use anyhow::{ensure, Result};
use rust_server::serving::{native_mlx::Device, native_qwen::QwenDeviceExecutor};
use std::{future::Future, path::Path, task::Poll};
fn drift(a: &[f32], b: &[f32]) -> Result<(f64, f64)> {
    ensure!(
        !a.is_empty() && a.len() == b.len() && a.iter().chain(b).all(|x| x.is_finite()),
        "state comparison finite/shape"
    );
    let mut d = 0.;
    let mut r = 0.;
    let mut max = 0f64;
    for (&x, &y) in a.iter().zip(b) {
        let delta = x as f64 - y as f64;
        d += delta * delta;
        r += (x as f64).powi(2);
        max = max.max(delta.abs());
    }
    Ok(((d / r.max(1e-30)).sqrt(), max))
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        (1..=3).contains(&args.len()),
        "native_device_pressure NEW_DIRECTORY [GROUP] [reference|fused]"
    );
    let group: u32 = args.get(1).map_or(Ok(1), |s| s.parse())?;
    let mode = args.get(2).map_or("reference", String::as_str);
    ensure!(
        matches!(mode, "reference" | "fused"),
        "recurrent kernel mode"
    );
    let out = Path::new(&args[0]);
    std::fs::create_dir(out)?;
    let executor =
        QwenDeviceExecutor::open_tuned(Device::Gpu, 8, 768 * 1024 * 1024, group, mode == "fused")
            .await?;
    let prefix = executor
        .step(&executor.empty_state(), vec![16; 8])
        .await?
        .state;
    let mut branches = Vec::new();
    for i in 0..7 {
        branches.push(executor.step(&prefix, vec![17 + i]).await?);
    }
    ensure!(
        executor.stats().await?.residents == 8,
        "resident slot pressure"
    );
    ensure!(
        executor.step(&prefix, vec![18]).await.is_err(),
        "slot exhaustion must reject"
    );
    ensure!(
        executor.stats().await?.residents == 8,
        "failed reservation leaked"
    );
    drop(branches);
    ensure!(
        executor.stats().await?.residents == 1,
        "branches failed to retire"
    );
    let repeat = executor.step(&prefix, vec![17]).await?;
    let sibling = executor.step(&prefix, vec![18, 19]).await?;
    let again = executor.step(&prefix, vec![17]).await?;
    ensure!(repeat.logits == again.logits, "branch isolation logits");
    let stats = executor.stats().await?;
    ensure!(
        stats.exported_bytes == 0 && stats.uploaded_bytes == 0,
        "hot path copied full state"
    );
    let snapshot = executor.snapshot(&prefix).await?;
    let mut cancelled = Box::pin(executor.step(&prefix, vec![20]));
    let pending = std::future::poll_fn(|cx| {
        Poll::Ready(matches!(cancelled.as_mut().poll(cx), Poll::Pending))
    })
    .await;
    ensure!(
        pending,
        "cancellation probe did not reach pending completion"
    );
    drop(cancelled);
    ensure!(
        executor.stats().await?.residents == 4,
        "single cancelled receipt failed to drain before queue pressure"
    );
    // Three admitted native jobs, including a cancelled queued reader/output.
    let mut first = Box::pin(executor.step(&prefix, vec![21]));
    let mut queued = Box::pin(executor.step(&prefix, vec![22]));
    let mut third = Box::pin(executor.step(&prefix, vec![23]));
    let all_pending = std::future::poll_fn(|cx| {
        let a = matches!(first.as_mut().poll(cx), Poll::Pending);
        let b = matches!(queued.as_mut().poll(cx), Poll::Pending);
        let c = matches!(third.as_mut().poll(cx), Poll::Pending);
        Poll::Ready(a && b && c)
    })
    .await;
    ensure!(
        all_pending,
        "three-job pressure did not reach pending completion"
    );
    ensure!(
        executor.step(&prefix, vec![24]).await.is_err(),
        "fourth admitted step exceeded semaphore"
    );
    drop(queued);
    let (first, third) = tokio::join!(first, third);
    let (first, third) = (first?, third?);
    ensure!(
        first.state.position() == 9 && third.state.position() == 9,
        "queued siblings lost state"
    );
    drop((first, third));
    let after = executor.stats().await?;
    ensure!(after.residents == 4, "cancelled output failed retirement");
    ensure!(
        snapshot == executor.snapshot(&prefix).await?,
        "cancel/branch mutated shared prefix"
    );
    ensure!(
        executor.snapshot(&repeat.state).await? == executor.snapshot(&again.state).await?,
        "branch isolation state"
    );
    drop((repeat, sibling, again, prefix));
    ensure!(executor.stats().await?.residents == 0, "branch drain");
    let mut state = executor.empty_state();
    for _ in 0..63 {
        state = executor.step(&state, vec![16; 8]).await?.state;
    }
    ensure!(state.position() == 504, "continuation position");
    let chunk = executor.step(&state, vec![17; 8]).await?;
    let mut single = state.clone();
    let mut logits = Vec::new();
    for _ in 0..8 {
        let step = executor.step(&single, vec![17]).await?;
        single = step.state;
        logits = step.logits;
    }
    ensure!(
        chunk.state.position() == 512
            && single.position() == 512
            && chunk.state.bytes() == 86_835_200,
        "context 512 state geometry"
    );
    let logit_drift = drift(&chunk.logits, &logits)?;
    let state_drift = drift(
        &executor.snapshot(&chunk.state).await?,
        &executor.snapshot(&single).await?,
    )?;
    ensure!(
        logit_drift.0 <= 1e-5 && state_drift.0 <= 1e-5,
        "context chunk/single parity {logit_drift:?} {state_drift:?}"
    );
    ensure!(
        executor.step(&chunk.state, vec![16]).await.is_err(),
        "context overflow accepted"
    );
    drop((state, chunk));
    let survivor = single;
    let before = executor.stats().await?;
    ensure!(before.residents == 1, "live reference accounting");
    let shutdown = executor.shutdown().await?;
    ensure!(
        shutdown.residents == 0
            && shutdown.logical_bytes == 0
            && shutdown.invalidated_at_shutdown == 1,
        "shutdown did not invalidate live version"
    );
    let restarted =
        QwenDeviceExecutor::open_tuned(Device::Gpu, 2, 1, group, mode == "fused").await?;
    ensure!(
        restarted.step(&survivor, vec![16]).await.is_err(),
        "foreign epoch accepted after restart"
    );
    drop(survivor); // Safe after original worker and native session destruction.
    ensure!(
        restarted
            .step(&restarted.empty_state(), vec![16])
            .await
            .is_err(),
        "byte exhaustion accepted"
    );
    let final_stats = restarted.shutdown().await?;
    ensure!(
        final_stats.residents == 0 && final_stats.steps == 0,
        "restart drain"
    );
    let report = serde_json::json!({"pass":true,"layer_group":group,"recurrent_kernel":mode,"slot_exhaustion":true,"recovery":true,"branch_isolation":true,"pending_receipt_cancel":true,"three_admitted_queue_cancel":true,"fourth_step_rejected":true,"context":512,"chunk_single_logit_drift":logit_drift,"chunk_single_state_drift":state_drift,"hot_path_exported_bytes":stats.exported_bytes,"hot_path_uploaded_bytes":stats.uploaded_bytes,"explicit_diagnostic_exported_bytes":before.exported_bytes,"shutdown_invalidated":shutdown.invalidated_at_shutdown,"restart_foreign_epoch_rejected":true,"mlx_peak_bytes_process":before.mlx_peak_bytes,"physical_command_buffer_cap_claim":false});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
