//! Persistent designated-model loopback server; policy remains explicit and experimental.
use super::{
    http::{self, TextCodec, TextOutput},
    native_text_stream::NativeTextStream,
    native_tokenizer::NativeTokenizer,
};
use anyhow::{ensure, Result};
use std::{net::SocketAddr, sync::Arc};
pub struct Codec {
    tokenizer: NativeTokenizer,
    identity: String,
}
impl Codec {
    pub fn new(tokenizer: NativeTokenizer, identity: String) -> Self {
        Self {
            tokenizer,
            identity,
        }
    }
}
impl TextOutput for NativeTextStream<'_> {
    fn push(&mut self, id: u32) -> Result<Option<String>> {
        self.push(id)
    }
    fn finish(&mut self) -> Result<String> {
        self.finish()
    }
    fn text(&self) -> &str {
        self.text()
    }
}
impl TextCodec for Codec {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn prepare(&self, text: &str, max: usize) -> Result<Vec<u32>> {
        let ids = self.tokenizer.chat(None, text)?;
        ensure!(ids.len() + max <= 512, "HTTP context cap");
        Ok(ids)
    }
    fn stream(&self) -> Result<Box<dyn TextOutput + '_>> {
        Ok(Box::new(self.tokenizer.text_stream()?))
    }
    fn decode(&self, ids: &[u32]) -> Result<String> {
        self.tokenizer.decode(ids)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Profile {
    Target,
    Mtp,
    Audited,
}
fn profile_flags(flags: &[String]) -> Result<(Profile, Vec<String>)> {
    ensure!(flags.len().is_multiple_of(2), "HTTP flags require pairs");
    let mut profile = None;
    let mut policy = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--execution-profile" {
            ensure!(profile.is_none(), "duplicate execution profile");
            profile = Some(match pair[1].as_str() {
                "target" => Profile::Target,
                "mtp" => Profile::Mtp,
                "mtp-audited" => Profile::Audited,
                _ => anyhow::bail!("execution profile must be target|mtp|mtp-audited"),
            });
        } else {
            policy.extend_from_slice(pair);
        }
    }
    Ok((
        profile.ok_or_else(|| {
            anyhow::anyhow!("required --execution-profile target|mtp|mtp-audited")
        })?,
        policy,
    ))
}
fn seed_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(flags.len().is_multiple_of(2), "seed flags require pairs");
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--seed-mode" {
            ensure!(mode.is_none(), "duplicate seed mode");
            mode = Some(match pair[1].as_str() {
                "eager" => false,
                "deferred" => true,
                _ => anyhow::bail!("seed mode must be eager|deferred"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
pub async fn serve(
    address: SocketAddr,
    body_timeout_ms: u64,
    max_connections: usize,
    flags: &[String],
) -> Result<()> {
    use super::{
        config::Config,
        native_mlx::Device,
        native_mtp_cli::Options,
        native_qwen::{QwenDeviceExecutor, ScheduledDevice, ScheduledMtp, StateMode},
        scheduler::Engine,
        weights::Manifest,
    };
    ensure!(
        address.ip().is_loopback()
            && address.port() != 0
            && (1..=30000).contains(&body_timeout_ms)
            && (1..=64).contains(&max_connections),
        "explicit loopback address/nonzero port and HTTP limits required"
    );
    let (deferred_seed, flags) = seed_flags(flags)?;
    let (execution, flags) = profile_flags(&flags)?;
    ensure!(
        !(deferred_seed && execution == Profile::Target),
        "deferred seed requires an MTP execution profile"
    );
    let options = Options::parse(&flags)?;
    let controller = options.controller()?;
    let manifest = Manifest::read(std::path::Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
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
    let (engine, proposer, seed_metrics) = if execution == Profile::Target {
        let backend = ScheduledDevice::new(executor.clone(), tokenizer.eos(), StateMode::Device);
        (Engine::new(config, backend.storage(), backend)?, None, None)
    } else {
        let mut backend = ScheduledMtp::new(
            controller.clone(),
            executor.clone(),
            tokenizer.eos(),
            options.width,
            "local-native-http".into(),
        )?;
        if deferred_seed {
            backend = backend.with_deferred_seed()?;
        }
        let seed_metrics = backend.seed_metrics();
        let id = backend.proposer_id();
        if execution == Profile::Audited {
            backend = backend.with_exclusive_metadata_audits();
        }
        (
            Engine::new(config, backend.storage(), backend)?,
            Some(id),
            Some(seed_metrics),
        )
    };
    let profile=format!("EXPERIMENTAL-NATIVE-HTTP execution={execution:?} seed_mode={} width={} stream_capacity={} policy={:?} context=511 in_flight=3 body_timeout_ms={body_timeout_ms} max_connections={max_connections} physical_tensor_batching=false production_qualified=false audit_clock={}",if deferred_seed {"deferred"} else {"eager"},options.width,options.capacity,options.policy,if execution==Profile::Audited {"validated-batches"} else {"disabled"});
    let engine = Arc::new(engine);
    let codec = Arc::new(Codec::new(tokenizer, executor.identity().into()));
    let server = http::Server::bind(
        address,
        engine.clone(),
        codec,
        http::Config {
            max_connections,
            body_timeout_ms,
            shutdown_grace_ms: 2000,
            stream_capacity: options.capacity,
            profile,
        },
    )
    .await?;
    println!(
        "READY http://{} identity={}",
        server.address,
        executor.identity()
    );
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {r=tokio::signal::ctrl_c()=>r?,_=terminate.recv()=>{}}
    let transport = server.shutdown().await;
    let scheduler = Arc::try_unwrap(engine)
        .map_err(|_| anyhow::anyhow!("HTTP engine retained"))?
        .shutdown()
        .await;
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("HTTP executor retained"))?
        .shutdown()
        .await;
    let transport = transport?;
    let scheduler = scheduler?;
    let native = native?;
    ensure!(
        transport.active_producers == 0
            && transport.active_connections == 0
            && scheduler.memory_used == 0
            && native.logical_bytes == 0,
        "HTTP shutdown drain"
    );
    if let Some(metrics) = seed_metrics {
        println!("SEED_METRICS {:?}", metrics.snapshot());
    }
    if let Some(id) = proposer {
        let ledger = controller.draft_ledger(id)?;
        println!("DRAFT_LEDGER proposed={} accepted={} known_seed={} committed={} target_calls={} failures={} unfinished={}",ledger.proposed,ledger.accepted,ledger.known_prefix,ledger.committed,ledger.target_calls,ledger.failures,ledger.unfinished);
    }
    println!(
        "DRAIN http={transport:?} scheduler={scheduler:?} native_bytes={} audit_epochs={}",
        native.logical_bytes,
        controller.stats()?.epoch
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_owned).collect()
    }
    #[test]
    fn seed_mode_is_explicit_and_default_remains_eager() {
        assert!(!seed_flags(&[]).unwrap().0);
        assert!(seed_flags(&args("--seed-mode deferred")).unwrap().0);
        for s in [
            "--seed-mode",
            "--seed-mode bad",
            "--seed-mode eager --seed-mode deferred",
        ] {
            assert!(seed_flags(&args(s)).is_err());
        }
        let (deferred, rest) =
            seed_flags(&args("--seed-mode eager --execution-profile mtp-audited")).unwrap();
        assert!(!deferred);
        assert_eq!(rest, args("--execution-profile mtp-audited"));
    }
    #[test]
    fn explicit_profile_controls_fail_closed() {
        for s in [
            "",
            "--execution-profile",
            "--execution-profile bogus",
            "--execution-profile target --execution-profile mtp",
        ] {
            assert!(profile_flags(&args(s)).is_err(), "{s}");
        }
        for (s, p) in [
            ("target", Profile::Target),
            ("mtp", Profile::Mtp),
            ("mtp-audited", Profile::Audited),
        ] {
            let (actual, rest) =
                profile_flags(&args(&format!("--draft-width 4 --execution-profile {s}"))).unwrap();
            assert_eq!(actual, p);
            assert_eq!(rest, args("--draft-width 4"));
        }
    }
}
