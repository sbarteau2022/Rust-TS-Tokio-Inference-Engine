//! Real-text transactional MTP correctness gate, not a serving benchmark.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mlx::Device, native_qwen::QwenDeviceExecutor, native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::path::Path;
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_decode_smoke NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let e = QwenDeviceExecutor::open_mtp(Device::Gpu, 16, 1536 * 1024 * 1024).await?;
    let mut cases = Vec::new();
    for width in [1, 2, 4, 8] {
        let prompt = tokenizer.chat(None, "Complete: 1, 2, 3,")?;
        let mut cursor = e.mtp_start(&prompt).await?;
        ensure!(
            e.mtp_decode(&cursor, 0, &tokenizer.eos()).await.is_err(),
            "zero width"
        );
        ensure!(
            e.mtp_decode(&cursor, 9, &tokenizer.eos()).await.is_err(),
            "oversized width"
        );
        let parent = cursor.clone();
        let original = e.snapshot(parent.target().state()).await?;
        let mut emitted = Vec::new();
        let (mut proposed, mut accepted, mut target_calls, mut draft_calls) = (0, 0, 0, 0);
        for _ in 0..4 {
            let tx = e.mtp_decode(&cursor, width, &tokenizer.eos()).await?;
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
            ensure!(relative <= 1e-5, "MTP committed state drift {relative}");
            proposed += tx.learned_proposed;
            accepted += tx.learned_accepted;
            target_calls += tx.target_calls;
            draft_calls += tx.draft_calls;
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
        cases.push(serde_json::json!({"width":width,"tokens":emitted,"text":tokenizer.decode(&emitted)?,"learned_proposed":proposed,"learned_accepted":accepted,"target_calls":target_calls,"draft_calls":draft_calls}));
        drop(cursor);
        drop(parent);
    }
    let stats = e.shutdown().await?;
    ensure!(stats.logical_bytes == 0, "MTP decode drain");
    let report = serde_json::json!({"pass":true,"cases":cases,"profile":"authoritative-replay-correctness-only","serving_qualified":false,"throughput_claim":false});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
