//! Offline scheduler workload smoke, not an HTTP load test or agent quality benchmark.
use super::{
    config::Config,
    native_mlx::Device,
    native_qwen::{QwenDeviceExecutor, ScheduledDevice, StateMode},
    native_tokenizer::NativeTokenizer,
    scheduler::Engine,
    weights::Manifest,
};
use anyhow::{ensure, Result};
use std::{collections::BTreeMap, io::Write, path::Path, sync::Arc, time::Instant};
fn percentile(mut values: Vec<f64>, q: f64) -> Result<f64> {
    ensure!(
        !values.is_empty()
            && values.iter().all(|v| v.is_finite() && *v >= 0.)
            && (0. ..=1.).contains(&q),
        "invalid latency samples"
    );
    values.sort_by(f64::total_cmp);
    Ok(values[((values.len() as f64 * q).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1)])
}
pub async fn run(out: &Path) -> Result<()> {
    std::fs::create_dir(out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let executor = Arc::new(QwenDeviceExecutor::open(Device::Gpu, 8, 768 * 1024 * 1024).await?);
    let prompts=["Write a short Rust function that sums a slice of i32 values using i64 accumulation. Code only.","List three concrete security checks for an HTTP file-upload endpoint. Be concise.","Explain why dropping a Tokio JoinHandle differs from cancelling the task. Use two sentences."];
    let mut baseline = BTreeMap::new();
    let mut reports = Vec::new();
    let mut requests_log = std::fs::File::create(out.join("requests.jsonl"))?;
    for mode in [StateMode::Snapshot, StateMode::Device] {
        for concurrency in [1, 3] {
            let backend = ScheduledDevice::new(executor.clone(), eos.clone(), mode);
            let storage = backend.storage();
            let observer = backend.drain.clone();
            let engine = Engine::new(
                Config {
                    version: 1,
                    max_queued: 3,
                    max_sequences: 3,
                    max_batch_tokens: 24,
                    prefill_chunk_tokens: 8,
                    max_context_tokens: 512,
                    max_new_tokens: 64,
                    model_memory_bytes: 1024 * 1024 * 1024,
                    max_in_flight: 3,
                },
                storage,
                backend,
            )?;
            let warm = engine
                .submit(
                    tokenizer.chat(None, "Reply with exactly READY and nothing else.")?,
                    8,
                )?
                .result()
                .await?;
            ensure!(
                warm.finish_reason == "eos" && tokenizer.decode(&warm.tokens)?.trim() == "READY",
                "warmup smoke failed"
            );
            let before = executor.stats().await?;
            let started = Instant::now();
            let mut first = Vec::new();
            let mut decode = Vec::new();
            let mut latency = Vec::new();
            let mut tokens = 0;
            let mut non_eos = 0;
            for group in (0..6usize).collect::<Vec<_>>().chunks(concurrency) {
                let mut pending = Vec::new();
                for &id in group {
                    pending.push((
                        id,
                        engine.submit(tokenizer.chat(None, prompts[id % 3])?, 48)?,
                    ));
                }
                for (id, request) in pending {
                    let result = request.result().await?;
                    ensure!(
                        result.tokens.len() == result.token_times_ms.len()
                            && !result.tokens.is_empty()
                            && result.token_times_ms.windows(2).all(|w| w[1] >= w[0])
                            && result.elapsed_ms >= *result.token_times_ms.last().unwrap(),
                        "scheduler timing accounting"
                    );
                    if mode == StateMode::Snapshot && concurrency == 1 {
                        baseline.insert(id, result.tokens.clone());
                    } else {
                        ensure!(baseline.get(&id)==Some(&result.tokens),"scheduler trajectory changed mode={mode:?} concurrency={concurrency} id={id}");
                    }
                    first.push(result.token_times_ms[0]);
                    decode.extend(result.token_times_ms.windows(2).map(|w| w[1] - w[0]));
                    latency.push(result.elapsed_ms);
                    tokens += result.tokens.len();
                    non_eos += result.tokens.iter().filter(|t| !eos.contains(t)).count();
                    writeln!(
                        requests_log,
                        "{}",
                        serde_json::json!({"mode":format!("{mode:?}"),"concurrency":concurrency,"id":id,"prompt":prompts[id%3],"text":tokenizer.decode(&result.tokens)?,"result":result})
                    )?;
                    requests_log.flush()?;
                }
            }
            let seconds = started.elapsed().as_secs_f64();
            let scheduler = engine.shutdown().await?;
            ensure!(
                scheduler.memory_used == 0
                    && !scheduler.admission_closed
                    && scheduler.failed == 0
                    && scheduler.completed == 7,
                "scheduler smoke drain"
            );
            let after = observer
                .lock()
                .map_err(|_| anyhow::anyhow!("drain observer poisoned"))?
                .context("missing native drain")?;
            ensure!(
                after.residents == 0
                    && after.logical_bytes == 0
                    && after.exported_bytes == before.exported_bytes
                    && after.uploaded_bytes == before.uploaded_bytes,
                "native smoke state-copy/drain"
            );
            let report = serde_json::json!({"mode":format!("{mode:?}"),"concurrency":concurrency,"requests":6,"max_new":48,"seconds":seconds,"tokens_including_eos":tokens,"tokens_excluding_eos":non_eos,"tokens_per_second_including_eos":tokens as f64/seconds,"tokens_per_second_excluding_eos":non_eos as f64/seconds,"ttft_p50_ms":percentile(first.clone(),0.5)?,"ttft_p95_ms":percentile(first,0.95)?,"inter_token_p50_ms":percentile(decode.clone(),0.5)?,"inter_token_p95_ms":percentile(decode,0.95)?,"request_p50_ms":percentile(latency.clone(),0.5)?,"request_p95_ms":percentile(latency,0.95)?,"scheduler_logical_peak_bytes":scheduler.memory_peak,"mlx_peak_bytes_process":after.mlx_peak_bytes,"device_state_exported_bytes":after.exported_bytes-before.exported_bytes,"device_state_uploaded_bytes":after.uploaded_bytes-before.uploaded_bytes,"native_residents_after_window":after.residents});
            eprintln!("SCHEDULER-SMOKE {report}");
            reports.push(report);
        }
    }
    let executor =
        Arc::try_unwrap(executor).map_err(|_| anyhow::anyhow!("scheduler retained executor"))?;
    let drain = executor.shutdown().await?;
    ensure!(
        drain.residents == 0 && drain.invalidated_at_shutdown == 0,
        "final smoke drain"
    );
    let report = serde_json::json!({"identity":manifest.identity,"profile":"fp32-explicit-attention-regular-kv","pass":true,"windows":reports,"token_trajectory_parity":true,"load_and_warmup_excluded":true,"quantiles":"nearest-rank","native_calls":"serial rows; no tensor batching","scope":"offline Rust scheduler smoke; no HTTP transport or autonomous-agent quality claim","held_out_qualified":false,"physical_command_buffer_cap_claim":false});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
use anyhow::Context;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quantiles_and_invalid_observations() {
        assert_eq!(percentile(vec![3., 1., 2.], 0.5).unwrap(), 2.);
        assert_eq!(percentile(vec![3., 1., 2.], 0.95).unwrap(), 3.);
        for values in [vec![], vec![f64::NAN], vec![-1.]] {
            assert!(percentile(values, 0.5).is_err());
        }
        assert!(percentile(vec![1.], 2.).is_err());
    }
}
