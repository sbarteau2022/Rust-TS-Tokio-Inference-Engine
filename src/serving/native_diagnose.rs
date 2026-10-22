//! Opt-in diagnostic I/O boundary, never a model qualification certificate.
use super::{
    model_ir::{ModelStepIr, StepKind},
    native_extended::relative,
    native_mlx::Device,
    native_qwen::QwenExecutor,
    native_tokenizer::NativeTokenizer,
};
use anyhow::{ensure, Result};
pub async fn run() -> Result<()> {
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    println!("DIAGNOSE identity={} activation=f32 state=fp32 context=512 window=504..512 trace=layer-stage-token",executor.manifest().identity);
    let cycle = tokenizer.encode(
        "Rust ownership and careful security checks preserve application state across requests. ",
    )?;
    let ids: Vec<_> = cycle.iter().copied().cycle().take(512).collect();
    let mut base = executor.empty_state();
    for part in ids[..504].chunks(8) {
        base = executor.step(&base, part.to_vec()).await?.state;
    }
    let full = executor.step(&base, ids[504..].to_vec()).await?;
    let traced = executor
        .execute(
            &base,
            ModelStepIr::compile(StepKind::Prefill, 504, ids[504..].to_vec(), 248320)?.with_trace(),
        )
        .await?;
    let full_control = relative(&full.logits, &traced.logits)?.max(relative(
        full.state.snapshot_values(),
        traced.state.snapshot_values(),
    )?);
    let mut state = base.clone();
    let mut plain = base.clone();
    let mut traces = Vec::new();
    let mut last = None;
    let mut last_plain = None;
    for token in &ids[504..] {
        let result = executor
            .execute(
                &state,
                ModelStepIr::compile(StepKind::Decode, state.position(), vec![*token], 248320)?
                    .with_trace(),
            )
            .await?;
        state = result.state.clone();
        traces.push(result.trace);
        last = Some((result.logits, result.state));
        let result = executor.step(&plain, vec![*token]).await?;
        plain = result.state.clone();
        last_plain = Some(result);
    }
    let (single_logits, single_state) = last.unwrap();
    let plain = last_plain.unwrap();
    let single_control = relative(&single_logits, &plain.logits)?.max(relative(
        single_state.snapshot_values(),
        plain.state.snapshot_values(),
    )?);
    println!("TRACE-CONTROL chunk_nrmse={full_control:.9} single_nrmse={single_control:.9} trustworthy={}",full_control<=1e-6&&single_control<=1e-6);
    let names = ["input_norm", "mixer", "residual", "moe", "output"];
    let mut first = None;
    for layer in 0..40 {
        for (stage, name) in names.iter().enumerate() {
            let row = layer * 5 + stage;
            let chunk = &traced.trace[row * 8 * 2048..(row + 1) * 8 * 2048];
            let singles: Vec<_> = traces
                .iter()
                .flat_map(|t| t[row * 2048..(row + 1) * 2048].iter().copied())
                .collect();
            let error = relative(chunk, &singles)?;
            let max = chunk
                .iter()
                .zip(&singles)
                .map(|(a, b)| (*a as f64 - *b as f64).abs())
                .fold(0., f64::max);
            if error > 1e-4 && first.is_none() {
                first = Some((layer, *name));
            }
            println!("TRACE layer={layer} stage={name} nrmse={error:.9} max_abs={max:.9}");
        }
    }
    println!("DIAGNOSTIC-SUMMARY first_over_1e_minus_4={first:?} logit_nrmse={:.9} state_nrmse={:.9} independent_reference=false",relative(&full.logits,&plain.logits)?,relative(full.state.snapshot_values(),plain.state.snapshot_values())?);
    ensure!(
        full_control <= 1e-6 && single_control <= 1e-6,
        "trace materially perturbs original execution"
    );
    ensure!(
        relative(&full.logits, &plain.logits)? <= 0.01
            && relative(full.state.snapshot_values(), plain.state.snapshot_values())? <= 0.01,
        "context chunk/single diagnostic gate failed"
    );
    drop(full);
    drop(traced);
    drop(plain);
    drop(state);
    drop(base);
    drop(single_state);
    let resources = executor.shutdown().await?;
    ensure!(
        resources.leases == 0,
        "diagnostic native work did not drain"
    );
    Ok(())
}
