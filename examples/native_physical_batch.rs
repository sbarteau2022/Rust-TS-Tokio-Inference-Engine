//! Offline real-model gate for batch-shaped decode; not HTTP or throughput qualification.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    choices,
    native_mlx::Device,
    native_qwen::{ExecutionTuning, QwenDeviceExecutor},
    native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{path::Path, sync::Arc};
fn compare(a: &[f32], b: &[f32]) -> Result<serde_json::Value> {
    ensure!(a.len() == b.len() && !a.is_empty(), "comparison geometry");
    let (mut delta, mut scale, mut maximum, mut unequal) = (0f64, 0f64, 0f64, 0usize);
    for (&a, &b) in a.iter().zip(b) {
        let d = f64::from(a) - f64::from(b);
        delta += d * d;
        scale += f64::from(b).powi(2);
        maximum = maximum.max(d.abs());
        unequal += usize::from(a.to_bits() != b.to_bits());
    }
    let nrmse = (delta / scale.max(1e-30)).sqrt();
    Ok(
        serde_json::json!({"nrmse":nrmse,"max_abs":maximum,"unequal_bits":unequal,"elements":a.len(),"pass":nrmse<=1e-5&&maximum<=1e-3}),
    )
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_physical_batch NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let group: u32 = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "1".into())
        .parse()?;
    let mode = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "reference".into());
    ensure!(
        matches!(mode.as_str(), "reference" | "fused"),
        "recurrent kernel mode"
    );
    let graph = std::env::args().nth(4).unwrap_or_else(|| "eager".into());
    ensure!(
        matches!(graph.as_str(), "eager" | "compiled-decode"),
        "graph mode"
    );
    let expert = std::env::args()
        .nth(5)
        .unwrap_or_else(|| "reference".into());
    let expert_kernel = match expert.as_str() {
        "reference" => 0,
        "sorted" => 1,
        "fused" => 2,
        _ => anyhow::bail!("expert kernel"),
    };
    let executor = Arc::new(
        QwenDeviceExecutor::open_configured(
            Device::Gpu,
            32,
            // Fixed-shape (context-cap) states are larger than the old
            // position-sized accounting; the resident budget is sized for them.
            4 * 1024 * 1024 * 1024,
            ExecutionTuning {
                layer_group: group,
                fused_recurrent: mode == "fused",
                execution_timing: false,
                compiled_decode: graph == "compiled-decode",
                operator_profile: false,
                fused_attention: false,
                expert_kernel,
                validation_mode: 2,
                activation_precision: 1,
            },
        )
        .await?,
    );
    let texts: Vec<_> = include_str!("../fixtures/native-speculative-prompts.tsv")
        .lines()
        .map(|s| s.split_once('\t').unwrap().1)
        .collect();
    let references: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        "evidence/native-mtp-stream-matrix/raw/references.json",
    )?)?;
    let mut observations = Vec::new();
    let mut passed = true;
    for (prompt, text) in texts.iter().enumerate() {
        let ids = tokenizer.chat(None, text)?;
        let mut prefix = executor.empty_state();
        let mut seed = 0;
        for chunk in ids.chunks(8) {
            let step = executor.step(&prefix, chunk.to_vec()).await?;
            seed = choices::select(&step.logits, None)?;
            prefix = step.state;
        }
        ensure!(
            references["tokens"][prompt][0].as_u64() == Some(seed as u64),
            "scalar seed changed prior reference"
        );
        for batch in 1..=3 {
            let mut scalar = vec![prefix.clone(); batch];
            let mut packed = scalar.clone();
            // Distinct controlled first-input branches, then greedy continuation on each lane.
            let mut tokens: Vec<_> = (0..batch)
                .map(|row| (seed + row as u32 * 17) % 248320)
                .collect();
            for step in 0..4 {
                let mut reference = Vec::new();
                for (parent, token) in scalar.iter().zip(&tokens) {
                    reference.push(executor.step(parent, vec![*token]).await?);
                }
                let result = executor.decode_batch(&packed, tokens.clone()).await?;
                let mut next_tokens = Vec::new();
                for row in 0..batch {
                    let logits = compare(&result[row].logits, &reference[row].logits)?;
                    let a = executor.snapshot(&result[row].state).await?;
                    let b = executor.snapshot(&reference[row].state).await?;
                    let state = compare(&a, &b)?;
                    drop((a, b));
                    let actual = choices::select(&result[row].logits, None)?;
                    let expected = choices::select(&reference[row].logits, None)?;
                    let ok = logits["pass"] == true && state["pass"] == true && actual == expected;
                    passed &= ok;
                    if row == 0 {
                        ensure!(
                            references["tokens"][prompt][step + 1].as_u64() == Some(actual as u64),
                            "greedy branch changed prior reference"
                        );
                    }
                    observations.push(serde_json::json!({"prompt":prompt,"batch":batch,"step":step,"row":row,"position":result[row].state.position(),"token":actual,"expected":expected,"logits":logits,"state":state,"pass":ok}));
                    next_tokens.push(expected);
                }
                scalar = reference.into_iter().map(|r| r.state).collect();
                packed = result.into_iter().map(|r| r.state).collect();
                tokens = next_tokens;
                std::fs::write(
                    format!("{out}/partial.json"),
                    serde_json::to_vec_pretty(&observations)?,
                )?;
            }
            // Same parents in reverse order must map back to the same outputs/state.
            let forward = executor.decode_batch(&packed, tokens.clone()).await?;
            let mut reversed = packed.clone();
            reversed.reverse();
            let mut reversed_tokens = tokens;
            reversed_tokens.reverse();
            let reverse = executor.decode_batch(&reversed, reversed_tokens).await?;
            for row in 0..batch {
                ensure!(
                    forward[row].logits == reverse[batch - 1 - row].logits,
                    "row permutation logits"
                );
                ensure!(
                    executor.snapshot(&forward[row].state).await?
                        == executor.snapshot(&reverse[batch - 1 - row].state).await?,
                    "row permutation state"
                );
            }
            drop((forward, reverse));
            let charge = packed[0].bytes();
            ensure!(
                packed.iter().all(|p| p.bytes() == charge),
                "packed child charges"
            );
            while packed.len() > 1 {
                packed.pop();
            }
            ensure!(packed[0].bytes() == charge, "survivor backing charge");
            drop((packed, scalar));
        }
        ensure!(
            executor.stats().await?.residents == 1,
            "prompt resident drain"
        );
        ensure!(
            executor
                .decode_batch(&[prefix.clone()], vec![248320])
                .await
                .is_err(),
            "invalid token admitted"
        );
        ensure!(
            executor.decode_batch(&[], vec![]).await.is_err(),
            "empty batch admitted"
        );
        let later = executor.step(&prefix, vec![seed]).await?;
        ensure!(
            executor
                .decode_batch(&[prefix.clone(), later.state.clone()], vec![seed, seed])
                .await
                .is_err(),
            "unequal positions admitted"
        );
        drop(later);
        let before = executor.stats().await?;
        let pending = executor.submit_decode_batch(
            &[prefix.clone(), prefix.clone(), prefix.clone()],
            vec![seed; 3],
        )?;
        drop(pending);
        let after = executor.stats().await?;
        ensure!(
            after.steps == before.steps + 3 && after.residents == 1,
            "dropped receiver completion/retirement"
        );
        if prompt == 0 {
            let mut pressure = Vec::new();
            for _ in 0..32 {
                match executor.step(&prefix, vec![seed]).await {
                    Ok(step) => pressure.push(step.state),
                    Err(_) => break,
                }
            }
            let before = executor.stats().await?;
            ensure!(
                executor
                    .decode_batch(&vec![prefix.clone(); 3], vec![seed; 3])
                    .await
                    .is_err(),
                "pressure admitted batch"
            );
            ensure!(
                executor.stats().await?.residents == before.residents,
                "budget rejection leaked reservation"
            );
            drop(pressure);
            drop(
                executor
                    .decode_batch(&vec![prefix.clone(); 3], vec![seed; 3])
                    .await?,
            );
            ensure!(
                executor.stats().await?.residents == 1,
                "pressure recovery drain"
            );
        }
        drop(prefix);
        ensure!(executor.stats().await?.residents == 0, "prompt full drain");
    }
    let live = executor.stats().await?;
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("executor owner"))?
        .shutdown()
        .await?;
    let report = serde_json::json!({"expert_kernel":expert,"pass":passed,"layer_group":group,"recurrent_kernel":mode,"graph_mode":graph,"scope":"native equal-position batch graph, controlled branches then greedy; no HTTP/throughput claim","identity":manifest.identity,"observations":observations,"native_residents":native.residents,"native_bytes":native.logical_bytes,"mlx_active_bytes":live.mlx_active_bytes,"mlx_peak_bytes":live.mlx_peak_bytes});
    std::fs::write(
        format!("{out}/summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    ensure!(
        passed,
        "native batch numerical/token gate failed; observations retained"
    );
    println!("PASS native physical batch graph: scalar token/state parity, row permutations, sibling leases, canceled result drain");
    Ok(())
}
