//! Offline Combined-only greedy verifier/state pressure and request-local retrieval smoke.
//! Oracle draft cases test acceptance mechanics; their rate is not drafter acceptance.
#[allow(dead_code)]
#[path = "native_suite_reference.rs"]
mod frozen;
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    choices::select,
    native_mlx::Device,
    native_qwen::{DeviceState, GreedyCursor, QwenDeviceExecutor, SuffixDraft},
    native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{future::Future, io::Write, path::Path, task::Poll, time::Instant};
fn drift(a: &[f32], b: &[f32]) -> Result<(f64, f64)> {
    ensure!(
        !a.is_empty() && a.len() == b.len() && a.iter().chain(b).all(|x| x.is_finite()),
        "comparison finite/shape"
    );
    let (mut d, mut r, mut max) = (0., 0., 0f64);
    for (&x, &y) in a.iter().zip(b) {
        let delta = x as f64 - y as f64;
        d += delta * delta;
        r += (x as f64).powi(2);
        max = max.max(delta.abs());
    }
    Ok(((d / r.max(1e-30)).sqrt(), max))
}
async fn prefill(e: &QwenDeviceExecutor, ids: &[u32]) -> Result<GreedyCursor> {
    let mut chunks = ids.chunks(8);
    let mut cursor = e
        .greedy_start(chunks.next().context("empty prompt")?.to_vec())
        .await?;
    for chunk in chunks {
        cursor = e.greedy_extend(&cursor, chunk.to_vec()).await?;
    }
    Ok(cursor)
}
async fn same_state(
    e: &QwenDeviceExecutor,
    a: &DeviceState,
    b: &DeviceState,
) -> Result<(f64, f64)> {
    ensure!(a.position() == b.position(), "state positions");
    let error = drift(&e.snapshot(a).await?, &e.snapshot(b).await?)?;
    ensure!(error.0 <= 1e-5, "unified state preservation {error:?}");
    Ok(error)
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 1, "native_speculative_smoke NEW_DIRECTORY");
    let out = Path::new(&args[0]);
    std::fs::create_dir(out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let e = QwenDeviceExecutor::open(Device::Gpu, 16, 1536 * 1024 * 1024).await?;
    let mut log = std::fs::File::create(out.join("transactions.jsonl"))?;
    let prompt = tokenizer.chat(
        None,
        "Continue this Rust sequence with a short code example: let a = 1; let b = 2; let c = 3;",
    )?;
    let seed = prefill(&e, &prompt).await?;
    let mut oracle = Vec::new();
    let mut current = seed.clone();
    let mut baselines = Vec::new();
    for _ in 0..9 {
        let t = select(current.logits(), None)?;
        oracle.push(t);
        current = e.greedy_extend(&current, vec![t]).await?;
        baselines.push(current.clone());
    }
    // Verify every block row against a one-token target call, not just the final winner.
    let mut row_max = [0f64; 2];
    let mut state_max = [0f64; 2];
    for width in [1, 2, 4, 8] {
        let block = e
            .verify_block(seed.state(), oracle[..width].to_vec())
            .await?;
        ensure!(block.logits.len() == width, "position-major row count");
        for (i, row) in block.logits.iter().enumerate() {
            let error = drift(baselines[i].logits(), row)?;
            row_max[0] = row_max[0].max(error.0);
            row_max[1] = row_max[1].max(error.1);
            ensure!(error.0 <= 1e-5, "per-position logit gate {error:?}");
            ensure!(
                select(row, None)? == select(baselines[i].logits(), None)?,
                "per-position argmax changed"
            );
        }
        let error = same_state(&e, &block.state, baselines[width - 1].state()).await?;
        state_max[0] = state_max[0].max(error.0);
        state_max[1] = state_max[1].max(error.1);
    }
    let mut boundary_cases = 0;
    for width in [1, 2, 4, 8] {
        for boundary in 0..=width {
            let mut draft = oracle[..width].to_vec();
            if boundary < width {
                draft[boundary] = (draft[boundary] + 1) % 248320;
            }
            let t = e.greedy_verify(&seed, draft, &[]).await?;
            let n = if boundary < width {
                boundary + 1
            } else {
                width + 1
            };
            ensure!(
                t.accepted == boundary && t.emitted == oracle[..n],
                "acceptance boundary width={width} boundary={boundary}"
            );
            let error = same_state(&e, t.cursor.state(), baselines[n - 1].state()).await?;
            ensure!(
                drift(t.cursor.logits(), baselines[n - 1].logits())?.0 <= 1e-5,
                "accepted logits"
            );
            writeln!(
                log,
                "{}",
                serde_json::json!({"type":"oracle-boundary","width":width,"boundary":boundary,"emitted":t.emitted,"target_calls":t.target_calls,"replayed":t.replayed_tokens,"state_error":error})
            )?;
            log.flush()?;
            boundary_cases += 1;
        }
    }
    // EOS at the first candidate, inside an accepted prefix, and as a target correction.
    for width in [1, 4, 8] {
        for stop in 0..width {
            let terminal = [oracle[stop]];
            let expected = oracle[..=stop]
                .iter()
                .position(|t| terminal.contains(t))
                .unwrap()
                + 1;
            let t = e
                .greedy_verify(&seed, oracle[..width].to_vec(), &terminal)
                .await?;
            ensure!(
                t.cursor.terminal() && t.emitted == oracle[..expected],
                "EOS prefix"
            );
            same_state(&e, t.cursor.state(), baselines[expected - 1].state()).await?;
            ensure!(
                e.greedy_verify(&t.cursor, vec![], &terminal).await.is_err(),
                "terminal decode"
            );
        }
    }
    let correction = e
        .greedy_verify(&seed, vec![(oracle[0] + 1) % 248320], &[oracle[0]])
        .await?;
    ensure!(
        correction.cursor.terminal() && correction.accepted == 0,
        "correction EOS"
    );
    drop(correction);
    // Bad drafts leave the original cursor resident and reusable.
    let before = e.stats().await?;
    for draft in [vec![248320], vec![1; 9]] {
        ensure!(
            e.greedy_verify(&seed, draft, &eos).await.is_err(),
            "bad draft"
        );
    }
    ensure!(
        e.greedy_verify(&seed, vec![], &[248320]).await.is_err(),
        "bad EOS"
    );
    ensure!(
        before.residents == e.stats().await?.residents,
        "rejected admission leaked"
    );
    let mut cancelled = Box::pin(e.greedy_verify(&seed, oracle[..8].to_vec(), &[]));
    ensure!(
        std::future::poll_fn(|cx| Poll::Ready(matches!(
            cancelled.as_mut().poll(cx),
            Poll::Pending
        )))
        .await,
        "pending cancellation"
    );
    drop(cancelled);
    ensure!(
        e.stats().await?.residents == before.residents,
        "cancelled verifier leaked"
    );
    // Cancel after verification has completed and bonus work has been admitted.
    let mut cancelled = Box::pin(e.greedy_verify(&seed, oracle[..8].to_vec(), &[]));
    ensure!(
        std::future::poll_fn(|cx| Poll::Ready(matches!(
            cancelled.as_mut().poll(cx),
            Poll::Pending
        )))
        .await,
        "verification pending"
    );
    e.stats().await?; // Worker barrier: block receipt is available, transaction unpolled.
    ensure!(
        std::future::poll_fn(|cx| Poll::Ready(matches!(
            cancelled.as_mut().poll(cx),
            Poll::Pending
        )))
        .await,
        "bonus pending"
    );
    drop(cancelled);
    ensure!(
        e.stats().await?.residents == before.residents,
        "cancelled bonus leaked"
    );
    let repeat = e.greedy_verify(&seed, vec![], &[]).await?;
    ensure!(repeat.emitted == oracle[..1], "seed mutation");
    drop(repeat);
    drop((current, baselines));
    // Hold 15 versions: verification can reserve the last slot but bonus cannot.
    let mut held = Vec::new();
    for _ in 0..14 {
        held.push(e.greedy_extend(&seed, vec![oracle[0]]).await?);
    }
    ensure!(e.stats().await?.residents == 15, "pre-commit slot pressure");
    ensure!(
        e.greedy_verify(&seed, oracle[..1].to_vec(), &[])
            .await
            .is_err(),
        "partial transaction exceeded slot budget"
    );
    ensure!(
        e.stats().await?.residents == 15,
        "partial transaction leaked"
    );
    drop(held);
    let recovered = e.greedy_verify(&seed, oracle[..1].to_vec(), &[]).await?;
    ensure!(recovered.emitted == oracle[..2], "transaction recovery");
    drop(recovered);
    let (a, b, c) = tokio::join!(
        e.greedy_verify(&seed, oracle[..2].to_vec(), &[]),
        e.greedy_verify(&seed, vec![(oracle[0] + 1) % 248320], &[]),
        e.greedy_verify(&seed, vec![], &[])
    );
    let (a, b, c) = (a?, b?, c?);
    ensure!(
        a.emitted == oracle[..3] && b.emitted == oracle[..1] && c.emitted == oracle[..1],
        "concurrent branch isolation"
    );
    drop((a, b, c, seed));
    ensure!(e.stats().await?.residents == 0, "mechanics drain");
    // Context-edge verification, full acceptance without room for bonus, overflow rejection.
    let mut cursor = e.greedy_start(vec![16; 8]).await?;
    for _ in 1..63 {
        cursor = e.greedy_extend(&cursor, vec![16; 8]).await?;
    }
    ensure!(cursor.state().position() == 504, "edge start");
    let mut edge = Vec::new();
    let mut reference = cursor.clone();
    for _ in 0..8 {
        let token = select(reference.logits(), None)?;
        edge.push(token);
        reference = e.greedy_extend(&reference, vec![token]).await?;
    }
    let t = e.greedy_verify(&cursor, edge.clone(), &[]).await?;
    ensure!(
        t.accepted == 8 && t.emitted == edge && t.cursor.state().position() == 512,
        "context-edge commit"
    );
    same_state(&e, t.cursor.state(), reference.state()).await?;
    ensure!(
        e.greedy_verify(&t.cursor, vec![], &[]).await.is_err(),
        "context overflow"
    );
    drop((cursor, reference, t));
    // Full-vocabulary greedy only. Constrained-choice cases are explicitly excluded,
    // not silently reinterpreted as free generation. Existing 60-case reference runs separately.
    let mut captures = std::fs::File::create(out.join("requests.jsonl"))?;
    let mut requests = 0;
    let mut totals = [0usize; 4];
    let mut cases: Vec<_> = frozen::inventory()?
        .into_iter()
        .filter(|c| c.choices.is_none() && c.suite != "contract")
        .collect();
    for line in include_str!("../fixtures/native-speculative-prompts.tsv").lines() {
        let (id, prompt) = line.split_once('\t').context("authored probe fields")?;
        cases.push(frozen::Case {
            suite: "agent-smoke".into(),
            id: id.into(),
            prompt: prompt.into(),
            choices: None,
            expected: String::new(),
        });
    }
    ensure!(cases.len() == 25, "fixed free-greedy/authored inventory");
    for case in cases {
        let cap = if case.suite == "agent-smoke" { 64 } else { 32 };
        let ids = tokenizer.chat(None, &case.prompt)?;
        let seed = prefill(&e, &ids).await?;
        let mut base = seed.clone();
        let start = Instant::now();
        let mut tokens = Vec::new();
        for _ in 0..cap {
            let t = e.greedy_verify(&base, vec![], &eos).await?;
            tokens.extend_from_slice(&t.emitted);
            base = t.cursor;
            if base.terminal() {
                break;
            }
        }
        let baseline_seconds = start.elapsed().as_secs_f64();
        let mut d = SuffixDraft::new(4096)?;
        d.observe(&ids);
        let mut spec = seed.clone();
        let start = Instant::now();
        let mut generated = Vec::new();
        let mut proposed = 0;
        let mut accepted = 0;
        let mut calls = 0;
        let mut replayed = 0;
        while generated.len() < cap && !spec.terminal() {
            let draft = d.propose(8.min(cap - generated.len() - 1))?;
            let t = e.greedy_verify(&spec, draft, &eos).await?;
            proposed += t.proposed;
            accepted += t.accepted;
            calls += t.target_calls;
            replayed += t.replayed_tokens;
            d.observe(&t.emitted);
            generated.extend_from_slice(&t.emitted);
            spec = t.cursor;
        }
        let speculative_seconds = start.elapsed().as_secs_f64();
        ensure!(
            tokens == generated && base.terminal() == spec.terminal(),
            "trajectory changed {} {}",
            case.suite,
            case.id
        );
        let state_error = same_state(&e, base.state(), spec.state()).await?;
        let text = tokenizer.decode(&generated)?;
        let quality = spec.terminal() && text.trim() == case.expected;
        writeln!(
            captures,
            "{}",
            serde_json::json!({"suite":case.suite,"id":case.id,"prompt_ids":ids,"generated":generated,"text":text,"quality_pass":if case.suite=="agent-smoke" {None}else{Some(quality)},"terminal":spec.terminal(),"proposed":proposed,"accepted":accepted,"target_calls":calls,"replayed":replayed,"baseline_seconds":baseline_seconds,"speculative_seconds":speculative_seconds,"state_error":state_error})
        )?;
        captures.flush()?;
        requests += 1;
        totals[0] += proposed;
        totals[1] += accepted;
        totals[2] += calls;
        totals[3] += generated.len();
        eprintln!("SPEC-AB {} {}", case.suite, case.id);
    }
    let stats = e.stats().await?;
    ensure!(
        stats.residents == 0 && stats.uploaded_bytes == 0,
        "request drain/uploads"
    );
    let final_stats = e.shutdown().await?;
    ensure!(
        final_stats.residents == 0 && final_stats.invalidated_at_shutdown == 0,
        "shutdown drain"
    );
    let report = serde_json::json!({"pass":true,"identity":manifest.identity,"boundary_cases":boundary_cases,"per_position_logit_maxima":row_max,"block_state_maxima":state_max,"requests":requests,"frozen_free_greedy_requests":21,"authored_agent_smoke_requests":4,"retrieval_proposed":totals[0],"retrieval_accepted":totals[1],"retrieval_acceptance":if totals[0]>0{Some(totals[1] as f64/totals[0] as f64)}else{None},"retrieval_target_calls":totals[2],"retrieval_committed_tokens":totals[3],"context_edge":512,"cancellation":true,"production_qualified":false,"trained_mtp":false,"scheduler_integrated":false,"stochastic_sampling":false,"diagnostic_exported_bytes":stats.exported_bytes,"uploaded_bytes":stats.uploaded_bytes,"mlx_peak_bytes_process":stats.mlx_peak_bytes,"residents_after_drain":final_stats.residents});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
