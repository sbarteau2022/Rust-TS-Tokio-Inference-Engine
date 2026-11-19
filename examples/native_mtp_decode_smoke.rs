//! Real-text transactional MTP correctness gate, not a serving benchmark.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mlx::Device,
    native_qwen::{ExecutionTuning, QwenDeviceExecutor},
    native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{io::Write, path::Path, time::Instant};

struct Progress {
    file: std::fs::File,
    start: Instant,
}
impl Progress {
    fn mark(&mut self, phase: &str, width: usize, iteration: usize) -> Result<()> {
        writeln!(
            self.file,
            "{}",
            serde_json::json!({"phase": phase, "width": width, "iteration": iteration, "elapsed_ms": self.start.elapsed().as_secs_f64() * 1000.0})
        )?;
        self.file.flush()?;
        Ok(())
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_decode_smoke NEW_DIRECTORY")?;
    let graph = std::env::args().nth(2).unwrap_or_else(|| "eager".into());
    let group: u32 = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "40".into())
        .parse()?;
    ensure!(
        matches!(graph.as_str(), "eager" | "compiled-decode") && matches!(group, 1 | 4 | 10 | 40),
        "execution profile"
    );
    std::fs::create_dir(&out)?;
    let mut progress = Progress {
        file: std::fs::File::create(Path::new(&out).join("progress.jsonl"))?,
        start: Instant::now(),
    };
    progress.mark("open.begin", 0, 0)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let e = QwenDeviceExecutor::open_mtp_configured(
        Device::Gpu,
        16,
        1536 * 1024 * 1024,
        ExecutionTuning {
            layer_group: group,
            fused_recurrent: true,
            compiled_decode: graph == "compiled-decode",
            ..ExecutionTuning::default()
        },
    )
    .await?;
    progress.mark("open.end", 0, 0)?;
    let mut cases = Vec::new();
    for width in [1, 2, 4, 8] {
        let prompt = tokenizer.chat(None, "Complete: 1, 2, 3,")?;
        progress.mark("start.begin", width, 0)?;
        let mut cursor = e.mtp_start(&prompt).await?;
        progress.mark("start.end", width, 0)?;
        let memory = e.stats().await?;
        writeln!(
            progress.file,
            "{}",
            serde_json::json!({"phase":"start.memory", "width":width,"active_bytes":memory.mlx_active_bytes,"peak_bytes":memory.mlx_peak_bytes,"compiled_entries":memory.compiled_entries})
        )?;
        progress.file.flush()?;
        ensure!(
            e.mtp_decode(&cursor, 0, &tokenizer.eos()).await.is_err(),
            "zero width"
        );
        ensure!(
            e.mtp_decode(&cursor, 9, &tokenizer.eos()).await.is_err(),
            "oversized width"
        );
        ensure!(
            e.mtp_decode_bounded(&cursor, width, 0, &tokenizer.eos())
                .await
                .is_err(),
            "zero output budget"
        );
        ensure!(
            e.mtp_decode(&cursor, width, &[u32::MAX]).await.is_err(),
            "invalid EOS"
        );
        progress.mark("limited.begin", width, 0)?;
        let limited = e
            .mtp_decode_bounded(&cursor, width, 1, &tokenizer.eos())
            .await?;
        ensure!(
            limited.emitted.len() == 1
                && limited.reused_target_tokens == 1
                && limited.target_calls == 1,
            "one-token budget/accepted terminal window"
        );
        drop(limited);
        progress.mark("limited.end", width, 0)?;
        let parent = cursor.clone();
        let original = e.snapshot(parent.target().state()).await?;
        let mut emitted = Vec::new();
        let (mut proposed, mut accepted, mut target_calls, mut draft_calls) = (0, 0, 0, 0);
        let (mut reused, mut replayed, mut forwarded) = (0, 0, 0);
        for iteration in 0..4 {
            progress.mark("decode.begin", width, iteration)?;
            let tx = e.mtp_decode(&cursor, width, &tokenizer.eos()).await?;
            progress.mark("decode.end", width, iteration)?;
            ensure!(
                tx.reused_target_tokens
                    == if tx.learned_accepted == tx.learned_proposed {
                        tx.learned_proposed + 1
                    } else {
                        0
                    },
                "reuse requires complete acceptance"
            );
            ensure!(
                tx.replayed_target_tokens + tx.reused_target_tokens == tx.emitted.len(),
                "committed coverage"
            );
            ensure!(
                tx.target_forward_tokens == tx.learned_proposed + 1 + tx.replayed_target_tokens,
                "forward accounting"
            );
            #[cfg(feature = "debug-validation")]
            {
                let reference = e
                    .mtp_decode_replay(&cursor, width, &tokenizer.eos())
                    .await?;
                ensure!(
                    reference.emitted == tx.emitted
                        && reference.learned_accepted == tx.learned_accepted,
                    "reuse/reference decision"
                );
                ensure!(
                    reference.target_forward_tokens
                        == tx.target_forward_tokens + tx.reused_target_tokens,
                    "replay/reference work savings"
                );
                let a = e.snapshot(reference.cursor.target().state()).await?;
                let b = e.snapshot(tx.cursor.target().state()).await?;
                let err = (a
                    .iter()
                    .zip(&b)
                    .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
                    .sum::<f64>()
                    / a.iter()
                        .map(|a| (*a as f64).powi(2))
                        .sum::<f64>()
                        .max(1e-30))
                .sqrt();
                ensure!(err <= 1e-5, "reuse/reference unified state {err}");
            }
            progress.mark("reference.begin", width, iteration)?;
            // Independent sequential target decode from the identical committed parent.
            let mut baseline = cursor.target().clone();
            for &token in &tx.emitted {
                let step = e
                    .greedy_verify(&baseline, Vec::new(), &tokenizer.eos())
                    .await?;
                ensure!(step.emitted == [token], "MTP greedy output mismatch");
                baseline = step.cursor;
            }
            let a = e.snapshot(baseline.state()).await?;
            let b = e.snapshot(tx.cursor.target().state()).await?;
            let relative = (a
                .iter()
                .zip(&b)
                .map(|(x, y)| (*x as f64 - *y as f64).powi(2))
                .sum::<f64>()
                / a.iter()
                    .map(|x| (*x as f64).powi(2))
                    .sum::<f64>()
                    .max(1e-30))
            .sqrt();
            let maximum = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (f64::from(*x) - f64::from(*y)).abs())
                .fold(0f64, f64::max);
            ensure!(
                relative <= 1e-5 && maximum <= 1e-3,
                "MTP committed state drift relative={relative} max={maximum}"
            );
            progress.mark("reference.end", width, iteration)?;
            proposed += tx.learned_proposed;
            accepted += tx.learned_accepted;
            target_calls += tx.target_calls;
            draft_calls += tx.draft_calls;
            reused += tx.reused_target_tokens;
            replayed += tx.replayed_target_tokens;
            forwarded += tx.target_forward_tokens;
            emitted.extend(tx.emitted);
            cursor = tx.cursor;
            if cursor.target().terminal() {
                break;
            }
        }
        let after = e.snapshot(parent.target().state()).await?;
        ensure!(
            original
                .iter()
                .zip(after)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "mutated parent"
        );
        cases.push(serde_json::json!({"width":width,"tokens":emitted,"text":tokenizer.decode(&emitted)?,"learned_proposed":proposed,"learned_accepted":accepted,"target_calls":target_calls,"draft_calls":draft_calls,"reused_target_tokens":reused,"replayed_target_tokens":replayed,"target_forward_tokens":forwarded}));
        drop(cursor);
        drop(parent);
    }
    progress.mark("drain.begin", 0, 0)?;
    let stats = e.shutdown().await?;
    progress.mark("drain.end", 0, 0)?;
    ensure!(stats.logical_bytes == 0, "MTP decode drain");
    let report = serde_json::json!({"pass":true,"graph_mode":graph,"layer_group":group,"cases":cases,"profile":"accepted-target-window-reuse","same_parent_replay_reference":cfg!(feature="debug-validation"),"serving_qualified":false,"throughput_claim":false});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
