//! Designated checkpoint tokenizer boundary; no GPU or alternate model.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{native_tokenizer::NativeTokenizer, weights::Manifest};
use std::path::Path;
fn main() -> Result<()> {
    let out = std::env::args()
        .nth(1)
        .context("native_text_stream_smoke NEW_DIRECTORY")?;
    std::fs::create_dir(&out)?;
    let manifest = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let mut rows = Vec::new();
    for text in [
        "résumé 😀 日本語 中文 العربية",
        "e\u{301} 👩‍💻 \r\n\t end  ",
        "literal replacement � followed by x",
        "<|im_end|>résumé<|im_end|>",
        "",
        "```rust\nfn main() { println!(\"hello 🌍\"); }\n```",
        "\0\u{0001}\\\"\n",
    ] {
        let ids = tokenizer.encode(text)?;
        // Test every termination boundary, including a possible unfinished UTF-8 sequence.
        for length in 0..=ids.len() {
            let expected = tokenizer.decode(&ids[..length])?;
            let mut stream = tokenizer.text_stream()?;
            let mut deltas = Vec::new();
            let mut published = String::new();
            for id in &ids[..length] {
                if let Some(delta) = stream.push(*id)? {
                    published.push_str(&delta);
                    ensure!(
                        expected.starts_with(&published),
                        "published prefix mismatch"
                    );
                    deltas.push(delta);
                }
            }
            let tail = stream.finish()?;
            published.push_str(&tail);
            ensure!(
                published == expected && stream.text() == expected,
                "stream/full mismatch"
            );
            rows.push(serde_json::json!({"source":text,"prefix_tokens":length,"ids":&ids[..length],"deltas":deltas,"tail":tail,"text":expected}));
        }
    }
    let report = serde_json::json!({"pass":true,"scope":"designated checkpoint tokenizer incremental text, every token termination boundary; no GPU/HTTP qualification","cases":rows.len(),"rows":rows});
    std::fs::write(
        Path::new(&out).join("summary.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("PASS native text prefixes={}", report["cases"]);
    Ok(())
}
