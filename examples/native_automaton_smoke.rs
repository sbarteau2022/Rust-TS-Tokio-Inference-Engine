//! Combined-only Spec 15 proposer integration; no trained MTP or serving promotion.
use anyhow::{ensure, Context, Result};
use rust_server::{
    cache::*,
    serving::{
        native_mlx::Device,
        native_qwen::{
            AutomatonProposer, CompletionGuard, DraftProposer, ProposalContext, ProposalFuture,
            QwenDeviceExecutor, RetrievalProposer,
        },
        native_tokenizer::NativeTokenizer,
        weights::Manifest,
    },
};
use std::{
    future::Future,
    io::Write,
    path::Path,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};
use tokio::sync::Notify;
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
struct Gate {
    entered: Notify,
    release: Notify,
}
impl DraftProposer for Gate {
    fn name(&self) -> &str {
        "CONTROL-blocked-proposer"
    }
    fn propose(&self, c: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async move {
            self.entered.notify_one();
            self.release.notified().await;
            let _guard = c.completion;
            Ok(vec![])
        })
    }
}
struct Oversize;
impl DraftProposer for Oversize {
    fn name(&self) -> &str {
        "CONTROL-invalid-budget"
    }
    fn propose(&self, c: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async move { Ok(vec![1; c.maximum + 1]) })
    }
}
struct Panics;
impl DraftProposer for Panics {
    fn name(&self) -> &str {
        "CONTROL-panic"
    }
    fn propose(&self, _: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async { panic!("deliberate control proposer panic") })
    }
}
struct InvalidToken;
impl DraftProposer for InvalidToken {
    fn name(&self) -> &str {
        "CONTROL-invalid-token"
    }
    fn propose(&self, _: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async { Ok(vec![u32::MAX]) })
    }
}
struct NextToken;
impl DraftProposer for NextToken {
    fn name(&self) -> &str {
        "CONTROL-oracle-for-bonus-exhaustion"
    }
    fn propose(&self, c: ProposalContext) -> ProposalFuture<'_> {
        Box::pin(async move {
            Ok(vec![rust_server::serving::choices::select(
                c.cursor.logits(),
                None,
            )?])
        })
    }
}
struct Guard {
    permit: Option<DraftPermit>,
}
impl Drop for Guard {
    fn drop(&mut self) {
        drop(self.permit.take());
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 1, "native_automaton_smoke NEW_DIRECTORY");
    let out = Path::new(&args[0]);
    std::fs::create_dir(out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let e = Arc::new(QwenDeviceExecutor::open(Device::Gpu, 16, 1536 * 1024 * 1024).await?);
    let c = controller()?;
    let p = Arc::new(AutomatonProposer::register(
        c.clone(),
        e.clone(),
        "bench:code-cap64".into(),
        Arc::new(RetrievalProposer),
    )?);
    let eos = tokenizer.eos();
    let mut log = std::fs::File::create(out.join("requests.jsonl"))?;
    let mut totals = (0usize, 0usize, 0usize, 0usize, 0f64, 0f64);
    for (index, line) in include_str!("../fixtures/native-speculative-prompts.tsv")
        .lines()
        .enumerate()
    {
        let (name, prompt) = line.split_once('\t').context("fixture")?;
        let initial = p.start(tokenizer.chat(None, prompt)?).await?;
        let before = c.draft_ledger(p.id())?;
        let mut base = initial.clone();
        let mut expected = Vec::new();
        let timer = Instant::now();
        while expected.len() < 64 && !base.greedy().terminal() {
            let t = p.no_draft(&base, &eos).await?;
            expected.extend(t.emitted);
            base = t.cursor;
        }
        let baseline_seconds = timer.elapsed().as_secs_f64();
        let mut actual = Vec::new();
        let mut cursor = initial.clone();
        let (mut proposed, mut accepted, mut calls, mut drafted) = (0, 0, 0, 0);
        let timer = Instant::now();
        while actual.len() < 64 && !cursor.greedy().terminal() {
            let t = AutomatonProposer::next_best(&[p.as_ref()], &cursor, 64 - actual.len(), &eos)
                .await?;
            proposed += t.proposed;
            accepted += t.accepted;
            calls += t.target_calls;
            drafted += usize::from(t.proposer.is_some());
            actual.extend(t.emitted);
            cursor = t.cursor;
        }
        let seconds = timer.elapsed().as_secs_f64();
        ensure!(
            actual == expected,
            "automaton altered greedy output: {name}"
        );
        let a = e.snapshot(cursor.greedy().state()).await?;
        let b = e.snapshot(base.greedy().state()).await?;
        let delta: f64 = a
            .iter()
            .zip(&b)
            .map(|(&x, &y)| (f64::from(x) - f64::from(y)).powi(2))
            .sum();
        let norm: f64 = b.iter().map(|&x| f64::from(x).powi(2)).sum();
        let state_drift = (delta / norm.max(1e-30)).sqrt();
        ensure!(
            a.len() == b.len() && state_drift <= 1e-5,
            "unified state gate"
        );
        let after = c.draft_ledger(p.id())?;
        writeln!(
            log,
            "{}",
            serde_json::json!({"id":name,"tokens":actual.len(),"ids":actual,
            "baseline_seconds":baseline_seconds,"automaton_seconds":seconds,"proposed":proposed,
            "accepted":accepted,"target_calls":calls,"draft_admissions":drafted,"state_relative_l2":state_drift,
            "ledger_before":before,"ledger_after":after,"equal":true})
        )?;
        totals.0 += expected.len();
        totals.1 += proposed;
        totals.2 += accepted;
        totals.3 += calls;
        totals.4 += baseline_seconds;
        totals.5 += seconds;
        c.audit(index as u64 + 1, &[], 0, Pressure::Normal)?;
    }
    ensure!(c.draft_stats()?.active == 0, "benchmark leases leaked");
    // Waiter cancellation: admission remains held while plugin/native work runs.
    let controls = controller()?;
    let gate = Arc::new(Gate {
        entered: Notify::new(),
        release: Notify::new(),
    });
    let blocked = Arc::new(AutomatonProposer::register(
        controls.clone(),
        e.clone(),
        "control:cancel".into(),
        gate.clone(),
    )?);
    let original = blocked.start(vec![16; 8]).await?;
    let before = e.snapshot(original.greedy().state()).await?;
    let pp = blocked.clone();
    let cc = original.clone();
    let task = tokio::spawn(async move { pp.next(&cc, 8, &[]).await });
    gate.entered.notified().await;
    ensure!(controls.draft_stats()?.active == 1, "cancel gate not held");
    ensure!(
        controls.audit(1, &[], 0, Pressure::Normal).is_err(),
        "active audit accepted"
    );
    task.abort();
    ensure!(
        task.await.is_err_and(|e| e.is_cancelled()),
        "waiter not cancelled"
    );
    ensure!(
        controls.draft_stats()?.active == 1,
        "waiter released live lease"
    );
    gate.release.notify_one();
    tokio::time::timeout(Duration::from_secs(30), async {
        while controls.draft_stats().unwrap().active > 0 {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    let cancelled = controls.draft_ledger(blocked.id())?;
    ensure!(
        cancelled.observations == 1 && cancelled.committed == 0 && cancelled.cost_ns > 0,
        "unpublished accounting"
    );
    ensure!(
        before == e.snapshot(original.greedy().state()).await?,
        "cancel mutated parent"
    );
    controls.audit(1, &[], 0, Pressure::Normal)?;
    // A malformed proposer is rejected before a target call and records failure.
    let invalid = AutomatonProposer::register(
        controls.clone(),
        e.clone(),
        "control:cancel".into(),
        Arc::new(Oversize),
    )?;
    let steps = e.stats().await?.steps;
    ensure!(
        invalid.next(&original, 8, &[]).await.is_err(),
        "oversize accepted"
    );
    ensure!(
        e.stats().await?.steps == steps,
        "invalid proposer reached target"
    );
    ensure!(
        controls.draft_ledger(invalid.id())?.failures == 1,
        "failure not audited"
    );
    ensure!(
        invalid.next(&original, 0, &[]).await.is_err(),
        "zero budget"
    );
    ensure!(
        p.next(&original, 8, &[]).await.is_err(),
        "cross-namespace cursor accepted"
    );
    let invalid_token = AutomatonProposer::register(
        controls.clone(),
        e.clone(),
        "control:cancel".into(),
        Arc::new(InvalidToken),
    )?;
    ensure!(
        invalid_token.next(&original, 8, &[]).await.is_err(),
        "invalid token accepted"
    );
    ensure!(
        controls.draft_ledger(invalid_token.id())?.failures == 1,
        "invalid token not audited"
    );
    let panics = AutomatonProposer::register(
        controls.clone(),
        e.clone(),
        "control:cancel".into(),
        Arc::new(Panics),
    )?;
    ensure!(
        panics.next(&original, 8, &[]).await.is_err(),
        "panic accepted"
    );
    ensure!(
        controls.draft_ledger(panics.id())?.unfinished == 1 && controls.draft_stats()?.active == 0,
        "panic leaked lease"
    );
    drop((invalid_token, panics));
    // Native work owns an independent guard after its Rust verification future drops.
    let native = controller()?;
    let identity = Identity {
        model: e.identity().into(),
        tokenizer: e.identity().into(),
        quantization: "fp32-explicit-attention-regular-kv-v1".into(),
        namespace: "control:native-cancel".into(),
    };
    let id = native.register_proposer("CONTROL-native-guard".into(), identity.clone())?;
    let permit = match native.request_draft(id, prefix_key(&identity, &[16; 8])?, 8, true)? {
        DraftAdmission::Ready(p) => p,
        _ => anyhow::bail!("guard admission"),
    };
    permit.bind(&[17; 8])?;
    let guard: CompletionGuard = Arc::new(Guard {
        permit: Some(permit),
    });
    let mut future =
        Box::pin(e.greedy_verify_guarded(original.greedy(), vec![17; 8], &[], Some(guard.clone())));
    ensure!(
        std::future::poll_fn(|cx| Poll::Ready(matches!(future.as_mut().poll(cx), Poll::Pending)))
            .await,
        "native not pending"
    );
    drop(future);
    drop(guard);
    ensure!(
        native.draft_stats()?.active == 1,
        "native command lost completion guard"
    );
    ensure!(
        native.audit(1, &[], 0, Pressure::Normal).is_err(),
        "native audit advanced early"
    );
    e.stats().await?; // FIFO barrier after native completion.
    ensure!(
        native.draft_stats()?.active == 0 && native.draft_ledger(id)?.unfinished == 1,
        "native guard drain"
    );
    native.audit(1, &[], 0, Pressure::Normal)?;
    ensure!(
        before == e.snapshot(original.greedy().state()).await?,
        "guard cancellation mutated parent"
    );
    // Force a successful verifier allocation followed by failed bonus allocation.
    // Both admitted target commands must remain charged despite zero publication.
    let faults = controller()?;
    let oracle = AutomatonProposer::register(
        faults.clone(),
        e.clone(),
        "control:cancel".into(),
        Arc::new(NextToken),
    )?;
    ensure!(
        e.stats().await?.residents == 1,
        "fault control input residency"
    );
    let mut pressure = Vec::new();
    for token in 30..44 {
        pressure.push(e.step(original.greedy().state(), vec![token]).await?);
    }
    ensure!(e.stats().await?.residents == 15, "bonus pressure setup");
    ensure!(
        oracle.next(&original, 8, &[]).await.is_err(),
        "bonus allocation unexpectedly succeeded"
    );
    let failed_calls = faults.draft_ledger(oracle.id())?;
    ensure!(
        failed_calls.failures == 1 && failed_calls.target_calls == 2 && failed_calls.committed == 0,
        "failed transaction lost admitted target calls"
    );
    drop((pressure, oracle));
    ensure!(e.stats().await?.residents == 1, "failed bonus state leaked");
    ensure!(
        before == e.snapshot(original.greedy().state()).await?,
        "failed bonus mutated parent"
    );
    let final_ledger = c.draft_ledger(p.id())?;
    drop((original, invalid, blocked, p));
    let stats = e.stats().await?;
    ensure!(stats.residents == 0, "state retirement");
    let summary = serde_json::json!({"pass":true,"model":e.identity(),"tokens":totals.0,"proposed":totals.1,
        "accepted":totals.2,"target_calls":totals.3,"baseline_tokens_per_second":totals.0 as f64/totals.4,
        "automaton_tokens_per_second":totals.0 as f64/totals.5,"baseline_seconds":totals.4,"automaton_seconds":totals.5,
        "ledger":final_ledger,
        "failed_calls":failed_calls,"cancelled":cancelled,"native_aborted":native.draft_ledger(id)?,"bounded_native_guard":true,
        "active":c.draft_stats()?.active,"controls":controls.draft_stats()?,"production_qualified":false,
        "trained_mtp":false,"physical_command_buffer_cap_claim":false});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    println!("{summary}");
    let executor = Arc::try_unwrap(e).map_err(|_| anyhow::anyhow!("executor references remain"))?;
    executor.shutdown().await?;
    Ok(())
}
