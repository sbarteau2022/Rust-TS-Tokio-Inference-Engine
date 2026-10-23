//! Real Combined MTP through the shared Engine's bounded token streams.
use anyhow::{ensure, Context, Result};
use rust_server::{
    cache::*,
    serving::{
        config::Config as ServingConfig,
        native_mlx::Device,
        native_qwen::{AutomatonMtp, QwenDeviceExecutor, ScheduledMtp},
        native_tokenizer::NativeTokenizer,
        scheduler::{Engine, Request},
        weights::Manifest,
    },
};
use std::{path::Path, sync::Arc};
fn controller() -> Result<Controller> {
    let c = Controller::new(serde_json::from_str(include_str!(
        "../fixtures/cache-config-bench.json"
    ))?)?;
    c.configure_drafts(DraftPolicy {
        max_proposers: 4,
        work_limit: 3,
        initial_mass: MAX_MASS,
        admission_mass: MAX_MASS / 8,
        reward_mass: MAX_MASS / 4,
        qualification_trials: 64,
        probe_every_epochs: 2,
    })?;
    Ok(c)
}
fn config() -> ServingConfig {
    ServingConfig {
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
async fn collect(mut request: Request) -> Result<rust_server::serving::scheduler::Generation> {
    let mut events = Vec::new();
    while let Some(event) = request.next_token().await {
        ensure!(event.index == events.len(), "stream index/order");
        events.push(event);
    }
    let generation = request.result().await?;
    ensure!(
        generation.tokens == events.iter().map(|e| e.token).collect::<Vec<_>>()
            && generation.token_times_ms == events.iter().map(|e| e.elapsed_ms).collect::<Vec<_>>(),
        "stream/final congruence"
    );
    Ok(generation)
}

fn bits(a: &[f32], b: &[f32]) -> Result<()> {
    ensure!(
        !a.is_empty()
            && a.len() == b.len()
            && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits()),
        "native state bitwise equality"
    );
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_seed_qualification NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let refs: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        "evidence/native-mtp-stream-matrix/raw/references.json",
    )?)?;
    let texts: Vec<_> = include_str!("../fixtures/native-speculative-prompts.tsv")
        .lines()
        .map(|s| s.split_once('\t').unwrap().1)
        .collect();
    let prompts: Vec<_> = texts
        .iter()
        .map(|s| tokenizer.chat(None, s))
        .collect::<Result<_>>()?;
    let expected: Vec<Vec<u32>> = serde_json::from_value(refs["tokens"].clone())?;
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
    ensure!(
        refs["identity"].as_str() == Some(executor.identity()),
        "model identity"
    );
    // Native primitive state gate: reading the seed cannot change either target or draft state.
    for (i, prompt) in prompts.iter().enumerate() {
        let lane =
            AutomatonMtp::register(controller()?, executor.clone(), format!("seed-state-{i}"))?;
        let eager = lane.start(prompt.clone()).await?;
        let delayed = lane.start(prompt.clone()).await?;
        let seed = delayed.target().next_greedy_token()?;
        ensure!(seed == expected[i][0], "prefill seed reference");
        let a = lane.no_draft(&eager, &eos).await?;
        let b = lane.no_draft(&delayed, &eos).await?;
        ensure!(
            a.emitted == [seed]
                && b.emitted == [seed]
                && a.cursor.committed() == b.cursor.committed(),
            "seed exactly once"
        );
        bits(a.cursor.target().logits(), b.cursor.target().logits())?;
        bits(
            &executor.snapshot(a.cursor.target().state()).await?,
            &executor.snapshot(b.cursor.target().state()).await?,
        )?;
        bits(
            &executor.mtp_snapshot(a.cursor.draft_state()).await?,
            &executor.mtp_snapshot(b.cursor.draft_state()).await?,
        )?;
    }
    let mut cells = Vec::new();
    for round in 0..2 {
        for lanes in [1, 3] {
            for deferred in if round == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let c = controller()?;
                let mut backend = ScheduledMtp::new(
                    c.clone(),
                    executor.clone(),
                    eos.clone(),
                    4,
                    format!("seed-{round}-{lanes}-{deferred}"),
                )?
                .with_exclusive_metadata_audits();
                if deferred {
                    backend = backend.with_deferred_seed()?;
                }
                let metrics = backend.seed_metrics();
                let engine = Engine::new(config(), backend.storage(), backend)?;
                let cases: Vec<_> = (0..prompts.len())
                    .flat_map(|i| [1, 2, 4, 8, 64].map(move |cap| (i, cap)))
                    .collect();
                let mut rows = Vec::new();
                let origin = std::time::Instant::now();
                for wave in cases.chunks(lanes) {
                    let mut jobs = Vec::new();
                    for &(i, cap) in wave {
                        jobs.push((
                            i,
                            cap,
                            tokio::spawn(collect(engine.submit_stream(
                                prompts[i].clone(),
                                cap,
                                1,
                            )?)),
                        ));
                    }
                    for (i, cap, job) in jobs {
                        let generation = job.await??;
                        ensure!(
                            generation.tokens == expected[i][..cap],
                            "eager/deferred reference"
                        );
                        rows.push(
                            serde_json::json!({"prompt":i,"cap":cap,"generation":generation}),
                        );
                    }
                }
                let elapsed = origin.elapsed().as_secs_f64();
                let stats = engine.shutdown().await?;
                ensure!(
                    stats.memory_used == 0 && stats.failed == 0 && stats.completed == 20,
                    "seed engine drain"
                );
                let snapshot = metrics.snapshot();
                if deferred {
                    ensure!(
                        snapshot.prefill_seed_receipts == 20
                            && snapshot.advance_attempts == 16
                            && snapshot.advance_completed == 16
                            && snapshot.pending_releases == 4,
                        "seed counters"
                    );
                }
                cells.push(serde_json::json!({"round":round,"lanes":lanes,"deferred":deferred,"wall_seconds":elapsed,"tokens":316,"rows":rows,"stats":stats,"seed_metrics":snapshot}));
                std::fs::write(
                    Path::new(&out).join("partial.json"),
                    serde_json::to_vec_pretty(&cells)?,
                )?;
            }
        }
    }
    // EOS and post-seed cancellation through the actual shared Engine.
    for deferred in [false, true] {
        let mut backend = ScheduledMtp::new(
            controller()?,
            executor.clone(),
            vec![expected[0][0]],
            4,
            format!("seed-eos-{deferred}"),
        )?;
        if deferred {
            backend = backend.with_deferred_seed()?;
        }
        let metrics = backend.seed_metrics();
        let engine = Engine::new(config(), backend.storage(), backend)?;
        let g = engine.submit(prompts[0].clone(), 8)?.result().await?;
        ensure!(
            g.tokens == expected[0][..1] && g.finish_reason == "eos",
            "seed EOS"
        );
        ensure!(engine.shutdown().await?.memory_used == 0, "EOS drain");
        if deferred {
            ensure!(
                metrics.snapshot().advance_attempts == 0
                    && metrics.snapshot().pending_releases == 1,
                "EOS unnecessary maintenance"
            );
        }
    }
    let backend = ScheduledMtp::new(
        controller()?,
        executor.clone(),
        eos,
        4,
        "seed-cancel".into(),
    )?
    .with_deferred_seed()?;
    let metrics = backend.seed_metrics();
    let engine = Engine::new(config(), backend.storage(), backend)?;
    let mut requests = Vec::new();
    for prompt in &prompts[..3] {
        requests.push(engine.submit_stream(prompt.clone(), 64, 1)?);
    }
    for (i, request) in requests.iter_mut().enumerate() {
        ensure!(
            request.next_token().await.context("cancel seed")?.token == expected[i][0],
            "cancel first token"
        );
    }
    drop(requests);
    let canceled = engine.shutdown().await?;
    ensure!(
        canceled.cancelled == 3 && canceled.failed == 0 && canceled.memory_used == 0,
        "post-seed cancellation drain"
    );
    let cancellation = metrics.snapshot();
    ensure!(
        cancellation.prefill_seed_receipts == 3
            && cancellation.advance_attempts == cancellation.advance_completed
            && cancellation.advance_completed + cancellation.pending_releases == 3,
        "cancel maintenance accounting"
    );
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("executor retained"))?
        .shutdown()
        .await?;
    ensure!(native.logical_bytes == 0, "native seed drain");
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"pass":true,"scope":"native state parity and alternating mixed-length Rust publication pilot; no HTTP speed claim","cells":cells,"cancellation":canceled,"cancellation_seed_metrics":cancellation,"native_bytes":native.logical_bytes,"native_state_bitwise_prompts":4}),
        )?,
    )?;
    println!("PASS seed qualification: bitwise native target/draft state, 160 mixed-length streams, EOS, post-seed cancellation, drained native state");
    Ok(())
}
