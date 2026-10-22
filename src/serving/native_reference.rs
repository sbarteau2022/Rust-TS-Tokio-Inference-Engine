//! Frozen semantic-case diagnostic at the text boundary; not a full-model oracle.
use super::{
    model_ir::{ModelStepIr, StepKind},
    native_extended::relative,
    native_mlx::Device,
    native_qwen::QwenExecutor,
    native_tokenizer::NativeTokenizer,
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};

pub async fn run(device: Device, full: bool) -> Result<()> {
    let line = include_str!("../../fixtures/native-domain-heldout.tsv")
        .lines()
        .find(|line| line.starts_with("r10\t"))
        .context("frozen cancellation case missing")?;
    let fields: Vec<_> = line.split('\t').collect();
    ensure!(fields.len() == 5 && fields[2] == "yes,no", "case contract");
    let executor = QwenExecutor::open(device).await?;
    let tokenizer = NativeTokenizer::load(executor.manifest())?;
    let ids = tokenizer.chat(None, fields[3])?;
    let yes = tokenizer.encode_choice("yes")?;
    let no = tokenizer.encode_choice("no")?;
    ensure!(
        yes.len() == 1 && no.len() == 1 && yes != no,
        "single-token alternatives required"
    );
    println!("REFERENCE identity={} device={device:?} case=r10 fixture_sha256={:x} prompt_tokens={} independent_full_model=false full_coverage={full} scope=operator-checks", executor.manifest().identity, Sha256::digest(line.as_bytes()), ids.len());
    let split = (ids.len() - 1) / 8 * 8;
    let mut base = executor.empty_state();
    for tokens in ids[..split].chunks(8) {
        base = executor.step(&base, tokens.to_vec()).await?.state;
    }
    let tail = &ids[split..];
    let plain = executor.step(&base, tail.to_vec()).await?;
    let checked = if full {
        let mut state = executor.empty_state();
        let mut last = None;
        for tokens in ids.chunks(8) {
            let step = executor
                .execute(
                    &state,
                    ModelStepIr::compile(
                        StepKind::Prefill,
                        state.position(),
                        tokens.to_vec(),
                        248320,
                    )?
                    .with_full_reference(),
                )
                .await?;
            state = step.state.clone();
            last = Some(step);
        }
        last.context("empty checked prompt")?
    } else {
        executor
            .execute(
                &base,
                ModelStepIr::compile(StepKind::Prefill, split, tail.to_vec(), 248320)?
                    .with_projection_outputs(vec![yes[0], no[0]])?
                    .with_operator_checks(),
            )
            .await?
    };
    let controls = relative(&plain.logits, &checked.logits)?.max(relative(
        plain.state.snapshot_values(),
        checked.state.snapshot_values(),
    )?);
    let mut single = base.clone();
    let mut final_single = None;
    for token in tail {
        let result = executor.step(&single, vec![*token]).await?;
        single = result.state.clone();
        final_single = Some(result);
    }
    let single = final_single.context("empty tail")?;
    let drift = relative(&plain.logits, &single.logits)?.max(relative(
        plain.state.snapshot_values(),
        single.state.snapshot_values(),
    )?);
    let yes_logit = plain.logits[yes[0] as usize];
    let no_logit = plain.logits[no[0] as usize];
    let choice = if yes_logit > no_logit || (yes_logit == no_logit && yes[0] < no[0]) {
        "yes"
    } else {
        "no"
    };
    println!("REFERENCE-CONTROL checked_unchecked_nrmse={controls:.9} chunk_single_nrmse={drift:.9} tail_tokens={}", tail.len());
    println!("SEMANTIC case=r10 yes_id={} no_id={} yes_logit={yes_logit:.9} no_logit={no_logit:.9} no_minus_yes={:.9} legal_choice={choice} expected={} pass={}",yes[0],no[0],no_logit-yes_logit,fields[4],choice==fields[4]);
    let resources = executor.shutdown().await?;
    ensure!(resources.leases == 0, "reference work did not drain");
    ensure!(controls <= 1e-6, "projection probes perturb execution");
    ensure!(drift <= 0.01, "semantic case chunk/single mismatch");
    ensure!(
        choice == fields[4],
        "frozen semantic failure remains; no full-model promotion"
    );
    Ok(())
}
