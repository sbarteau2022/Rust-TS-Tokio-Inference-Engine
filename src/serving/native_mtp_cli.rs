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
pub(super) struct Options {
    pub(super) width: usize,
    pub(super) capacity: usize,
    pub(super) policy: DraftPolicy,
}
impl Options {
    pub(super) fn parse(flags: &[String]) -> Result<Self> {
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
    pub(super) fn controller(&self) -> Result<Controller> {
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
    run_mode(prompt, max_new, flags, false).await
}
pub async fn run_wire(prompt: &str, max_new: usize, flags: &[String]) -> Result<()> {
    run_mode(prompt, max_new, flags, true).await
}
async fn run_mode(prompt: &str, max_new: usize, flags: &[String], binary: bool) -> Result<()> {
    use super::stream_wire::{Encoder, Frame};
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
    let identity = executor.identity().to_owned();
    let profile=format!("EXPERIMENTAL-NATIVE-MTP width={} stream_capacity={} policy={:?} context=511 in_flight=3 physical_tensor_batching=false http=false production_qualified=false audit_clock=validated-batches",options.width,options.capacity,options.policy);
    if binary {
        eprintln!("identity={identity} {profile}");
    } else {
        println!("identity={identity} {profile}");
    }
    let engine = Engine::new(config, backend.storage(), backend)?;
    let mut wire = if binary {
        Some(Encoder::new(std::io::stdout()))
    } else {
        None
    };
    let (mut published, mut sequence, mut last_us) = (0u32, 0u64, 0u64);
    let mut metadata_written = false;
    let result: Result<_> = async {
        if let Some(w) = &mut wire {
            w.write(&Frame::metadata(identity, profile))?;
            metadata_written = true;
        }
        let mut request = engine.submit_stream(ids, max_new, options.capacity)?;
        let mut text_stream = tokenizer.text_stream()?;
        while let Some(event) = request.next_token().await {
            let delta = text_stream.push(event.token)?.unwrap_or_default();
            let elapsed_us = (event.elapsed_ms * 1000.) as u64;
            if let Some(w) = &mut wire {
                w.write(&Frame {
                    kind: 1,
                    sequence: event.sequence,
                    index: u32::try_from(event.index)?,
                    token_id: event.token,
                    elapsed_us,
                    text: delta,
                    ..Frame::default()
                })?;
            } else {
                if !delta.is_empty() {
                    println!("TEXT_DELTA {}", delta.escape_debug());
                }
                println!(
                    "TOKEN sequence={} index={} id={} elapsed_ms={:.3}",
                    event.sequence, event.index, event.token, event.elapsed_ms
                );
            }
            published += 1;
            sequence = event.sequence;
            last_us = elapsed_us;
        }
        let generation = request.result().await?;
        let tail = text_stream.finish()?;
        let text = tokenizer.decode(&generation.tokens)?;
        ensure!(
            text_stream.text() == text && published as usize == generation.tokens.len(),
            "stream/final text or token mismatch"
        );
        ensure!(
            matches!(generation.finish_reason.as_str(), "length" | "eos"),
            "unknown finish reason"
        );
        Ok((generation, tail, text))
    }
    .await;
    // Every error above drops Request and therefore cancels admission while native
    // completion ownership stays with the scheduler. Always await cleanup.
    let scheduler = engine.shutdown().await.and_then(|s| {
        ensure!(
            s.memory_used == 0 && !s.admission_closed,
            "MTP scheduler drain"
        );
        Ok(())
    });
    let native = match Arc::try_unwrap(executor) {
        Ok(e) => e.shutdown().await.and_then(|s| {
            ensure!(s.logical_bytes == 0, "MTP native drain");
            Ok(())
        }),
        Err(_) => Err(anyhow::anyhow!("MTP executor retained")),
    };
    if binary {
        eprintln!(
            "DRAIN scheduler={} native={}",
            scheduler.is_ok(),
            native.is_ok()
        );
    }
    let drain = scheduler.and(native);
    let result = match (result, drain) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), Ok(())) => Err(e),
        (Ok(_), Err(e)) => Err(e),
        (Err(e), Err(d)) => Err(e.context(format!("cleanup also failed: {d}"))),
    };
    match result {
        Ok((generation, tail, text)) => {
            if let Some(w) = &mut wire {
                let stop = match generation.finish_reason.as_str() {
                    "length" => 1,
                    "eos" => 2,
                    _ => anyhow::bail!("unknown finish reason"),
                };
                w.write(&Frame {
                    kind: 2,
                    sequence: generation.sequence,
                    output_tokens: published,
                    stop,
                    elapsed_us: (generation.elapsed_ms * 1000.) as u64,
                    text: tail,
                    ..Frame::default()
                })?;
            } else {
                if !tail.is_empty() {
                    println!("TEXT_DELTA {}", tail.escape_debug());
                }
                println!("TEXT {text}");
                println!(
                    "FINISH tokens={} reason={} elapsed_ms={:.3}",
                    generation.tokens.len(),
                    generation.finish_reason,
                    generation.elapsed_ms
                );
            }
        }
        Err(error) => {
            if metadata_written {
                if let Some(w) = &mut wire {
                    let _ = w.write(&Frame {
                        kind: 3,
                        sequence,
                        output_tokens: published,
                        elapsed_us: last_us,
                        error_code: 6,
                        text: error.to_string().chars().take(1024).collect(),
                        ..Frame::default()
                    });
                }
            }
            return Err(error);
        }
    }
    if binary {
        eprintln!("AUDIT epochs={}", controller.stats()?.epoch);
    } else {
        println!(
            "AUDIT epochs={} clock=validated-batches pressure=normal metadata_only=true",
            controller.stats()?.epoch
        );
    }
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
