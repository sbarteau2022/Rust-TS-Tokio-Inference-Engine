//! Completion-group ablation: unchanged snapshot control, then unaudited timed generation.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    choices,
    model_ir::{ModelStepIr, StepKind},
    native_mlx::Device,
    native_qwen::{ExecutionMetrics, ExecutionTuning, QwenDeviceExecutor},
    native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{path::Path, time::Instant};
fn compare(a: &[f32], b: &[f32]) -> Result<serde_json::Value> {
    ensure!(!a.is_empty() && a.len() == b.len(), "comparison shape");
    let (mut d, mut r, mut max, mut unequal) = (0f64, 0f64, 0f64, 0usize);
    for (&a, &b) in a.iter().zip(b) {
        ensure!(a.is_finite() && b.is_finite(), "nonfinite comparison");
        let e = f64::from(a) - f64::from(b);
        d += e * e;
        r += f64::from(b).powi(2);
        max = max.max(e.abs());
        unequal += usize::from(a.to_bits() != b.to_bits());
    }
    let nrmse = (d / r.max(1e-30)).sqrt();
    Ok(
        serde_json::json!({"pass":nrmse<=1e-5&&max<=1e-3,"nrmse":nrmse,"max_abs":max,"unequal_bits":unequal}),
    )
}
fn metrics_delta(before: ExecutionMetrics, after: ExecutionMetrics) -> Result<serde_json::Value> {
    let a = serde_json::to_value(before)?;
    let mut b = serde_json::to_value(after)?;
    for (key, value) in b.as_object_mut().context("metrics object")? {
        let before = a[key].as_u64().context("metrics integer")?;
        let after = value.as_u64().context("metrics integer")?;
        *value = serde_json::json!(after.checked_sub(before).context("metrics decreased")?);
    }
    Ok(b)
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        (2..=5).contains(&args.len()),
        "native_group_qualification NEW_DIRECTORY GROUP [reference|fused] [off|on] [eager|compiled-decode]"
    );
    let group: u32 = args[1].parse()?;
    ensure!(matches!(group, 1 | 4 | 10 | 40), "group bounds");
    let mode = args.get(2).map_or("reference", String::as_str);
    ensure!(
        matches!(mode, "reference" | "fused"),
        "recurrent kernel mode"
    );
    let timing = args.get(3).map_or("off", String::as_str);
    ensure!(matches!(timing, "off" | "on"), "execution timing mode");
    let graph = args.get(4).map_or("eager", String::as_str);
    ensure!(matches!(graph, "eager" | "compiled-decode"), "graph mode");
    let out = Path::new(&args[0]);
    std::fs::create_dir(out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let executor = QwenDeviceExecutor::open_configured(
        Device::Gpu,
        16,
        1024 * 1024 * 1024,
        ExecutionTuning {
            layer_group: group,
            fused_recurrent: mode == "fused",
            execution_timing: timing == "on",
            compiled_decode: graph == "compiled-decode",
        },
    )
    .await?;
    let frozen: serde_json::Value = serde_json::from_slice(&std::fs::read(
        "evidence/native-mtp-stream-matrix/raw/references.json",
    )?)?;
    let mut observations = Vec::new();
    let mut timings = Vec::new();
    let mut passed = true;
    let mut previous_misses = 0;
    for (prompt, line) in include_str!("../fixtures/native-speculative-prompts.tsv")
        .lines()
        .enumerate()
    {
        let (_, text) = line.split_once('\t').context("prompt fixture")?;
        let ids = tokenizer.chat(None, text)?;
        let mut state = executor.empty_state();
        let mut reference = executor.reference_empty_state();
        let mut logits = Vec::new();
        // Snapshot control always completes every layer, independent of the candidate group.
        for tokens in ids
            .chunks(8)
            .map(|v| v.to_vec())
            .chain((0..8).map(|_| Vec::new()))
        {
            let decode = tokens.is_empty();
            let tokens = if decode {
                vec![choices::select(&logits, None)?]
            } else {
                tokens
            };
            let ir = ModelStepIr::compile(StepKind::Prefill, state.position(), tokens, 248320)?;
            let baseline = executor.reference_execute(&reference, ir.clone()).await?;
            let candidate_start = Instant::now();
            let candidate = executor.execute(&state, ir).await?;
            let candidate_seconds = candidate_start.elapsed().as_secs_f64();
            let candidate_stats = executor.stats().await?;
            let compiled_misses_delta = candidate_stats
                .compiled_misses
                .checked_sub(previous_misses)
                .context("compiler misses decreased")?;
            previous_misses = candidate_stats.compiled_misses;
            let values = executor.snapshot(&candidate.state).await?;
            let l = compare(&candidate.logits, &baseline.logits)?;
            let s = compare(&values, executor.reference_values(&baseline.state)?)?;
            let token = choices::select(&candidate.logits, None)?;
            let ok = l["pass"] == true
                && s["pass"] == true
                && token == choices::select(&baseline.logits, None)?;
            passed &= ok;
            observations.push(serde_json::json!({"prompt":prompt,"decode":decode,"candidate_seconds":candidate_seconds,"compiled_misses_delta":compiled_misses_delta,"position":candidate.state.position(),"logits":l,"state":s,"token":token,"pass":ok}));
            std::fs::write(
                out.join("partial.json"),
                serde_json::to_vec_pretty(&observations)?,
            )?;
            state = candidate.state;
            reference = baseline.state;
            logits = candidate.logits;
        }
        drop((state, reference, logits));
        // Time a fresh generation without snapshot/reference work; preserve exact frozen tokens.
        let before_prefill = executor.stats().await?.execution;
        let start = Instant::now();
        let mut state = executor.empty_state();
        let mut logits = Vec::new();
        for chunk in ids.chunks(8) {
            let step = executor.step(&state, chunk.to_vec()).await?;
            state = step.state;
            logits = step.logits;
        }
        let ttft = start.elapsed().as_secs_f64();
        let before_decode = executor.stats().await?.execution;
        let decode_start = Instant::now();
        let mut tokens = Vec::new();
        for i in 0..64 {
            let token = choices::select(&logits, None)?;
            ensure!(
                frozen["tokens"][prompt][i].as_u64() == Some(token as u64),
                "frozen token mismatch prompt={prompt} index={i}"
            );
            tokens.push(token);
            if i < 63 {
                let step = executor.step(&state, vec![token]).await?;
                state = step.state;
                logits = step.logits;
            }
        }
        let decode_seconds = decode_start.elapsed().as_secs_f64();
        let after_decode = executor.stats().await?.execution;
        let prefill_execution = metrics_delta(before_prefill, before_decode)?;
        let decode_execution = metrics_delta(before_decode, after_decode)?;
        timings.push(serde_json::json!({"prompt":prompt,"tokens":tokens,"prefill_execution":prefill_execution,"decode_execution":decode_execution,"ttft_seconds":ttft,"decode_seconds":decode_seconds,"decode_steps":63,"decode_tok_s":63./decode_seconds,"total_seconds":start.elapsed().as_secs_f64()}));
        drop(state);
    }
    let live = executor.stats().await?;
    ensure!(
        live.residents == 0 && live.logical_bytes == 0,
        "residual native state"
    );
    let shutdown = executor.shutdown().await?;
    ensure!(
        shutdown.invalidated_at_shutdown == 0 && shutdown.logical_bytes == 0,
        "shutdown residue"
    );
    let report = serde_json::json!({"pass":passed,"identity":manifest.identity,"layer_group":group,"recurrent_kernel":mode,"execution_timing":timing,"graph_mode":graph,"compiled_entries":live.compiled_entries,"compiled_misses":live.compiled_misses,"compiled_calls":live.compiled_calls,"scope":"four-prompt native parity and 64-token generation; no HTTP/held-out qualification","observations":observations,"timings":timings,"mlx_active_bytes":live.mlx_active_bytes,"mlx_peak_bytes":live.mlx_peak_bytes,"native_residents":live.residents,"native_bytes":live.logical_bytes});
    std::fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    ensure!(passed, "completion-group parity gate failed");
    println!(
        "PASS group={group} peak={} observations={}",
        live.mlx_peak_bytes,
        observations.len()
    );
    Ok(())
}
