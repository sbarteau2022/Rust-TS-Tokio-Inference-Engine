//! Offline margin diagnostics; no server configuration or decoding defaults change.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mlx::Device, native_qwen::QwenExecutor, native_tokenizer::NativeTokenizer,
};
fn logprobs(logits: &[f32], ids: &[u32]) -> Result<Vec<f64>> {
    ensure!(
        !logits.is_empty() && logits.iter().all(|v| v.is_finite()),
        "finite logits required"
    );
    ensure!(
        !ids.is_empty() && ids.len() <= 8 && ids.iter().all(|id| (*id as usize) < logits.len()),
        "candidate bounds"
    );
    let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    let normalizer = maximum
        + logits
            .iter()
            .map(|v| (*v as f64 - maximum).exp())
            .sum::<f64>()
            .ln();
    Ok(ids
        .iter()
        .map(|id| logits[*id as usize] as f64 - normalizer)
        .collect())
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.is_empty() || args == ["--probes"],
        "native_discriminators [--probes]"
    );
    let mut cases = Vec::new();
    for (fixture, wanted, column) in [
        (
            include_str!("../fixtures/native-domain-heldout.tsv"),
            "r10",
            3,
        ),
        (
            include_str!("../fixtures/native-heldout.tsv"),
            "rust-borrow",
            1,
        ),
    ] {
        let row = fixture
            .lines()
            .find(|s| s.starts_with(&format!("{wanted}\t")))
            .context("frozen case missing")?;
        let fields: Vec<_> = row.split('\t').collect();
        cases.push((wanted, *fields.get(column).context("case prompt missing")?));
    }
    if !args.is_empty() {
        cases.clear();
        for line in include_str!("../fixtures/native-interpretation-probes.tsv").lines() {
            let fields: Vec<_> = line.split('\t').collect();
            ensure!(fields.len() == 2, "probe fields");
            cases.push((fields[0], fields[1]));
        }
    }
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let words = ["yes", "no", "Yes", "No"];
    let ids = words
        .iter()
        .map(|s| {
            let ids = tokenizer.encode_choice(s)?;
            ensure!(ids.len() == 1, "single-token candidates required");
            Ok(ids[0])
        })
        .collect::<Result<Vec<_>>>()?;
    println!("NATIVE identity={} activation=f32 kv=f32 speculation=false prefix_reuse=false embedding_correction=build-bound", executor.manifest().identity);
    for (id, prompt) in cases {
        let tokens = tokenizer.chat(None, prompt)?;
        println!("INPUT case={id} prompt={prompt:?} ids={tokens:?}");
        let mut state = executor.empty_state();
        let mut last = None;
        for chunk in tokens.chunks(8) {
            let result = executor.step(&state, chunk.to_vec()).await?;
            state = result.state.clone();
            last = Some(result);
        }
        let mut result = last.context("empty prompt")?;
        let lp = logprobs(&result.logits, &ids)?;
        for (slot, word) in words.iter().enumerate() {
            println!(
                "SCORE case={id} candidate={word} token={} logit={:.9} logprob={:.9}",
                ids[slot], result.logits[ids[slot] as usize], lp[slot]
            );
        }
        println!(
            "MARGIN case={id} no_minus_yes={:.9} capital_No_minus_no={:.9}",
            lp[1] - lp[0],
            lp[3] - lp[1]
        );
        let eos = tokenizer.eos();
        let mut output = Vec::new();
        let mut finish = "length";
        for _ in 0..32 {
            let next = result.greedy_token();
            output.push(next);
            if eos.contains(&next) {
                finish = "eos";
                break;
            }
            result = executor.step(&result.state, vec![next]).await?;
        }
        println!(
            "OUTPUT case={id} text={:?} tokens={output:?} finish={finish}",
            tokenizer.decode(&output)?
        );
    }
    let stats = executor.shutdown().await?;
    ensure!(stats.leases == 0, "native diagnostic did not drain");
    println!("DRAIN leases=0");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable_margins_and_offset_invariance() -> Result<()> {
        let a = logprobs(&[-1000., -999., -998.], &[0, 1])?;
        let b = logprobs(&[0., 1., 2.], &[0, 1])?;
        assert!((a[1] - a[0] - 1.).abs() < 1e-12);
        for (a, b) in a.iter().zip(b) {
            assert!((a - b).abs() < 1e-12);
        }
        Ok(())
    }
    #[test]
    fn malformed_margin_inputs_reject() {
        for (v, ids) in [
            (vec![], vec![0]),
            (vec![f32::NAN], vec![0]),
            (vec![f32::INFINITY], vec![0]),
            (vec![0.], vec![1]),
            (vec![0.], vec![]),
            (vec![0.], vec![0; 9]),
        ] {
            assert!(logprobs(&v, &ids).is_err());
        }
    }
}
