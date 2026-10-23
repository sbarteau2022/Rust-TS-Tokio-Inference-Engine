//! Explicit experimental CLI: typed Rust token events, no Python worker/JSON transport.
use super::{
    config::Config,
    native_mlx::Device,
    native_qwen::{QwenDeviceExecutor, ScheduledMtp},
    native_tokenizer::NativeTokenizer,
    scheduler::Engine,
    weights::Manifest,
};
use crate::cache::{Controller, DraftPolicy, MAX_MASS};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, path::Path, sync::Arc};
pub const USAGE:&str="native-mtp-generate PROMPT MAX_NEW --draft-width N --stream-capacity N --qualification-trials N --probe-epochs N --initial-mass N --admission-mass N --reward-mass N";
struct Options {
    width: usize,
    capacity: usize,
    policy: DraftPolicy,
}
impl Options {
    fn parse(flags: &[String]) -> Result<Self> {
        ensure!(flags.len().is_multiple_of(2), "{USAGE}");
        let mut values = BTreeMap::new();
        for pair in flags.as_chunks::<2>().0 {
            ensure!(
                matches!(
                    pair[0].as_str(),
                    "--draft-width"
                        | "--stream-capacity"
                        | "--qualification-trials"
                        | "--probe-epochs"
                        | "--initial-mass"
                        | "--admission-mass"
                        | "--reward-mass"
                ),
                "unknown MTP flag {}",
                pair[0]
            );
            ensure!(
                values
                    .insert(pair[0].as_str(), pair[1].parse::<u64>()?)
                    .is_none(),
                "duplicate MTP flag"
            );
        }
        let value = |key| {
            values
                .get(key)
                .copied()
                .with_context(|| format!("required {key}; {USAGE}"))
        };
        let width = usize::try_from(value("--draft-width")?)?;
        let capacity = usize::try_from(value("--stream-capacity")?)?;
        ensure!(
            (1..=8).contains(&width) && (1..=128).contains(&capacity),
            "MTP width/stream capacity"
        );
        Ok(Self {
            width,
            capacity,
            policy: DraftPolicy {
                max_proposers: 1,
                work_limit: 3,
                initial_mass: u32::try_from(value("--initial-mass")?)?,
                admission_mass: u32::try_from(value("--admission-mass")?)?,
                reward_mass: u32::try_from(value("--reward-mass")?)?,
                qualification_trials: value("--qualification-trials")?,
                probe_every_epochs: value("--probe-epochs")?,
            },
        })
    }
    fn controller(&self) -> Result<Controller> {
        // Explicit bounded draft-metadata scaffold. It does not supply model KV payloads.
        let c = Controller::new(crate::cache::Config {
            capacity_blocks: 8,
            bytes_per_block: 64,
            max_branches: 64,
            max_declared_tokens: 4096,
            max_leases: 16,
            anchor_fraction_q31: MAX_MASS / 2 + 1,
            dead_threshold: 2,
            rates_q31: [268435456, 1073741824, MAX_MASS],
            pressure_multipliers_q31: [2147483648, 4294967296, 8589934592],
            queue_depth: 4,
            max_wait_epochs: 3,
        })?;
        c.configure_drafts(self.policy.clone())?;
        Ok(c)
    }
}
pub async fn run(prompt: &str, max_new: usize, flags: &[String]) -> Result<()> {
    ensure!(
        !prompt.is_empty() && prompt.len() <= 65536 && (1..=128).contains(&max_new),
        "MTP prompt/output bounds"
    );
    let options = Options::parse(flags)?;
    let controller = options.controller()?;
    let manifest = Manifest::read(Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let ids = tokenizer.chat(None, prompt)?;
    ensure!(
        ids.len() + max_new <= 512,
        "MTP consumed-output context exceeds 512"
    );
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
    let backend = ScheduledMtp::new(
        controller.clone(),
        executor.clone(),
        tokenizer.eos(),
        options.width,
        "local-native-mtp-cli".into(),
    )?
    .with_exclusive_metadata_audits();
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 511,
        max_new_tokens: 128,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    println!("EXPERIMENTAL-NATIVE-MTP identity={} width={} stream_capacity={} policy={:?} context=511 in_flight=3 physical_tensor_batching=false http=false production_qualified=false audit_clock=validated-batches",executor.identity(),options.width,options.capacity,options.policy);
    let engine = Engine::new(config, backend.storage(), backend)?;
    let mut request = engine.submit_stream(ids, max_new, options.capacity)?;
    let mut text_stream = tokenizer.text_stream()?;
    while let Some(event) = request.next_token().await {
        if let Some(delta) = text_stream.push(event.token)? {
            println!("TEXT_DELTA {}", delta.escape_debug());
        }
        println!(
            "TOKEN sequence={} index={} id={} elapsed_ms={:.3}",
            event.sequence, event.index, event.token, event.elapsed_ms
        );
    }
    let generation = request.result().await?;
    let tail = text_stream.finish()?;
    if !tail.is_empty() {
        println!("TEXT_DELTA {}", tail.escape_debug());
    }
    let text = tokenizer.decode(&generation.tokens)?;
    ensure!(text_stream.text() == text, "stream/final text mismatch");
    println!("TEXT {text}");
    println!(
        "FINISH tokens={} reason={} elapsed_ms={:.3}",
        generation.tokens.len(),
        generation.finish_reason,
        generation.elapsed_ms
    );
    let stats = engine.shutdown().await?;
    ensure!(
        stats.memory_used == 0 && !stats.admission_closed,
        "MTP scheduler drain"
    );
    println!(
        "AUDIT epochs={} clock=validated-batches pressure=normal metadata_only=true",
        controller.stats()?.epoch
    );
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("MTP executor retained"))?
        .shutdown()
        .await?;
    ensure!(native.logical_bytes == 0, "MTP native drain");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn flags() -> Vec<String> {
        [
            "--draft-width",
            "4",
            "--stream-capacity",
            "1",
            "--qualification-trials",
            "64",
            "--probe-epochs",
            "2",
            "--initial-mass",
            "2147483647",
            "--admission-mass",
            "268435455",
            "--reward-mass",
            "536870911",
        ]
        .map(str::to_string)
        .to_vec()
    }
    #[test]
    fn explicit_policy_flags_refuse_duplicates_unknown_missing_and_bad_bounds() {
        assert!(Options::parse(&flags()).unwrap().controller().is_ok());
        for key in (0..flags().len()).step_by(2) {
            let mut f = flags();
            f.drain(key..key + 2);
            assert!(Options::parse(&f).is_err());
        }
        for value in ["-1", "18446744073709551616", "junk"] {
            let mut f = flags();
            f[1] = value.into();
            assert!(Options::parse(&f).is_err());
        }
        for (index, value) in [
            (1, "0"),
            (1, "9"),
            (3, "0"),
            (3, "129"),
            (5, "0"),
            (7, "0"),
            (9, "4294967296"),
            (11, "0"),
            (13, "2147483648"),
        ] {
            let mut f = flags();
            f[index] = value.into();
            assert!(Options::parse(&f).and_then(|o| o.controller()).is_err());
        }
        let mut f = flags();
        f.push("--draft-width".into());
        assert!(Options::parse(&f).is_err());
        f.push("4".into());
        assert!(Options::parse(&f).is_err());
        let mut f = flags();
        f[0] = "--surprise".into();
        assert!(Options::parse(&f).is_err());
    }
}
