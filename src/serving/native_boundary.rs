//! Text I/O boundary: native tokenization, chat rendering and local qualification only.
use super::{
    native_mlx::Device,
    native_qwen::{ActivationPrecision, QwenExecutor, ScheduledQwen},
};
use anyhow::{ensure, Result};
/// Local generation through the same fair scheduler used by serving requests.
pub async fn generate_cli(
    prompt: &str,
    max_new: usize,
    precision: ActivationPrecision,
    device: Device,
    choices: Option<Vec<String>>,
) -> Result<()> {
    use super::{config::Config, native_tokenizer::NativeTokenizer, scheduler::Engine};
    ensure!((1..=128).contains(&max_new), "generation limit 1..128");
    let started = std::time::Instant::now();
    let executor = QwenExecutor::open_with_precision(device, precision).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let ids = tokenizer.chat(None, prompt)?;
    println!("NATIVE-QWEN identity={} device={device:?} mode={precision:?}-activations/fp32-recurrence kv=regular max_in_flight=3 context=512 chunk=8 prompt_tokens={} load_ms={:.3}",executor.manifest().identity,ids.len(),started.elapsed().as_secs_f64()*1000.);
    let adapter = ScheduledQwen::new(executor, tokenizer.eos());
    let storage = adapter.storage();
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 512,
        max_new_tokens: 128,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    let engine = Engine::new(config, storage, adapter)?;
    let begin = std::time::Instant::now();
    let request = match choices {
        Some(values) => engine.submit_choices(
            ids,
            max_new,
            values
                .iter()
                .map(|v| tokenizer.encode_choice(v))
                .collect::<Result<Vec<_>>>()?,
        )?,
        None => engine.submit(ids, max_new)?,
    };
    let result = request.result().await?;
    println!(
        "OUTPUT-CONTRACT fingerprint={:?}",
        result.output_contract_fingerprint
    );
    println!("{}", tokenizer.decode(&result.tokens)?);
    println!(
        "NATIVE-QWEN generated={} elapsed_ms={:.3} finish={} tokens={:?} held_out_qualified=false",
        result.tokens.len(),
        begin.elapsed().as_secs_f64() * 1000.,
        result.finish_reason,
        result.tokens
    );
    let stats = engine.shutdown().await?;
    ensure!(
        stats.memory_used == 0 && !stats.admission_closed,
        "scheduler did not drain cleanly"
    );
    Ok(())
}

fn drift(a: &[f32], b: &[f32]) -> Result<(f64, f64)> {
    ensure!(a.len() == b.len() && !a.is_empty(), "drift shapes");
    let mut error = 0f64;
    let mut magnitude = 0f64;
    let mut max = 0f64;
    for (a, b) in a.iter().zip(b) {
        let d = *a as f64 - *b as f64;
        error += d * d;
        magnitude += (*a as f64) * (*a as f64);
        max = max.max(d.abs());
    }
    Ok(((error / magnitude.max(1e-30)).sqrt(), max))
}
/// Fixed local qualification fixtures; output is a plain text report, never a runtime JSON protocol.
pub async fn qualify_cli(contracts: bool) -> Result<()> {
    use super::{config::Config, native_tokenizer::NativeTokenizer, scheduler::Engine};
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    println!("QUALIFICATION identity={} SDK={} profile=native-f32/fp32-state regular-kv context=512 chunk=8",executor.manifest().identity,super::native_mlx::version());
    let mut prefix =
        tokenizer.encode("The quick brown fox jumps over the lazy dog and continues walking.")?;
    prefix.truncate(8);
    ensure!(prefix.len() == 8, "continuation fixture too short");
    let full = executor
        .step(&executor.empty_state(), prefix.clone())
        .await?;
    let mut sequential = executor.empty_state();
    let mut last = None;
    for token in prefix {
        let result = executor.step(&sequential, vec![token]).await?;
        sequential = result.state.clone();
        last = Some(result);
    }
    let single = last.unwrap();
    let (logit_rmse, logit_max) = drift(&full.logits, &single.logits)?;
    let (state_rmse, state_max) =
        drift(full.state.snapshot_values(), single.state.snapshot_values())?;
    let continuation =
        full.greedy_token() == single.greedy_token() && logit_rmse <= 0.01 && state_rmse <= 0.01;
    println!("CONTINUATION pass={continuation} top1_chunk={} top1_single={} logit_nrmse={logit_rmse:.8} logit_max_abs={logit_max:.8} state_nrmse={state_rmse:.8} state_max_abs={state_max:.8}",full.greedy_token(),single.greedy_token());
    drop(full);
    drop(single);
    drop(sequential);
    let backend = ScheduledQwen::new(executor, tokenizer.eos());
    let storage = backend.storage();
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 512,
        max_new_tokens: 32,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    let engine = Engine::new(config, storage, backend)?;
    let mut cases = include_str!("../../fixtures/native-heldout.tsv")
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split('\t').collect();
            ensure!(parts.len() == 3, "heldout row");
            Ok((parts[0], parts[1], parts[2]))
        })
        .collect::<Result<Vec<_>>>()?;
    println!(
        "OUTPUT-PROFILE {}",
        if contracts {
            "explicit-yes-no-contract-v1"
        } else {
            "unconstrained-original"
        }
    );
    if contracts {
        for line in include_str!("../../fixtures/native-choice-heldout.tsv").lines() {
            let parts: Vec<_> = line.split('\t').collect();
            ensure!(parts.len() == 3, "choice heldout row");
            cases.push((parts[0], parts[1], parts[2]));
        }
    }
    let mut passed = 0;
    let mut total = 0;
    for chunk in cases.chunks(3) {
        let mut pending = Vec::new();
        for (id, prompt, expected) in chunk {
            pending.push((
                *id,
                *expected,
                if contracts && (*id == "rust-borrow" || id.starts_with("choice-")) {
                    engine.submit_choices(
                        tokenizer.chat(None, prompt)?,
                        32,
                        vec![
                            tokenizer.encode_choice("yes")?,
                            tokenizer.encode_choice("no")?,
                        ],
                    )?
                } else {
                    engine.submit(tokenizer.chat(None, prompt)?, 32)?
                },
            ));
        }
        for (id, expected, request) in pending {
            let result = request.result().await?;
            let text = tokenizer.decode(&result.tokens)?;
            let pass = text.trim() == expected && result.finish_reason == "eos";
            total += 1;
            if pass {
                passed += 1;
            }
            println!(
                "HELDOUT id={id} pass={pass} finish={} expected={expected:?} actual={text:?}",
                result.finish_reason
            );
        }
    }
    let stats = engine.shutdown().await?;
    let drain = stats.memory_used == 0 && !stats.admission_closed;
    println!("SUMMARY heldout={passed}/{total} continuation={continuation} drain={drain} scope=limited-smoke production_qualified=false");
    ensure!(
        continuation && passed == total && drain,
        "native qualification gate failed; retain report and do not promote"
    );
    Ok(())
}
