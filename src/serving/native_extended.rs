//! Extended text-boundary qualification; never an independent model oracle.
use super::{
    config::Config,
    model_ir::{ModelStepIr, StepKind},
    native_mlx::Device,
    native_qwen::{QwenExecutor, ScheduledQwen},
    native_tokenizer::NativeTokenizer,
    scheduler::Engine,
};
use anyhow::{ensure, Result};
use std::time::Instant;
struct Case<'a> {
    id: &'a str,
    domain: &'a str,
    choices: Option<Vec<&'a str>>,
    prompt: &'a str,
    expected: &'a str,
}
fn cases(text: &str) -> Result<Vec<Case<'_>>> {
    ensure!(text.len() <= 64 * 1024, "domain fixture cap");
    let mut seen = std::collections::BTreeSet::new();
    text.lines()
        .map(|line| {
            let p: Vec<_> = line.split('\t').collect();
            ensure!(
                p.len() == 5 && p.iter().all(|s| !s.is_empty()),
                "domain row"
            );
            ensure!(
                seen.insert(p[0]) && ["rust", "security"].contains(&p[1]),
                "duplicate/unknown case"
            );
            let choices = if p[2] == "-" {
                None
            } else {
                let v: Vec<_> = p[2].split(',').collect();
                ensure!(
                    (2..=16).contains(&v.len())
                        && v.iter().all(|s| !s.is_empty())
                        && v.contains(&p[4]),
                    "domain alternatives"
                );
                let unique: std::collections::BTreeSet<_> = v.iter().collect();
                ensure!(unique.len() == v.len(), "duplicate alternatives");
                Some(v)
            };
            Ok(Case {
                id: p[0],
                domain: p[1],
                choices,
                prompt: p[3],
                expected: p[4],
            })
        })
        .collect()
}
fn relative(a: &[f32], b: &[f32]) -> Result<f64> {
    ensure!(
        !a.is_empty() && a.len() == b.len() && a.iter().chain(b).all(|v| v.is_finite()),
        "comparison shape/finite"
    );
    let (mut e, mut n) = (0., 0.);
    for (a, b) in a.iter().zip(b) {
        e += (*a as f64 - *b as f64).powi(2);
        n += (*a as f64).powi(2);
    }
    Ok((e / n.max(1e-30)).sqrt())
}
pub async fn run() -> Result<()> {
    let cases = cases(include_str!("../../fixtures/native-domain-heldout.tsv"))?;
    ensure!(cases.len() == 24, "frozen domain case count");
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    println!("EXTENDED identity={} sdk={} activation=f32 state=fp32 regular-kv independent_reference=false full_model_qualified=false",executor.manifest().identity,super::native_mlx::version());
    let cycle = tokenizer.encode(
        "Rust ownership and careful security checks preserve application state across requests. ",
    )?;
    ensure!(!cycle.is_empty(), "empty cycle");
    let ids: Vec<_> = cycle.iter().copied().cycle().take(512).collect();
    let timer = Instant::now();
    let mut base = executor.empty_state();
    for chunk in ids[..504].chunks(8) {
        base = executor.step(&base, chunk.to_vec()).await?.state;
    }
    let full = executor.step(&base, ids[504..].to_vec()).await?;
    let mut sequential = base.clone();
    let mut last = None;
    let mut position511 = None;
    for token in &ids[504..] {
        let ir = ModelStepIr::compile(
            StepKind::Decode,
            sequential.position(),
            vec![*token],
            248320,
        )?;
        let result = executor.execute(&sequential, ir).await?;
        sequential = result.state.clone();
        if sequential.position() == 511 {
            position511 = Some(sequential.clone());
        }
        last = Some(result);
    }
    let single = last.unwrap();
    let logit = relative(&full.logits, &single.logits)?;
    let state = relative(full.state.snapshot_values(), single.state.snapshot_values())?;
    let continuation =
        full.greedy_token() == single.greedy_token() && logit <= 0.01 && state <= 0.01;
    let overflow =
        executor.step(&full.state, vec![1]).await.is_err() && full.state.position() == 512;
    println!("CONTEXT position={} state_bytes={} continuation={continuation} logit_nrmse={logit:.8} state_nrmse={state:.8} overflow_rejected={overflow} elapsed_ms={:.3}",full.state.position(),full.state.bytes(),timer.elapsed().as_secs_f64()*1000.);
    drop(base);
    drop(full);
    drop(single);
    drop(sequential);
    let fork = position511.unwrap();
    let (a, b, c) = tokio::join!(
        executor.step(&fork, vec![1]),
        executor.step(&fork, vec![2]),
        executor.step(&fork, vec![3])
    );
    let parallel = [a?, b?, c?];
    let mut isolated = true;
    for (i, result) in parallel.iter().enumerate() {
        let serial = executor.step(&fork, vec![i as u32 + 1]).await?;
        isolated &= result.greedy_token() == serial.greedy_token()
            && relative(&result.logits, &serial.logits)? <= 1e-7
            && relative(
                result.state.snapshot_values(),
                serial.state.snapshot_values(),
            )? <= 1e-7;
    }
    drop(parallel);
    drop(fork);
    ensure!(
        executor.resources().leases == 0,
        "native leases did not drain"
    );
    println!("FORKS admitted=3 distinct_tokens=true isolated={isolated} native_leases=0");
    let backend = ScheduledQwen::new(executor, tokenizer.eos());
    let storage = backend.storage();
    let engine = Engine::new(
        Config {
            version: 1,
            max_queued: 3,
            max_sequences: 3,
            max_batch_tokens: 24,
            prefill_chunk_tokens: 8,
            max_context_tokens: 512,
            max_new_tokens: 32,
            model_memory_bytes: 1024 * 1024 * 1024,
            max_in_flight: 3,
        },
        storage,
        backend,
    )?;
    let mut passed = 0;
    let timer = Instant::now();
    let mut output_tokens = 0;
    for group in cases.chunks(3) {
        let mut requests = Vec::new();
        for case in group {
            let prompt = tokenizer.chat(None, case.prompt)?;
            let request = match &case.choices {
                Some(choices) => engine.submit_choices(
                    prompt,
                    32,
                    choices
                        .iter()
                        .map(|s| tokenizer.encode_choice(s))
                        .collect::<Result<Vec<_>>>()?,
                )?,
                None => engine.submit(prompt, 32)?,
            };
            requests.push((case, request));
        }
        for (case, request) in requests {
            let result = request.result().await?;
            output_tokens += result.tokens.len();
            let actual = tokenizer.decode(&result.tokens)?;
            let pass = actual.trim() == case.expected && result.finish_reason == "eos";
            passed += usize::from(pass);
            println!("DOMAIN id={} domain={} constrained={} pass={pass} expected={:?} actual={actual:?} finish={}",case.id,case.domain,case.choices.is_some(),case.expected,result.finish_reason);
        }
    }
    let elapsed = timer.elapsed().as_secs_f64();
    let stats = engine.shutdown().await?;
    let drain = stats.memory_used == 0 && !stats.admission_closed;
    println!("EXTENDED-SUMMARY domain={passed}/{} continuation={continuation} overflow={overflow} isolation={isolated} drain={drain} output_tokens_including_eos={output_tokens} wall_seconds={elapsed:.3} logical_memory_peak={} independent_reference=false full_model_qualified=false scope=authored-local-gate",cases.len(),stats.memory_peak);
    ensure!(
        passed == cases.len() && continuation && overflow && isolated && drain,
        "extended local qualification failed; preserve report"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn domain_fixtures_and_invalid_contracts() {
        let all = cases(include_str!("../../fixtures/native-domain-heldout.tsv")).unwrap();
        assert_eq!(all.len(), 24);
        assert_eq!(all.iter().filter(|c| c.domain == "rust").count(), 12);
        for input in [
            "id\trust\tyes\tprompt\tyes",
            "id\trust\tyes,no\tprompt\tmaybe",
            "id\trust\tyes,yes\tprompt\tyes",
            "id\tunknown\t-\tprompt\tyes",
        ] {
            assert!(cases(input).is_err());
        }
        assert!(relative(&[1.], &[f32::NAN]).is_err());
        assert!(relative(&[], &[]).is_err());
    }
}
