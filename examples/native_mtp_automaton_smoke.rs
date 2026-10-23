//! Combined-only Spec 15 MTP admission, accounting, cancellation and pressure gate.
use anyhow::{ensure, Context, Result};
use rust_server::{
    cache::*,
    serving::{
        config::Config as ServingConfig,
        native_mlx::Device,
        native_qwen::{AutomatonMtp, QwenDeviceExecutor, ScheduledMtp},
        native_tokenizer::NativeTokenizer,
        scheduler::Engine,
        weights::Manifest,
    },
};
use std::{
    future::Future,
    path::Path,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};
fn controller() -> Result<Controller> {
    let c = Controller::new(serde_json::from_str(include_str!(
        "../fixtures/cache-config-bench.json"
    ))?)?;
    c.configure_drafts(DraftPolicy {
        max_proposers: 4,
        work_limit: 1,
        initial_mass: MAX_MASS,
        admission_mass: MAX_MASS / 8,
        reward_mass: MAX_MASS / 4,
        qualification_trials: 64,
        probe_every_epochs: 2,
    })?;
    Ok(c)
}
fn same(a: &[f32], b: &[f32]) -> Result<()> {
    ensure!(a.len() == b.len() && !a.is_empty(), "state geometry");
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
    ensure!(e <= 1e-5, "state drift {e}");
    Ok(())
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_automaton_smoke NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let e = Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 16, 1536 * 1024 * 1024).await?);
    let c = controller()?;
    let p = AutomatonMtp::register(c.clone(), e.clone(), "mtp-admission-smoke".into())?;
    let prompt = tokenizer.chat(None, "Complete: 1, 2, 3,")?;
    let parent = p.start(prompt.clone()).await?;
    let original = e.snapshot(parent.target().state()).await?;
    ensure!(
        p.next(&parent, 0, 8, &tokenizer.eos()).await.is_err(),
        "invalid width"
    );
    ensure!(
        p.next(&parent, 4, 0, &tokenizer.eos()).await.is_err(),
        "zero budget"
    );
    let other = AutomatonMtp::register(c.clone(), e.clone(), "foreign".into())?;
    ensure!(
        other.next(&parent, 4, 8, &tokenizer.eos()).await.is_err(),
        "foreign namespace"
    );
    let baseline = p.no_draft(&parent, &tokenizer.eos()).await?;
    ensure!(
        !baseline.admitted && baseline.emitted.len() == 1 && baseline.learned_proposed == 0,
        "baseline maintenance"
    );
    drop(baseline);
    let mut proposed = 0;
    let mut accepted = 0;
    let mut committed = 0;
    let mut calls = 0;
    for width in [1, 2, 4, 8] {
        let tx = p.next(&parent, width, 8, &tokenizer.eos()).await?;
        ensure!(tx.admitted == (width > 1), "admission decision");
        let mut reference = parent.target().clone();
        for &token in &tx.emitted {
            let step = e
                .greedy_verify(&reference, vec![], &tokenizer.eos())
                .await?;
            ensure!(step.emitted == [token], "admitted greedy output");
            reference = step.cursor;
        }
        same(
            &e.snapshot(reference.state()).await?,
            &e.snapshot(tx.cursor.target().state()).await?,
        )?;
        if tx.admitted {
            proposed += tx.learned_proposed;
            accepted += tx.learned_accepted;
            committed += tx.emitted.len();
            calls += tx.target_calls;
        }
    }
    let ledger = c.draft_ledger(p.id())?;
    ensure!(
        ledger.known_prefix == 3
            && ledger.proposed == proposed as u64
            && ledger.accepted == accepted as u64
            && ledger.committed == committed as u64
            && ledger.target_calls == calls as u64,
        "seed/learned/command accounting"
    );
    let before = ledger.clone();
    let cancel_eos = tokenizer.eos();
    let mut canceled = Box::pin(p.next(&parent, 4, 8, &cancel_eos));
    std::future::poll_fn(|cx| {
        ensure_pending(canceled.as_mut().poll(cx));
        Poll::Ready(())
    })
    .await;
    ensure!(
        c.draft_stats()?.active == 1,
        "native lease before cancellation"
    );
    drop(canceled);
    let wait = Instant::now();
    while c.draft_stats()?.active > 0 {
        ensure!(
            wait.elapsed() < Duration::from_secs(30),
            "cancel drain timeout"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    let after = c.draft_ledger(p.id())?;
    ensure!(
        after.observations == before.observations + 1
            && after.committed == before.committed
            && after.unfinished == 0,
        "canceled publication accounting"
    );
    let current = e.snapshot(parent.target().state()).await?;
    ensure!(
        original
            .iter()
            .zip(&current)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "immutable canceled parent"
    );
    // Fill the target/draft shared table: verification fits, draft synchronization fails.
    let fault_c = controller()?;
    let fault = AutomatonMtp::register(fault_c.clone(), e.clone(), "mtp-admission-smoke".into())?;
    let mut occupied = Vec::new();
    while e.stats().await?.residents < 15 {
        occupied.push(e.step(&e.empty_state(), vec![1]).await?.state);
    }
    ensure!(
        fault.next(&parent, 2, 8, &tokenizer.eos()).await.is_err(),
        "post-verifier pressure failure"
    );
    let failed = fault_c.draft_ledger(fault.id())?;
    ensure!(
        failed.failures == 1
            && failed.target_calls == 1
            && failed.committed == 0
            && failed.accepted == 0
            && failed.known_prefix == 1,
        "failed admitted command accounting"
    );
    drop(occupied);
    let recovered = p.no_draft(&parent, &tokenizer.eos()).await?;
    drop(recovered);
    // Another admitted lease saturates the controller; queued metadata is canceled.
    let identity = Identity {
        model: e.identity().into(),
        tokenizer: e.identity().into(),
        quantization: "fp32-explicit-attention-regular-kv-mtp-maintenance-v1".into(),
        namespace: "mtp-admission-smoke".into(),
    };
    let held = match c.request_draft(p.id(), prefix_key(&identity, &prompt)?, 4, true)? {
        DraftAdmission::Ready(p) => p,
        _ => anyhow::bail!("held admission"),
    };
    let fallback = p.next(&parent, 4, 8, &tokenizer.eos()).await?;
    ensure!(
        !fallback.admitted
            && fallback.emitted.len() == 1
            && c.draft_stats()?.queued == 0
            && c.draft_stats()?.cancelled == 1,
        "bounded queued fallback"
    );
    drop(fallback);
    drop(held);
    drop(parent);
    drop(other);
    drop(fault);
    drop(p);
    // Verify the shared Engine MTP bridge (ScheduledMtp) with submit and submit_stream.
    let sched_c = controller()?;
    let sched_mtp = ScheduledMtp::new(
        sched_c,
        e.clone(),
        tokenizer.eos(),
        4,
        "mtp-scheduled-bridge".into(),
    )?;
    let sched_storage = sched_mtp.storage();
    let engine = Engine::new(
        ServingConfig {
            version: 1,
            max_queued: 4,
            max_sequences: 3,
            max_batch_tokens: 24,
            prefill_chunk_tokens: 8,
            max_context_tokens: 511,
            max_new_tokens: 16,
            model_memory_bytes: 1024 * 1024 * 1024,
            max_in_flight: 3,
        },
        sched_storage,
        sched_mtp,
    )?;
    let mut streamed_req = engine.submit_stream(prompt.clone(), 8, 2)?;
    let nonstream_req = engine.submit(prompt.clone(), 8)?;
    let mut streamed_tokens = Vec::new();
    while let Some(ev) = streamed_req.next_token().await {
        streamed_tokens.push(ev.token);
    }
    let streamed_gen = streamed_req.result().await?;
    let nonstream_gen = nonstream_req.result().await?;
    ensure!(
        streamed_gen.tokens == nonstream_gen.tokens && streamed_tokens == streamed_gen.tokens,
        "ScheduledMtp stream/non-stream parity"
    );
    let sched_stats = engine.shutdown().await?;
    ensure!(
        sched_stats.completed == 2 && sched_stats.memory_used == 0 && !sched_stats.admission_closed,
        "ScheduledMtp engine drain"
    );
    let stats = Arc::try_unwrap(e)
        .map_err(|_| anyhow::anyhow!("executor leaked"))?
        .shutdown()
        .await?;
    ensure!(stats.logical_bytes == 0, "MTP automaton shutdown");
    let report = serde_json::json!({"pass":true,"native_output_and_state_gate":true,"seed_accounting":ledger,"cancellation_accounting":after,"post_verifier_failure":failed,"queue_fallback":true,"scheduled_mtp_stream_parity":true,"shutdown_bytes":stats.logical_bytes,"serving_qualified":false});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
fn ensure_pending<T>(poll: Poll<T>) {
    assert!(poll.is_pending(), "expected asynchronous native admission");
}
