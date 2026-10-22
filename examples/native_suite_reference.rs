//! Offline frozen-suite reference capture. Expected answers never enter execution.
use anyhow::{ensure, Context, Result};
use rust_server::serving::{
    choices::{select, Choices},
    native_mlx::Device,
    native_qwen::QwenExecutor,
    native_tokenizer::NativeTokenizer,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, io::Write, path::PathBuf};

#[derive(Clone, Debug, Serialize)]
struct Case {
    suite: String,
    id: String,
    prompt: String,
    choices: Option<Vec<String>>,
    expected: String,
}
fn rows(text: &str, count: usize, width: usize) -> Result<Vec<Vec<&str>>> {
    let rows: Vec<Vec<_>> = text.lines().map(|s| s.split('\t').collect()).collect();
    ensure!(rows.len() == count, "frozen inventory count");
    let mut seen = BTreeSet::new();
    for r in &rows {
        ensure!(
            r.len() == width && r.iter().all(|s| !s.is_empty()),
            "fixture fields"
        );
        ensure!(seen.insert(r[0]), "duplicate fixture ID");
    }
    Ok(rows)
}
fn inventory() -> Result<Vec<Case>> {
    let raw = rows(include_str!("../fixtures/native-heldout.tsv"), 16, 3)?;
    let extra = rows(include_str!("../fixtures/native-choice-heldout.tsv"), 4, 3)?;
    let domain = rows(include_str!("../fixtures/native-domain-heldout.tsv"), 24, 5)?;
    let mut cases = Vec::new();
    for suite in ["raw", "contract"] {
        for row in raw.iter().chain(if suite == "contract" {
            extra.iter()
        } else {
            [].iter()
        }) {
            cases.push(Case {
                suite: suite.into(),
                id: row[0].into(),
                prompt: row[1].into(),
                choices: (suite == "contract"
                    && (row[0] == "rust-borrow" || row[0].starts_with("choice-")))
                .then(|| vec!["yes".into(), "no".into()]),
                expected: row[2].into(),
            });
        }
    }
    for row in domain {
        ensure!(["rust", "security"].contains(&row[1]), "domain");
        cases.push(Case {
            suite: "domain".into(),
            id: row[0].into(),
            prompt: row[3].into(),
            choices: (row[2] != "-").then(|| row[2].split(',').map(str::to_owned).collect()),
            expected: row[4].into(),
        });
    }
    ensure!(cases.len() == 60, "suite coverage");
    Ok(cases)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 1,
        "native_suite_reference NEW_SCRATCH_DIRECTORY"
    );
    let out = PathBuf::from(&args[0]);
    // Refuse clobbering evidence or reusing stale vectors.
    std::fs::create_dir(&out).context("scratch directory must not already exist")?;
    let cases = inventory()?;
    let executor = QwenExecutor::open(Device::Gpu).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let eos = tokenizer.eos();
    let mut report = std::fs::File::create(out.join("native.jsonl"))?;
    for case in &cases {
        let prompt_ids = tokenizer.chat(None, &case.prompt)?;
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
        let mut state = executor.empty_state();
        let mut last = None;
        for chunk in prompt_ids.chunks(8) {
            let result = executor.step(&state, chunk.to_vec()).await?;
            state = result.state.clone();
            last = Some(result);
        }
        let mut result = last.context("empty prompt")?;
        let choices = paths
            .as_ref()
            .map(|p| Choices::new(p.clone(), result.logits.len() as u32, &eos, 32))
            .transpose()?;
        ensure!(
            result.logits.iter().all(|v| v.is_finite()),
            "nonfinite logits"
        );
        let first_bytes: Vec<_> = result.logits.iter().flat_map(|v| v.to_le_bytes()).collect();
        let name = format!("{}-{}.f32le", case.suite, case.id);
        std::fs::write(out.join(&name), &first_bytes)?;
        let mut ranked: Vec<_> = (0..result.logits.len()).collect();
        ranked.sort_by(|a, b| {
            result.logits[*b]
                .total_cmp(&result.logits[*a])
                .then(a.cmp(b))
        });
        let top: Vec<_> = ranked
            .iter()
            .take(16)
            .map(|id| (*id, result.logits[*id]))
            .collect();
        let mut generated = Vec::new();
        let mut finish = "length";
        for _ in 0..32 {
            let allowed = choices.as_ref().map(|c| c.allowed(&generated, &eos));
            let next = select(&result.logits, allowed.as_deref())?;
            generated.push(next);
            if eos.contains(&next) {
                finish = "eos";
                break;
            }
            if generated.len() < 32 {
                result = executor.step(&result.state, vec![next]).await?;
            }
        }
        let text = tokenizer.decode(&generated)?;
        let record = serde_json::json!({"case":case,"identity":executor.manifest().identity,
            "profile":"native-f32", "prompt_ids":prompt_ids,"choice_paths":paths,
            "contract":choices.as_ref().map(Choices::fingerprint),"eos":eos,
            "generated":generated,"text":text,"finish":finish,
            "quality_pass":finish=="eos" && text.trim()==case.expected,
            "first_logits_file":name,"first_logits_sha256":digest(&first_bytes),"top16":top});
        writeln!(report, "{record}")?;
        report.flush()?;
        eprintln!("CAPTURE {} {}", case.suite, case.id);
    }
    let stats = executor.shutdown().await?;
    ensure!(stats.leases == 0, "reference capture failed to drain");
    std::fs::write(out.join("drain.txt"), "leases=0 cases=60\n")?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_inventory_and_execution_contracts() -> Result<()> {
        let cases = inventory()?;
        for (suite, count, contracts) in [("raw", 16, 0), ("contract", 20, 5), ("domain", 24, 19)] {
            let selected: Vec<_> = cases.iter().filter(|c| c.suite == suite).collect();
            assert_eq!(selected.len(), count);
            assert_eq!(
                selected.iter().filter(|c| c.choices.is_some()).count(),
                contracts
            );
        }
        let raw = cases
            .iter()
            .find(|c| c.suite == "raw" && c.id == "rust-borrow")
            .unwrap();
        let contract = cases
            .iter()
            .find(|c| c.suite == "contract" && c.id == "rust-borrow")
            .unwrap();
        assert_eq!(raw.prompt, contract.prompt);
        assert_eq!(raw.expected, contract.expected);
        assert!(raw.choices.is_none());
        assert_eq!(contract.choices.as_ref().unwrap(), &["yes", "no"]);
        Ok(())
    }
    #[test]
    fn malformed_fixture_inventory_rejects() {
        for (s, n, w) in [
            ("a\tb\tc\n", 2, 3),
            ("a\tb\tc\na\td\te", 2, 3),
            ("a\t\tc", 1, 3),
            ("a\tb", 1, 3),
        ] {
            assert!(rows(s, n, w).is_err());
        }
    }
}
