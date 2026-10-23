//! Rust Engine delivery matrix; no HTTP or physical tensor batching.
use anyhow::{ensure, Context, Result};
use rust_server::{
    cache::{Config as CacheConfig, Controller, DraftPolicy, MAX_MASS},
    serving::{
        config::Config,
        native_mlx::Device,
        native_qwen::{QwenDeviceExecutor, ScheduledDevice, ScheduledMtp, StateMode},
        native_tokenizer::NativeTokenizer,
        scheduler::{Engine, Generation, Request},
        weights::Manifest,
    },
};
use serde_json::{json, Value};
use std::{path::Path, sync::Arc, time::Instant};
fn config() -> Config {
    Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 511,
        max_new_tokens: 128,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    }
}
fn controller() -> Result<Controller> {
    let c = Controller::new(CacheConfig {
        capacity_blocks: 8,
        bytes_per_block: 64,
        max_branches: 64,
        max_declared_tokens: 4096,
        max_leases: 16,
        anchor_fraction_q31: MAX_MASS / 2 + 1,
        dead_threshold: 2,
        rates_q31: [268435456, 1073741824, MAX_MASS],
        pressure_multipliers_q31: [2147483648, 4294967296, 8589934592],
        queue_depth: 4,
        max_wait_epochs: 3,
    })?;
    c.configure_drafts(DraftPolicy {
        max_proposers: 1,
        work_limit: 3,
        initial_mass: MAX_MASS,
        admission_mass: MAX_MASS / 8,
        reward_mass: MAX_MASS / 4,
        qualification_trials: 64,
        probe_every_epochs: 2,
    })?;
    Ok(c)
}
fn quantile(xs: &[f64], q: f64) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    let mut v = xs.to_vec();
    v.sort_by(f64::total_cmp);
    Some(v[((v.len() - 1) as f64 * q).ceil() as usize])
}
fn distribution(v: &[f64]) -> Value {
    json!({"count":v.len(),"mean":if v.is_empty(){None}else{Some(v.iter().sum::<f64>()/v.len() as f64)},"p50":quantile(v,0.5),"p95":quantile(v,0.95),"max":quantile(v,1.0)})
}
async fn collect(mut r: Request, origin: Instant) -> Result<(Generation, Vec<f64>)> {
    let mut events = Vec::new();
    let mut times = Vec::new();
    let mut publication = Vec::new();
    while let Some(e) = r.next_token().await {
        ensure!(e.index == events.len(), "event order");
        events.push(e.token);
        publication.push(e.elapsed_ms);
        times.push(origin.elapsed().as_secs_f64() * 1000.);
    }
    let g = r.result().await?;
    ensure!(
        g.tokens == events && g.token_times_ms == publication,
        "stream/final mismatch"
    );
    Ok((g, times))
}
async fn reference(e: &QwenDeviceExecutor, prompt: &[u32], eos: &[u32]) -> Result<Vec<u32>> {
    let mut chunks = prompt.chunks(8);
    let mut c = e
        .greedy_start(chunks.next().context("empty prompt")?.to_vec())
        .await?;
    for t in chunks {
        c = e.greedy_extend(&c, t.to_vec()).await?;
    }
    let mut ids = Vec::new();
    while ids.len() < 64 && !c.terminal() {
        let tx = e.greedy_verify(&c, vec![], eos).await?;
        ids.extend(tx.emitted);
        c = tx.cursor;
    }
    ensure!(!ids.is_empty(), "empty reference");
    Ok(ids)
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_scheduler_bench NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let mut prompts = Vec::new();
    for line in include_str!("../fixtures/native-speculative-prompts.tsv").lines() {
        let (name, text) = line.split_once('\t').context("fixture")?;
        let ids = tokenizer.chat(None, text)?;
        ensure!(ids.len() + 64 <= 512, "prompt context");
        prompts.push((name.to_owned(), ids));
    }
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
    let mut refs = Vec::new();
    for (_, p) in &prompts {
        refs.push(reference(&executor, p, &eos).await?);
    }
    std::fs::write(
        Path::new(&out).join("references.json"),
        serde_json::to_vec_pretty(
            &json!({"identity":executor.identity(),"prompts":prompts,"tokens":refs}),
        )?,
    )?;
    let mut cells = Vec::new();
    for round in 0..2 {
        for lanes in [1, 3] {
            for profile in if round == 0 { [0, 1, 2] } else { [2, 1, 0] } {
                let name = ["target-only", "mtp-no-live-audit", "mtp-live-audit"][profile];
                let c = controller()?;
                let (engine, id) = if profile == 0 {
                    let b = ScheduledDevice::new(executor.clone(), eos.clone(), StateMode::Device);
                    (Engine::new(config(), b.storage(), b)?, None)
                } else {
                    let mut b = ScheduledMtp::new(
                        c.clone(),
                        executor.clone(),
                        eos.clone(),
                        4,
                        format!("bench-{round}-{lanes}-{profile}"),
                    )?;
                    let id = b.proposer_id();
                    if profile == 2 {
                        b = b.with_exclusive_metadata_audits();
                    }
                    (Engine::new(config(), b.storage(), b)?, Some(id))
                };
                // Warmup is kept separate and subtracted from the cumulative acceptance ledger.
                let warm = engine.submit(prompts[0].1.clone(), 8)?.result().await?;
                ensure!(warm.tokens == refs[0][..warm.tokens.len()], "warmup parity");
                let before = id.map(|id| c.draft_ledger(id)).transpose()?;
                let epoch_before = c.stats()?.epoch;
                let origin = Instant::now();
                let mut delivered = 0;
                let mut ordinal = 0;
                let mut rows = Vec::new();
                let mut receive_all = Vec::new();
                let mut ttft = Vec::new();
                let mut publication_itl = Vec::new();
                let mut receiver_itl = Vec::new();
                let mut decode_seconds = 0.;
                let mut decode_tokens = 0;
                while delivered < 2000 {
                    let mut jobs = Vec::new();
                    let mut reserved = 0;
                    for _ in 0..lanes {
                        let budget = 64.min(2000 - delivered - reserved);
                        if budget == 0 {
                            break;
                        }
                        let prompt = ordinal % prompts.len();
                        ordinal += 1;
                        reserved += budget;
                        let req = engine.submit_stream(prompts[prompt].1.clone(), budget, 1)?;
                        jobs.push((prompt, budget, tokio::spawn(collect(req, origin))));
                    }
                    for (prompt, budget, job) in jobs {
                        let (g, times) = job.await??;
                        ensure!(
                            !g.tokens.is_empty()
                                && g.tokens.len() <= budget
                                && g.tokens == refs[prompt][..g.tokens.len()],
                            "measured output parity"
                        );
                        delivered += g.tokens.len();
                        ttft.push(g.token_times_ms[0]);
                        publication_itl.extend(g.token_times_ms.windows(2).map(|v| v[1] - v[0]));
                        receiver_itl.extend(times.windows(2).map(|v| v[1] - v[0]));
                        if g.tokens.len() > 1 {
                            decode_tokens += g.tokens.len() - 1;
                            decode_seconds += (g.token_times_ms[g.tokens.len() - 1]
                                - g.token_times_ms[0])
                                / 1000.;
                        }
                        receive_all.extend_from_slice(&times);
                        rows.push(json!({"prompt":prompts[prompt].0,"requested":budget,"generation":g,"receiver_times_ms":times}));
                    }
                }
                let seconds = origin.elapsed().as_secs_f64();
                receive_all.sort_by(f64::total_cmp);
                let windows:Vec<_>=[1000,1500,2000].into_iter().map(|n|json!({"tokens":n,"elapsed_ms":receive_all[n-1],"aggregate_tokens_per_second":n as f64*1000./receive_all[n-1]})).collect();
                let ledger = id.map(|id| c.draft_ledger(id)).transpose()?;
                let acceptance = match (&before, &ledger) {
                    (Some(a), Some(b)) => {
                        let proposed = b.proposed - a.proposed;
                        let accepted = b.accepted - a.accepted;
                        json!({"learned_proposed":proposed,"learned_accepted":accepted,"fraction":if proposed>0{Some(accepted as f64/proposed as f64)}else{None},"speculative_target_calls_only":b.target_calls-a.target_calls,"known_seed_attempts":b.known_prefix-a.known_prefix,"failures":b.failures-a.failures,"unfinished":b.unfinished})
                    }
                    _ => Value::Null,
                };
                let epochs = c.stats()?.epoch - epoch_before;
                let stats = engine.shutdown().await?;
                ensure!(
                    stats.memory_used == 0
                        && !stats.admission_closed
                        && stats.failed == 0
                        && stats.cancelled == 0,
                    "engine drain"
                );
                let native = executor.stats().await?;
                ensure!(
                    native.residents == 0 && native.logical_bytes == 0,
                    "native drain"
                );
                let cell = json!({"round":round,"lanes":lanes,"profile":name,"tokens":delivered,"wall_seconds":seconds,"aggregate_tokens_per_second":delivered as f64/seconds,"ttft_ms":distribution(&ttft),"publication_itl_ms":distribution(&publication_itl),"receiver_itl_ms":distribution(&receiver_itl),"post_first_tokens_per_lane_second":decode_tokens as f64/decode_seconds,"acceptance":acceptance,"audit_epochs":epochs,"warmup":warm,"ledger_before":before,"ledger_after":ledger,"windows":windows,"stats":stats,"requests":rows});
                std::fs::write(
                    Path::new(&out).join(format!("round-{round}-lanes-{lanes}-{name}.json")),
                    serde_json::to_vec_pretty(&cell)?,
                )?;
                println!(
                    "CELL round={round} lanes={lanes} profile={name} tok_s={:.3} epochs={epochs}",
                    delivered as f64 / seconds
                );
                cells.push(cell);
            }
        }
    }
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("executor retained"))?
        .shutdown()
        .await?;
    ensure!(native.logical_bytes == 0, "final drain");
    let report = json!({"pass":true,"scope":"Rust Engine bounded streams; no HTTP, physical tensor batching, isolation or broad quality qualification","draft_width":4,"stream_capacity":1,"request_budget":64,"policy":"qualification=64 probe_epochs=2 initial=2147483647 admission=268435455 reward=536870911; fixed CLI scaffold","cells":cells});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metrics_empty_and_nearest_rank() {
        assert_eq!(quantile(&[], 0.5), None);
        assert_eq!(quantile(&[4., 1., 2., 3.], 0.5), Some(3.));
        assert_eq!(quantile(&[4., 1., 2., 3.], 0.95), Some(4.));
        assert!(distribution(&[])["mean"].is_null());
    }
}
