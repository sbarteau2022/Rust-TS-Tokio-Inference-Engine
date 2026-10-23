//! Alternating Rust-boundary baseline/MTP measurements; single lane, no HTTP.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mlx::Device, native_qwen::QwenDeviceExecutor, native_tokenizer::NativeTokenizer,
    weights::Manifest,
};
use std::{collections::BTreeMap, path::Path, time::Instant};
fn percentile(xs: &[f64], q: f64) -> f64 {
    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);
    xs[((xs.len() - 1) as f64 * q).ceil() as usize]
}
#[tokio::main]
async fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_mtp_bench NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let e = QwenDeviceExecutor::open_mtp(Device::Gpu, 16, 1536 * 1024 * 1024).await?;
    // Warm both execution profiles; exclude model loading, warmup and tokenization.
    let warm = tokenizer.chat(None, "Continue: 1, 2, 3,")?;
    let mut chunks = warm.chunks(8);
    let mut b = e.greedy_start(chunks.next().unwrap().to_vec()).await?;
    for c in chunks {
        b = e.greedy_extend(&b, c.to_vec()).await?;
    }
    drop(e.greedy_verify(&b, vec![], &tokenizer.eos()).await?);
    drop(b);
    let m = e.mtp_start(&warm).await?;
    drop(e.mtp_decode_bounded(&m, 4, 8, &tokenizer.eos()).await?);
    drop(m);
    let mut rows = Vec::new();
    let mut outputs = BTreeMap::new();
    for round in 0..2 {
        for line in include_str!("../fixtures/native-speculative-prompts.tsv").lines() {
            let (name, text) = line.split_once('\t').context("fixture")?;
            let prompt = tokenizer.chat(None, text)?;
            for mtp in if round == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let start = Instant::now();
                let mut greedy = None;
                let mut draft = None;
                if mtp {
                    draft = Some(e.mtp_start(&prompt).await?);
                } else {
                    let mut chunks = prompt.chunks(8);
                    let mut c = e.greedy_start(chunks.next().unwrap().to_vec()).await?;
                    for tokens in chunks {
                        c = e.greedy_extend(&c, tokens.to_vec()).await?;
                    }
                    greedy = Some(c);
                }
                let prefill = start.elapsed().as_secs_f64();
                let decode = Instant::now();
                let mut ids = Vec::new();
                let mut intervals = Vec::new();
                let mut first = 0.;
                let (
                    mut proposed,
                    mut accepted,
                    mut target_calls,
                    mut draft_calls,
                    mut forward_tokens,
                ) = (0, 0, 0, 0, 0);
                while ids.len() < 32 {
                    let receipt = Instant::now();
                    let terminal;
                    if let Some(c) = draft.as_ref() {
                        let tx = e
                            .mtp_decode_bounded(c, 4, 32 - ids.len(), &tokenizer.eos())
                            .await?;
                        proposed += tx.learned_proposed;
                        accepted += tx.learned_accepted;
                        target_calls += tx.target_calls;
                        draft_calls += tx.draft_calls;
                        forward_tokens += tx.target_forward_tokens;
                        ids.extend(tx.emitted);
                        terminal = tx.cursor.target().terminal();
                        draft = Some(tx.cursor);
                    } else {
                        let tx = e
                            .greedy_verify(greedy.as_ref().unwrap(), vec![], &tokenizer.eos())
                            .await?;
                        target_calls += tx.target_calls;
                        forward_tokens += tx.emitted.len();
                        ids.extend(tx.emitted);
                        terminal = tx.cursor.terminal();
                        greedy = Some(tx.cursor);
                    }
                    intervals.push(receipt.elapsed().as_secs_f64() * 1000.);
                    if intervals.len() == 1 {
                        first = start.elapsed().as_secs_f64() * 1000.;
                    }
                    if terminal {
                        break;
                    }
                }
                let elapsed = decode.elapsed().as_secs_f64();
                ensure!(!ids.is_empty(), "empty output");
                if let Some(reference) = outputs.get(name) {
                    ensure!(reference == &ids, "profile/round output drift {name}");
                } else {
                    outputs.insert(name.to_string(), ids.clone());
                }
                rows.push(serde_json::json!({"round":round,"prompt":name,"profile":if mtp {"mtp-width4-reuse"} else {"target-greedy"},"tokens":ids.len(),"ids":ids,"prefill_seconds":prefill,"decode_seconds":elapsed,"decode_tokens_per_second":ids.len() as f64/elapsed,"first_receipt_ms":first,"receipt_interval_p50_ms":percentile(&intervals,0.5),"receipt_interval_p95_ms":percentile(&intervals,0.95),"amortized_decode_ms_per_token":elapsed*1000./ids.len() as f64,"learned_proposed":proposed,"learned_accepted":accepted,"target_calls":target_calls,"draft_calls":draft_calls,"target_forward_tokens":forward_tokens}));
                drop(greedy);
                drop(draft);
                std::fs::write(
                    Path::new(&out).join("partial.json"),
                    serde_json::to_string_pretty(&rows)?,
                )?;
            }
        }
    }
    ensure!(e.shutdown().await?.logical_bytes == 0, "benchmark drain");
    let report = serde_json::json!({"pass":true,"identity":manifest.identity,"rows":rows,"output_congruence":true,"lanes":1,"continuous_batching":false,"http_streaming":false,"output_cap":32,"sampling":"greedy","timing_scope":"warmed Rust API; tokenization/model load excluded; MTP receipt groups tokens, so receipt interval is not token streaming ITL","production_qualified":false});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{report}");
    Ok(())
}
