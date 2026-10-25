//! Offline native inventory/tokenization export for a standalone C++ control.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    native_mtp::MtpSidecar, native_tokenizer::NativeTokenizer, weights::Manifest,
};
use std::{io::Write, path::Path};
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(args.len() == 1, "native_cpp_plan NEW_PLAN.txt");
    let m = Manifest::read(Path::new(
        include_str!("../fixtures/native-model.path").trim(),
    ))?;
    MtpSidecar::read(&m)?;
    let tokenizer = NativeTokenizer::load(&m)?;
    let frozen: serde_json::Value = serde_json::from_str(include_str!(
        "../evidence/native-mtp-stream-matrix/raw/references.json"
    ))?;
    ensure!(
        frozen["identity"].as_str() == Some(&m.identity),
        "frozen target identity"
    );
    let refs = frozen["tokens"].as_array().context("frozen token rows")?;
    let prompts = include_str!("../fixtures/native-speculative-prompts.tsv")
        .lines()
        .collect::<Vec<_>>();
    ensure!(prompts.len() == 4 && refs.len() == 4, "four-prompt control");
    let mut text = String::new();
    use std::fmt::Write as _;
    writeln!(
        &mut text,
        "RS_CPP_BENCH_V1\n{}\n{} {}",
        m.identity, m.quantization.group, m.quantization.bits
    )?;
    let shards = m
        .shards
        .iter()
        .filter(|p| p.file_name().unwrap() != "model-mtp.safetensors")
        .collect::<Vec<_>>();
    ensure!(shards.len() == 4, "four target shards");
    writeln!(&mut text, "{}", shards.len())?;
    for p in shards {
        writeln!(&mut text, "{:?}", p.to_str().context("UTF-8 shard path")?)?;
    }
    writeln!(
        &mut text,
        "{:?}",
        m.directory
            .join("model-mtp.safetensors")
            .to_str()
            .context("UTF-8 sidecar path")?
    )?;
    writeln!(&mut text, "{}", m.overrides.len())?;
    for (name, q) in &m.overrides {
        writeln!(&mut text, "{name:?} {} {}", q.group, q.bits)?;
    }
    let eos = tokenizer.eos();
    writeln!(&mut text, "{}", eos.len())?;
    for t in eos {
        write!(&mut text, "{t} ")?;
    }
    writeln!(&mut text)?;
    writeln!(&mut text, "4")?;
    for (line, reference) in prompts.iter().zip(refs) {
        let (name, prompt) = line.split_once('\t').context("prompt fixture")?;
        let ids = tokenizer.chat(None, prompt)?;
        let reference = reference.as_array().context("reference IDs")?;
        ensure!(reference.len() >= 32, "reference length");
        writeln!(&mut text, "{name:?} {} 32", ids.len())?;
        for t in ids {
            write!(&mut text, "{t} ")?;
        }
        writeln!(&mut text)?;
        for t in &reference[..32] {
            write!(&mut text, "{} ", t.as_u64().context("reference integer")?)?;
        }
        writeln!(&mut text)?;
    }
    m.verify_stamps()?;
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[0])?;
    out.write_all(text.as_bytes())?;
    Ok(())
}
