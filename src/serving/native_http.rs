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
fn batching_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(
        flags.len().is_multiple_of(2),
        "batching flags require pairs"
    );
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--tensor-batching" {
            ensure!(mode.is_none(), "duplicate tensor batching mode");
            mode = Some(match pair[1].as_str() {
                "scalar" => false,
                "physical" => true,
                _ => anyhow::bail!("tensor batching must be scalar|physical"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
fn layer_group_flags(flags: &[String]) -> Result<(u32, Vec<String>)> {
    ensure!(
        flags.len().is_multiple_of(2),
        "layer group flags require pairs"
    );
    let mut group = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--layer-group" {
            ensure!(group.is_none(), "duplicate layer group");
            group = Some(match pair[1].as_str() {
                "1" => 1,
                "4" => 4,
                "10" => 10,
                "40" => 40,
                _ => anyhow::bail!("layer group must be 1|4|10|40"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((group.unwrap_or(1), rest))
}
fn recurrent_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(
        flags.len().is_multiple_of(2),
        "recurrent flags require pairs"
    );
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--recurrent-kernel" {
            ensure!(mode.is_none(), "duplicate recurrent kernel");
            mode = Some(match pair[1].as_str() {
                "reference" => false,
                "fused" => true,
                _ => anyhow::bail!("recurrent kernel must be reference|fused"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
fn timing_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(flags.len().is_multiple_of(2), "timing flags require pairs");
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--execution-timing" {
            ensure!(mode.is_none(), "duplicate execution timing");
            mode = Some(match pair[1].as_str() {
                "off" => false,
                "on" => true,
                _ => anyhow::bail!("execution timing must be off|on"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
fn graph_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(flags.len().is_multiple_of(2), "graph flags require pairs");
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--graph-mode" {
            ensure!(mode.is_none(), "duplicate graph mode");
            mode = Some(match pair[1].as_str() {
                "eager" => false,
                "compiled-decode" => true,
                _ => anyhow::bail!("graph mode must be eager|compiled-decode"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
fn operator_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(
        flags.len().is_multiple_of(2),
        "operator flags require pairs"
    );
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--operator-profile" {
            ensure!(mode.is_none(), "duplicate operator profile");
            mode = Some(match pair[1].as_str() {
                "off" => false,
                "stages" => true,
                _ => anyhow::bail!("operator profile must be off|stages"),
            });
        } else {
            rest.extend_from_slice(pair);
        }
    }
    Ok((mode.unwrap_or(false), rest))
}
fn attention_flags(flags: &[String]) -> Result<(bool, Vec<String>)> {
    ensure!(
        flags.len().is_multiple_of(2),
        "attention flags require pairs"
    );
    let mut mode = None;
    let mut rest = Vec::new();
    for pair in flags.as_chunks::<2>().0 {
        if pair[0] == "--attention-kernel" {
            ensure!(mode.is_none(), "duplicate attention kernel");
            mode = Some(match pair[1].as_str() {
                "reference" => false,
                "fused" => true,
                _ => anyhow::bail!("attention kernel must be reference|fused"),
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
        native_qwen::{
            ExecutionTuning, QwenDeviceExecutor, ScheduledDevice, ScheduledMtp, StateMode,
        },
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
    let (fused_attention, flags) = attention_flags(flags)?;
    let (operator_profile, flags) = operator_flags(&flags)?;
    let (compiled_decode, flags) = graph_flags(&flags)?;
    let (execution_timing, flags) = timing_flags(&flags)?;
    let (fused_recurrent, flags) = recurrent_flags(&flags)?;
    let (layer_group, flags) = layer_group_flags(&flags)?;
    let (physical, flags) = batching_flags(&flags)?;
    let (deferred_seed, flags) = seed_flags(&flags)?;
    let (execution, flags) = profile_flags(&flags)?;
    ensure!(
        !(deferred_seed && execution == Profile::Target),
        "deferred seed requires an MTP execution profile"
    );
    ensure!(
        !physical || execution == Profile::Target,
        "physical batching requires target profile"
    );
    ensure!(
        !compiled_decode || std::env::var_os("MLX_DISABLE_COMPILE").is_none(),
        "compiled decode refuses MLX_DISABLE_COMPILE"
    );
    ensure!(
        !operator_profile
            || (execution == Profile::Target && !physical && !compiled_decode && !execution_timing),
        "operator profile requires target/scalar/eager/timing off"
    );
    let options = Options::parse(&flags)?;
    let controller = options.controller()?;
    let manifest = Manifest::read(std::path::Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let state_budget = if physical { 2 } else { 1 } * 1024 * 1024 * 1024;
    let executor = Arc::new(if execution == Profile::Target {
        QwenDeviceExecutor::open_configured(
            Device::Gpu,
            64,
            state_budget,
            ExecutionTuning {
                layer_group,
                fused_recurrent,
                execution_timing,
                compiled_decode,
                operator_profile,
                fused_attention,
                validation_mode: 2,
                activation_precision: 1,
            },
        )
        .await?
    } else {
        QwenDeviceExecutor::open_mtp_configured(
            Device::Gpu,
            64,
            state_budget,
            ExecutionTuning {
                layer_group,
                fused_recurrent,
                execution_timing,
                compiled_decode,
                operator_profile: false,
                fused_attention,
                validation_mode: 2,
                activation_precision: 1,
            },
        )
        .await?
    });
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 511,
        max_new_tokens: 128,
        model_memory_bytes: state_budget as usize,
        max_in_flight: 3,
    };
    let (engine, proposer, seed_metrics, batch_counters) = if execution == Profile::Target {
        let backend = ScheduledDevice::new(executor.clone(), tokenizer.eos(), StateMode::Device);
        let backend = if physical {
            backend.with_physical_batching()?
        } else {
            backend
        };
        let counters = backend.batch_counters();
        (
            Engine::new(config, backend.storage(), backend)?,
            None,
            None,
            Some(counters),
        )
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
            None,
        )
    };
    let profile=format!("EXPERIMENTAL-NATIVE-HTTP execution={execution:?} seed_mode={} width={} stream_capacity={} policy={:?} context=511 in_flight=3 body_timeout_ms={body_timeout_ms} max_connections={max_connections} physical_tensor_batching={physical} layer_group={layer_group} recurrent_kernel={} execution_timing={} graph_mode={} operator_profile={} attention_kernel={} state_budget_bytes={state_budget} production_qualified=false audit_clock={}",if deferred_seed {"deferred"} else {"eager"},options.width,options.capacity,options.policy,if fused_recurrent {"fused"} else {"reference"},if execution_timing {"on"} else {"off"},if compiled_decode {"compiled-decode"} else {"eager"},if operator_profile {"stages"} else {"off"},if fused_attention {"fused"} else {"reference"},if execution==Profile::Audited {"validated-batches"} else {"disabled"});
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
    println!("EXECUTION_METRICS {:?}", native.execution);
    println!("OPERATOR_METRICS {:?}", native.operators);
    println!(
        "COMPILED_CACHE entries={} misses={} calls={}",
        native.compiled_entries, native.compiled_misses, native.compiled_calls
    );
    println!(
        "NATIVE_MEMORY active_bytes={} peak_bytes={} layer_group={layer_group}",
        native.mlx_active_bytes, native.mlx_peak_bytes
    );
    ensure!(
        transport.active_producers == 0
            && transport.active_connections == 0
            && scheduler.memory_used == 0
            && native.logical_bytes == 0,
        "HTTP shutdown drain"
    );
    if let Some(counters) = batch_counters {
        println!(
            "BATCH_METRICS {:?}",
            *counters
                .lock()
                .map_err(|_| anyhow::anyhow!("batch counters poisoned"))?
        );
    }
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
    fn diagnostic_flags_are_explicit_and_strict() {
        assert!(!operator_flags(&[]).unwrap().0);
        assert!(!attention_flags(&[]).unwrap().0);
        assert!(
            operator_flags(&args("--operator-profile stages"))
                .unwrap()
                .0
        );
        assert!(
            attention_flags(&args("--attention-kernel fused"))
                .unwrap()
                .0
        );
        for s in [
            "--operator-profile",
            "--operator-profile auto",
            "--operator-profile off --operator-profile stages",
        ] {
            assert!(operator_flags(&args(s)).is_err());
        }
        for s in [
            "--attention-kernel",
            "--attention-kernel auto",
            "--attention-kernel reference --attention-kernel fused",
        ] {
            assert!(attention_flags(&args(s)).is_err());
        }
        assert_eq!(
            operator_flags(&args("--operator-profile off --layer-group 4"))
                .unwrap()
                .1,
            args("--layer-group 4")
        );
    }
    #[test]
    fn compiled_mode_is_explicit() {
        assert!(!graph_flags(&[]).unwrap().0);
        assert!(
            graph_flags(&args("--graph-mode compiled-decode"))
                .unwrap()
                .0
        );
        for s in [
            "--graph-mode",
            "--graph-mode auto",
            "--graph-mode compiled-decode --graph-mode eager",
        ] {
            assert!(graph_flags(&args(s)).is_err());
        }
    }
    #[test]
    fn timing_is_explicit_and_fail_closed() {
        assert!(!timing_flags(&[]).unwrap().0);
        assert!(timing_flags(&args("--execution-timing on")).unwrap().0);
        for s in [
            "--execution-timing",
            "--execution-timing yes",
            "--execution-timing on --execution-timing off",
        ] {
            assert!(timing_flags(&args(s)).is_err());
        }
    }
    #[test]
    fn recurrent_kernel_is_explicit() {
        assert!(!recurrent_flags(&[]).unwrap().0);
        assert!(
            recurrent_flags(&args("--recurrent-kernel fused"))
                .unwrap()
                .0
        );
        for s in [
            "--recurrent-kernel",
            "--recurrent-kernel bad",
            "--recurrent-kernel fused --recurrent-kernel reference",
        ] {
            assert!(recurrent_flags(&args(s)).is_err());
        }
        assert_eq!(
            recurrent_flags(&args("--recurrent-kernel reference --layer-group 4"))
                .unwrap()
                .1,
            args("--layer-group 4")
        );
    }
    #[tokio::test]
    async fn invalid_group_constructor_rejects_before_model_loading() {
        for group in [0, 2, 3, 41, u32::MAX] {
            let result = super::super::native_qwen::QwenDeviceExecutor::open_grouped(
                super::super::native_mlx::Device::Gpu,
                8,
                1024,
                group,
            )
            .await;
            assert!(result.is_err());
        }
    }
    #[test]
    fn layer_groups_fail_closed() {
        assert_eq!(layer_group_flags(&[]).unwrap().0, 1);
        for group in [1, 4, 10, 40] {
            assert_eq!(
                layer_group_flags(&args(&format!("--layer-group {group}")))
                    .unwrap()
                    .0,
                group
            );
        }
        for value in [
            "--layer-group",
            "--layer-group 0",
            "--layer-group 3",
            "--layer-group 41",
            "--layer-group -1",
            "--layer-group 04",
            "--layer-group 4 --layer-group 10",
        ] {
            assert!(layer_group_flags(&args(value)).is_err());
        }
        assert_eq!(
            layer_group_flags(&args("--layer-group 4 --execution-profile target"))
                .unwrap()
                .1,
            args("--execution-profile target")
        );
    }
    #[test]
    fn batching_mode_fails_closed_and_defaults_scalar() {
        assert!(!batching_flags(&[]).unwrap().0);
        assert!(
            batching_flags(&args("--tensor-batching physical"))
                .unwrap()
                .0
        );
        for s in [
            "--tensor-batching",
            "--tensor-batching auto",
            "--tensor-batching scalar --tensor-batching physical",
        ] {
            assert!(batching_flags(&args(s)).is_err());
        }
        let (physical, rest) =
            batching_flags(&args("--tensor-batching scalar --execution-profile target")).unwrap();
        assert!(!physical);
        assert_eq!(rest, args("--execution-profile target"));
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
