//! Real Combined MTP through the shared Engine's bounded token streams.
use anyhow::{ensure, Context, Result};
use rust_server::{
    cache::*,
    serving::{
        config::Config as ServingConfig,
        native_mlx::Device,
        native_qwen::{QwenDeviceExecutor, ScheduledMtp},
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
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_scheduler_smoke NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let prompt = tokenizer.chat(None, "Complete: 1, 2, 3,")?;
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
    let mut chunks = prompt.chunks(8);
    let mut baseline = executor
        .greedy_start(chunks.next().unwrap().to_vec())
        .await?;
    for chunk in chunks {
        baseline = executor.greedy_extend(&baseline, chunk.to_vec()).await?;
    }
    let mut expected = Vec::new();
    for _ in 0..16 {
        let tx = executor.greedy_verify(&baseline, vec![], &eos).await?;
        expected.extend(tx.emitted);
        baseline = tx.cursor;
    }
    drop(baseline);
    let audit_controller = controller()?;
    let backend = ScheduledMtp::new(
        audit_controller.clone(),
        executor.clone(),
        eos.clone(),
        4,
        "scheduler-mtp".into(),
    )?
    .with_exclusive_metadata_audits();
    let engine = Engine::new(config(), backend.storage(), backend)?;
    let mut jobs = Vec::new();
    for cap in [1, 8, 16] {
        jobs.push((
            cap,
            tokio::spawn(collect(engine.submit_stream(prompt.clone(), cap, 1)?)),
        ));
    }
    let mut generations = Vec::new();
    for (cap, job) in jobs {
        let result = job.await??;
        ensure!(
            result.tokens == expected[..cap],
            "scheduler/target output budget {cap}"
        );
        generations.push(result);
    }
    let stats = engine.shutdown().await?;
    let audit_epochs = audit_controller.stats()?.epoch;
    ensure!(
        audit_epochs == u64::try_from(stats.iterations)?,
        "one audit per validated scheduler batch"
    );
    ensure!(
        stats.completed == 3 && stats.memory_used == 0 && !stats.admission_closed,
        "scheduler stream drain"
    );
    let backend = ScheduledMtp::new(
        controller()?,
        executor.clone(),
        eos,
        4,
        "scheduler-mtp-cancel".into(),
    )?
    .with_exclusive_metadata_audits();
    let engine = Engine::new(config(), backend.storage(), backend)?;
    let mut slow = engine.submit_stream(prompt.clone(), 32, 1)?;
    ensure!(slow.next_token().await.is_some(), "slow first token");
    let fast = engine.submit(prompt, 8)?.result().await?;
    ensure!(
        fast.tokens == expected[..8],
        "backpressured real lane fairness"
    );
    drop(slow);
    let canceled = engine.shutdown().await?;
    ensure!(
        canceled.completed == 1 && canceled.cancelled == 1 && canceled.memory_used == 0,
        "stream cancellation"
    );
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("scheduler retained executor"))?
        .shutdown()
        .await?;
    ensure!(native.logical_bytes == 0, "native state drain");
    let report = serde_json::json!({"pass":true,"lanes":3,"iteration_scheduling":true,"physical_tensor_batching":false,"http_streaming":false,"output_limits":[1,8,16],"generations":generations,"stats":stats,"cancellation":canceled,"shutdown_bytes":native.logical_bytes,"throughput_claim":false,"audit_epochs":audit_epochs,"audit_clock":"validated-batches"});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
