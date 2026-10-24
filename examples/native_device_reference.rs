//! Complete frozen-suite device/snapshot preservation gate, sharing one weight store.
#[allow(dead_code)]
#[path = "native_suite_reference.rs"]
mod frozen;
use anyhow::{ensure, Result};
use rust_server::serving::{
    choices::{select, Choices},
    model_ir::{ModelStepIr, StepKind},
    native_mlx::Device,
    native_qwen::{DeviceState, ExecutionTuning, ModelState, QwenDeviceExecutor},
    native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{io::Write, path::Path, time::Instant};
static BITWISE_MATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
fn error(a: &[f32], b: &[f32]) -> Result<(f64, f64)> {
    ensure!(
        !a.is_empty() && a.len() == b.len() && a.iter().chain(b).all(|v| v.is_finite()),
        "comparison geometry/finite"
    );
    if a.iter().zip(b).any(|(x, y)| x.to_bits() != y.to_bits()) {
        BITWISE_MATCH.store(false, std::sync::atomic::Ordering::Relaxed);
    }
    let (mut d, mut r, mut m) = (0., 0., 0f64);
    for (&x, &y) in a.iter().zip(b) {
        let e = x as f64 - y as f64;
        d += e * e;
        r += (x as f64).powi(2);
        m = m.max(e.abs());
    }
    let relative = (d / r.max(1e-30)).sqrt();
    ensure!(
        relative <= 1e-5 && m <= 1e-3,
        "preservation gate relative={relative} max_abs={m}"
    );
    Ok((relative, m))
}
async fn pair(
    executor: &QwenDeviceExecutor,
    reference: &ModelState,
    state: &DeviceState,
    tokens: Vec<u32>,
    maxima: &mut [f64; 6],
    trace_enabled: bool,
) -> Result<(ModelState, DeviceState, Vec<f32>, Vec<f32>)> {
    let ir = ModelStepIr::compile(StepKind::Prefill, state.position(), tokens, 248320)?;
    let ir = if trace_enabled { ir.with_trace() } else { ir };
    let baseline = executor.reference_execute(reference, ir.clone()).await?;
    let next = executor.execute(state, ir).await?;
    let exported = executor.snapshot(&next.state).await?;
    for (i, (a, b)) in [
        (&baseline.logits[..], &next.logits[..]),
        (&baseline.trace[..], &next.trace[..]),
        (executor.reference_values(&baseline.state)?, &exported[..]),
    ]
    .into_iter()
    .enumerate()
    {
        if i == 1 && !trace_enabled {
            continue;
        }
        let (r, m) = error(a, b)?;
        maxima[2 * i] = maxima[2 * i].max(r);
        maxima[2 * i + 1] = maxima[2 * i + 1].max(m);
    }
    Ok((baseline.state, next.state, next.logits, baseline.logits))
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 1 || args.len() == 4,
        "native_device_reference NEW_DIRECTORY [GROUP reference|fused eager|compiled-decode]"
    );
    let group: u32 = args.get(1).map_or(Ok(1), |s| s.parse())?;
    let kernel = args.get(2).map_or("reference", String::as_str);
    let graph = args.get(3).map_or("eager", String::as_str);
    ensure!(
        matches!(kernel, "reference" | "fused") && matches!(graph, "eager" | "compiled-decode"),
        "native tuning mode"
    );
    let trace_enabled = args.len() == 1;
    let out = Path::new(&args[0]);
    std::fs::create_dir(out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let eos = tokenizer.eos();
    let executor = QwenDeviceExecutor::open_configured(
        Device::Gpu,
        8,
        768 * 1024 * 1024,
        ExecutionTuning {
            layer_group: group,
            fused_recurrent: kernel == "fused",
            compiled_decode: graph == "compiled-decode",
            operator_profile: false,
            fused_attention: false,
            expert_kernel: 0,
            execution_timing: false,
            validation_mode: 2,
            activation_precision: 1,
        },
    )
    .await?;
    ensure!(
        executor.empty_mtp_state().is_err(),
        "default executor must not enable MTP"
    );
    let mut log = std::fs::File::create(out.join("suite.jsonl"))?;
    let mut maxima = [0.; 6];
    let mut quality = [0; 3];
    let start = Instant::now();
    for case in frozen::inventory()? {
        let ids = tokenizer.chat(None, &case.prompt)?;
        let paths = case
            .choices
            .as_ref()
            .map(|words| {
                words
                    .iter()
                    .map(|s| tokenizer.encode_choice(s))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;
        let choices = paths
            .as_ref()
            .map(|p| Choices::new(p.clone(), 248320, &eos, 32))
            .transpose()?;
        let mut reference = executor.reference_empty_state();
        let mut state = executor.empty_state();
        let mut logits = Vec::new();
        let mut reference_logits = Vec::new();
        for tokens in ids.chunks(8) {
            (reference, state, logits, reference_logits) = pair(
                &executor,
                &reference,
                &state,
                tokens.to_vec(),
                &mut maxima,
                trace_enabled,
            )
            .await?;
        }
        let mut generated = Vec::new();
        let mut finish = "length";
        for _ in 0..32 {
            let allowed = choices.as_ref().map(|c| c.allowed(&generated, &eos));
            let next = select(&logits, allowed.as_deref())?;
            ensure!(
                next == select(&reference_logits, allowed.as_deref())?,
                "held-out selected token differs from snapshot control"
            );
            generated.push(next);
            if eos.contains(&next) {
                finish = "eos";
                break;
            }
            if generated.len() < 32 {
                (reference, state, logits, reference_logits) = pair(
                    &executor,
                    &reference,
                    &state,
                    vec![next],
                    &mut maxima,
                    trace_enabled,
                )
                .await?;
            }
        }
        let text = tokenizer.decode(&generated)?;
        let pass = finish == "eos" && text.trim() == case.expected;
        let suite = match case.suite.as_str() {
            "raw" => 0,
            "contract" => 1,
            _ => 2,
        };
        quality[suite] += usize::from(pass);
        let row = serde_json::json!({"suite":case.suite,"id":case.id,"prompt_ids":ids,"generated":generated,"text":text,"finish":finish,"quality_pass":pass,"maxima":maxima});
        writeln!(log, "{row}")?;
        log.flush()?;
        eprintln!("DEVICE-AB {} {}", case.suite, case.id);
    }

    let before = executor.stats().await?;
    ensure!(before.uploaded_bytes == 0, "device state upload detected");
    let drain = executor.shutdown().await?;
    ensure!(
        drain.residents == 0 && drain.logical_bytes == 0 && drain.invalidated_at_shutdown == 0,
        "suite drain"
    );
    let summary = serde_json::json!({"pass":quality==[15,20,23],"identity":manifest.identity,"cases":60,"layer_group":group,"recurrent_kernel":kernel,"graph_mode":graph,"trace_compared":trace_enabled,"compiled_entries":before.compiled_entries,"compiled_misses":before.compiled_misses,"compiled_calls":before.compiled_calls,"quality":quality,"maxima_order":["logit_nrmse","logit_max_abs","trace_nrmse","trace_max_abs","state_nrmse","state_max_abs"],"maxima":maxima,"bitwise_match":BITWISE_MATCH.load(std::sync::atomic::Ordering::Relaxed),"elapsed_seconds":start.elapsed().as_secs_f64(),"steps":before.steps,"diagnostic_exported_bytes":before.exported_bytes,"uploaded_bytes":before.uploaded_bytes,"mlx_peak_bytes_process":before.mlx_peak_bytes,"residents_after_drain":drain.residents,"production_qualified":false});
    std::fs::write(
        out.join("summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    ensure!(quality == [15, 20, 23], "quality changed: {quality:?}");
    println!("{summary}");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparison_rejects_invalid_and_material_changes() {
        for (a, b) in [
            (vec![], vec![]),
            (vec![1.], vec![f32::NAN]),
            (vec![1.], vec![2.]),
            (vec![1.], vec![1., 2.]),
        ] {
            assert!(error(&a, &b).is_err());
        }
        assert_eq!(error(&[1., -2.], &[1., -2.]).unwrap(), (0., 0.));
    }
}
