use anyhow::{bail, ensure, Context, Result};
use rust_server::{
    execute,
    gpu::Gpu,
    ir,
    runtime::{ChainJob, Job, Runtime},
    wire,
};
use std::{path::Path, time::Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        #[cfg(feature = "native-mlx")]
        Some("native-mtp-inspect") => { ensure!(args.len()==2,"native-mtp-inspect takes no arguments");rust_server::serving::native_mtp::inspect_selected().await?; }
        #[cfg(feature = "native-mlx")]
        Some("native-device-smoke") => {ensure!(args.len()==3,"native-device-smoke NEW_DIRECTORY");rust_server::serving::native_device_smoke::run(Path::new(&args[2])).await?;}
        #[cfg(feature = "native-mlx")]
        Some("native-layer-reference") => {ensure!(args.len()==3,"native-layer-reference NEW_SCRATCH_DIRECTORY");rust_server::serving::native_layer_reference::run(Path::new(&args[2])).await?;}
        #[cfg(feature = "native-mlx")]
        Some("native-ingestion") => { ensure!(args.len()==2,"native-ingestion takes no arguments");rust_server::serving::native_ingestion::run().await?; }
        #[cfg(feature = "native-mlx")]
        Some("native-reference") => {
            let mut device=rust_server::serving::native_mlx::Device::Gpu;let mut full=false;let mut index=2;let mut seen_device=false;
            while index<args.len(){match args[index].as_str(){
                "--full"=>{ensure!(!full,"duplicate --full");full=true;index+=1;},
                "--device"=>{ensure!(!seen_device&&index+1<args.len(),"native-reference [--device cpu|gpu] [--full]");seen_device=true;device=match args[index+1].as_str(){"cpu"=>rust_server::serving::native_mlx::Device::Cpu,"gpu"=>rust_server::serving::native_mlx::Device::Gpu,_=>bail!("native-reference [--device cpu|gpu] [--full]")};index+=2;},
                _=>bail!("native-reference [--device cpu|gpu] [--full]")
            }}
            rust_server::serving::native_reference::run(device,full).await?;
        }
        #[cfg(feature = "native-mlx")]
        Some("native-diagnose") => {ensure!(args.len()==2,"native-diagnose takes no arguments");rust_server::serving::native_diagnose::run().await?;}
        #[cfg(feature = "native-mlx")]
        Some("native-qualify") => {
            ensure!(args.len()==2 || (args.len()==3 && ["--contracts","--extended"].contains(&args[2].as_str())),"native-qualify [--contracts|--extended]");
            if args.get(2).is_some_and(|s|s=="--extended") {rust_server::serving::native_extended::run().await?;} else {rust_server::serving::native_boundary::qualify_cli(args.len()==3).await?;}
        }
        #[cfg(feature = "native-mlx")]
        Some("native-mtp-serve") => {
            ensure!(args.len()>=5,"native-mtp-serve LOOPBACK:PORT BODY_TIMEOUT_MS MAX_CONNECTIONS --execution-profile target|mtp|mtp-audited [--tensor-batching scalar|physical (target only)] [--layer-group 1|4|10|40] [--recurrent-kernel reference|fused] [--attention-kernel reference|fused] [--expert-kernel reference|sorted|fused] [--sampling host|device (target scalar only)] [--execution-timing off|on] [--graph-mode eager|compiled-decode] [--operator-profile off|stages (target scalar eager only)] [seven explicit policy flags]");
            rust_server::serving::native_http::serve(args[2].parse()?,args[3].parse()?,args[4].parse()?,&args[5..]).await?;
        }
        #[cfg(feature = "native-mlx")]
        Some("native-mtp-generate" | "native-mtp-stream") => {
            ensure!(args.len()>=4,"{}",rust_server::serving::native_mtp_cli::USAGE);
            if args[1]=="native-mtp-stream" {rust_server::serving::native_mtp_cli::run_wire(&args[2],args[3].parse()?,&args[4..]).await?;}else{rust_server::serving::native_mtp_cli::run(&args[2],args[3].parse()?,&args[4..]).await?;}
        }
        #[cfg(feature = "native-mlx")]
        Some("native-generate") => {
            use rust_server::serving::{native_qwen::ActivationPrecision,native_mlx::Device};
            ensure!(args.len()>=4 && (args.len()-4)%2==0,"native-generate PROMPT MAX_NEW [--activation f32|bf16] [--device gpu|cpu] [--choices yes,no]; selected Combined model, regular KV only");
            let mut choices=None;let mut precision=ActivationPrecision::F32;let mut device=Device::Gpu;let mut seen=std::collections::BTreeSet::new();
            for pair in args[4..].as_chunks::<2>().0 {ensure!(seen.insert(pair[0].as_str()),"duplicate generation flag");match pair[0].as_str(){
                "--choices"=> {ensure!(pair[1].len()<=4096,"choice text cap");choices=Some(pair[1].split(',').map(str::to_owned).collect());},
                "--activation"=>precision=match pair[1].as_str(){"f32"=>ActivationPrecision::F32,"bf16"=>ActivationPrecision::Bf16,_=>bail!("activation must be f32 or bf16")},
                "--device"=>device=match pair[1].as_str(){"gpu"=>Device::Gpu,"cpu"=>Device::Cpu,_=>bail!("device must be gpu or cpu")},
                _=>bail!("unsupported generation flag")
            }}
            rust_server::serving::native_boundary::generate_cli(&args[2],args[3].parse()?,precision,device,choices).await?;
        }
        #[cfg(feature = "native-mlx")]
        Some("native-mlx") => {
            match args.get(2).map(String::as_str) {
                None => rust_server::serving::native_mlx::smoke().await?,
                Some("weights") if args.len()==3 => rust_server::serving::native_mlx::checkpoint_smoke().await?,
                Some("bench") if args.len()==3 => rust_server::serving::native_mlx::benchmark().await?,
                _ => bail!("native-mlx [bench|weights]; no JSON/configuration arguments"),
            }
        }
        #[cfg(not(feature = "legacy-python"))]
        Some("model-run" | "config") => bail!("Python model adapter disabled; use native-mlx. Historical adapter requires explicit legacy-python feature"),
        #[cfg(feature = "legacy-python")]
        Some("model-run") => {
            ensure!(args.len()>=5,"model-run CONFIG.json PROMPTS.json MAX_NEW [--kv-codec MODE] [--max-in-flight 2|3] [--prefill-chunk-tokens N] [--max-batch-tokens N] [--probe-directory PATH]");
            let mut config: rust_server::serving::rapid::RapidConfig =
                serde_json::from_slice(&read_bounded(&args[2], 65536)?)?;
            let prompts: Vec<String> =
                serde_json::from_slice(&read_bounded(&args[3], 512 * 1024)?)?;
            let max_new: usize = args[4].parse()?;
            let mut probe = None;
            let mut seen = std::collections::BTreeSet::new();
            ensure!(
                (args.len() - 5) % 2 == 0,
                "every model-run flag requires a value"
            );
            for pair in args[5..].as_chunks::<2>().0 {
                ensure!(seen.insert(pair[0].as_str()), "duplicate model-run flag");
                match pair[0].as_str() {
                    "--kv-codec" => config.codec = pair[1].clone(),
                    "--max-in-flight" => config.scheduler.max_in_flight = pair[1].parse()?,
                    "--prefill-chunk-tokens" => {
                        config.scheduler.prefill_chunk_tokens = pair[1].parse()?
                    }
                    "--max-batch-tokens" => config.scheduler.max_batch_tokens = pair[1].parse()?,
                    "--probe-directory" => probe = Some(std::path::PathBuf::from(&pair[1])),
                    _ => bail!("unknown model-run flag: {}", pair[0]),
                }
            }
            config.validate()?;
            ensure!(
                (1..=config.scheduler.max_new_tokens).contains(&max_new),
                "generation limit"
            );
            let worker = Path::new(env!("CARGO_MANIFEST_DIR")).join("backend/rapid_worker.py");
            let (backend, storage, tokens, ready, report) =
                rust_server::serving::rapid::RapidBackend::start(
                    &config,
                    &worker,
                    &prompts,
                    probe.as_deref(),
                )
                .await?;
            let engine = rust_server::serving::scheduler::Engine::new(
                config.scheduler.clone(),
                storage,
                backend,
            )?;
            let begin = Instant::now();
            let mut requests = Vec::new();
            for prompt in tokens {
                requests.push(engine.submit(prompt, max_new)?);
            }
            let mut generations = Vec::new();
            for request in requests {
                generations.push(request.result().await?);
            }
            let elapsed = begin.elapsed().as_secs_f64();
            let stats = engine.shutdown().await?;
            let token_lists: Vec<_> = generations.iter().map(|g| g.tokens.clone()).collect();
            let details = report.finish(&token_lists).await?;
            ensure!(
                details["active_sessions"] == 0 && stats.memory_used == 0,
                "model state did not drain"
            );
            use sha2::{Digest, Sha256};
            let config_hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&config)?));
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"pass":true,"qualification":"EXPERIMENTAL-MLX-AB","production_sampling_claims":false,"resolved_config":config,"resolved_config_sha256":config_hash,"readiness":ready,"generations":generations,"stats":stats,"rust_boundary_seconds":elapsed,"diagnostic_probe_io_included":probe.is_some(),"memory_accounting":"stats.memory_* are reserved KV/state/workspace bytes; measured MLX allocations including weights are in details; this is not a unified OS memory broker","details":details})
                )?
            );
        }
        #[cfg(feature = "legacy-python")]
        Some("config") => {
            ensure!(
                args.len() == 5
                    && args[3] == "--config"
                    && ["check", "print"].contains(&args[2].as_str()),
                "config check|print --config FILE.json"
            );
            let config: rust_server::serving::rapid::RapidConfig =
                serde_json::from_slice(&read_bounded(&args[4], 65536)?)?;
            config.validate()?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"valid":true,"resolved_config":config,"production_qualified":false})
                )?
            );
        }
        Some("cache-judge") => {
            ensure!(
                args.len() == 3,
                "cache-judge CONFIG.json (explicit experimental configuration)"
            );
            cache_judge(&args[2]).await?;
        }
        Some("capabilities") => {
            use rust_server::backend::{Config, KernelBackend, MetalBackend};
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &MetalBackend::new(Config::default())?.capabilities()
                )?
            );
        }
        Some("backend-judge") => {
            ensure!(args.len() == 3, "backend-judge DIRECTORY");
            backend_judge(&args[2])?;
        }
        Some("run-chain") => {
            ensure!(
                (4..=19).contains(&args.len()),
                "run-chain INPUTS.json TRACE [TRACE ...] (1..16 stages)"
            );
            let inputs = serde_json::from_slice(&read_bounded(&args[2], 32 * 1024 * 1024)?)?;
            let traces = args[3..]
                .iter()
                .map(|p| read_bounded(p, 65536))
                .collect::<Result<Vec<_>>>()?;
            let runtime = configured_runtime(8)?;
            let receipt = runtime.submit_chain(ChainJob { traces, inputs })?.await?;
            let success = receipt.failure.is_none();
            println!("{}", serde_json::to_string(&receipt)?);
            ensure!(success, "chain failed; see receipt");
            runtime.flush_validation().await?;
        }
        Some("run") => {
            ensure!(args.len() == 4, "usage: rust-server run TRACE INPUTS.json");
            let trace = read_bounded(&args[2], 65536)?;
            let inputs = serde_json::from_slice(&read_bounded(&args[3], 32 * 1024 * 1024)?)?;
            let runtime = configured_runtime(8)?;
            let receipt = runtime.submit(Job { trace, inputs })?.await?;
            let success = receipt.failure.is_none();
            println!("{}", serde_json::to_string(&receipt)?);
            ensure!(success, "job failed; see receipt");
        }
        Some("macro-bench") => {
            ensure!(args.len() == 6, "macro-bench DIRECTORY THREADS LIMIT COUNT");
            rust_server::macro_bench::run(
                &args[2],
                args[3].parse()?,
                args[4].parse()?,
                args[5].parse()?,
            )
            .await?;
        }
        Some("stream-check") => {
            ensure!(args.len()==3,"stream-check FILE");
            let bytes=read_bounded(&args[2],130*(72*1024+4))?;
            let mut decoder=rust_server::serving::stream_wire::Decoder::default();
            let mut frames=Vec::new();for chunk in bytes.chunks(7){frames.extend(decoder.push(chunk)?);}decoder.finish()?;
            let tokens=frames.iter().filter(|f|f.kind==1).count();
            println!("STREAM frames={} tokens={} terminal={}",frames.len(),tokens,frames.last().unwrap().kind);
            let text:String=frames.iter().filter(|f|f.kind==1||f.kind==2).map(|f|f.text.as_str()).collect();println!("TEXT {}",text.escape_debug());
        }
        Some("runtime-judge") => runtime_judge().await?,
        Some("kernel-judge") => {
            ensure!(
                (3..=4).contains(&args.len()),
                "kernel-judge DIRECTORY [ABS_CAPTURE_PATH]"
            );
            let dir = &args[2];
            let bytes = read_bounded(&format!("{dir}/trace.amvk"), 65536)?;
            let plan = ir::lower_trace(&bytes)?;
            let inputs = execute::seeded_inputs(&plan, 42);
            let capture = args.get(3).map(Path::new);
            let (output, samples, msl) = execute::execute_capture(&plan, &inputs, 53, capture)?;
            std::fs::write(format!("{dir}/inputs.json"), serde_json::to_vec(&inputs)?)?;
            std::fs::write(
                format!("{dir}/plan.json"),
                serde_json::to_vec_pretty(&plan)?,
            )?;
            std::fs::write(format!("{dir}/kernel.metal"), msl)?;
            std::fs::write(format!("{dir}/result.json"), serde_json::to_vec(&output)?)?;
            println!(
                "{}",
                serde_json::json!({"pass":true,"shape":[plan.m,plan.k,plan.n],"metrics":output.metrics,"samples_ms":samples})
            );
        }
        Some("uma-judge") => {
            ensure!(args.len() == 2, "uma-judge takes no arguments");
            println!("{}", Gpu::new()?.identity_judge()?);
        }
        Some("idle") => {
            ensure!(args.len() == 2, "idle takes no arguments");
            println!("ready");
            std::future::pending::<()>().await;
        }
        Some("decode") => {
            ensure!(args.len() == 3, "decode TRACE");
            let bytes = read_bounded(&args[2], 65536)?;
            println!("{}", serde_json::to_string(&wire::decode(&bytes)?)?);
        }
        Some("--help") | Some("help") => {
            println!("rust-server: native-mtp-serve LOOPBACK:PORT BODY_TIMEOUT_MS MAX_CONNECTIONS --execution-profile target|mtp|mtp-audited [--tensor-batching scalar|physical (target only)] [--layer-group 1|4|10|40] [--recurrent-kernel reference|fused] [--attention-kernel reference|fused] [--expert-kernel reference|sorted|fused] [--sampling host|device (target scalar only)] [--execution-timing off|on] [--graph-mode eager|compiled-decode] [--operator-profile off|stages (target scalar eager only)] [explicit policy flags] | stream-check FILE | native-mtp-stream PROMPT MAX_NEW [explicit policy flags] | native-mtp-generate PROMPT MAX_NEW [explicit policy flags; see README] | model-run CONFIG PROMPTS MAX_NEW [flags] | config check|print --config FILE | cache-judge CONFIG.json | run-chain INPUTS.json TRACE [TRACE ...] | macro-bench DIRECTORY THREADS LIMIT COUNT | capabilities | backend-judge DIRECTORY | idle | decode TRACE | uma-judge | kernel-judge DIRECTORY [ABS_CAPTURE_PATH] | runtime-judge | run TRACE INPUTS.json; runtime limit: RUST_SERVER_IN_FLIGHT=2|3 (default 3)");
        }
        _ => bail!("use rust-server --help"),
    }
    Ok(())
}

async fn runtime_judge() -> Result<()> {
    let bytes = read_bounded("artifacts/trace.amvk", 65536)?;
    let plan = ir::lower_trace(&bytes)?;
    let runtime = configured_runtime(32)?;
    let mut pending = Vec::new();
    for seed in 0..24 {
        let inputs = execute::seeded_inputs(&plan, seed);
        let before = Instant::now();
        let mut rx = runtime.submit(Job {
            trace: bytes.clone(),
            inputs,
        })?;
        let returned_before_completion = matches!(
            rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        );
        pending.push((
            rx,
            before.elapsed().as_secs_f64() * 1000.,
            returned_before_completion,
        ));
    }
    for (rx, submit_ms, returned_before_completion) in pending {
        let receipt = rx.await?;
        ensure!(receipt.failure.is_none(), "runtime job failed");
        ensure!(
            returned_before_completion,
            "job completed before submission returned"
        );
        println!(
            "{}",
            serde_json::json!({
                "submit_ms":submit_ms,"returned_before_completion":returned_before_completion,
                "process_id":std::process::id(),"kernel":receipt.kernel,"job_id":receipt.job_id,"shape":receipt.shape,
                "wall_ms":receipt.wall_ms,"queue_ms":receipt.queue_ms,"signpost_id":receipt.signpost_id,
                "signpost_enabled":receipt.signpost_enabled,"metrics":receipt.output.as_ref().map(|x| &x.metrics),
                "worker_id":receipt.worker_id,"flight":receipt.flight,"failure":receipt.failure
            })
        );
    }
    let flight = runtime.flight_stats();
    ensure!(
        flight.submitted == 24
            && flight.completed == 24
            && flight.active == 0
            && flight.peak <= flight.limit
            && flight.peak_outstanding <= flight.limit,
        "flight accounting mismatch"
    );
    std::fs::write(
        "artifacts/runtime-flight.json",
        serde_json::to_vec_pretty(&flight)?,
    )?;
    Ok(())
}

fn read_bounded(path: &str, max: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .with_context(|| format!("open {path}"))?
        .take(max + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= max, "file size budget");
    Ok(bytes)
}

fn backend_judge(dir: &str) -> Result<()> {
    use rust_server::backend::{Config, KernelBackend, MetalBackend};
    let plan = ir::lower_trace(&read_bounded(&format!("{dir}/trace.amvk"), 65536)?)?;
    let inputs = execute::seeded_inputs(&plan, 42);
    let start = Instant::now();
    let mut backend = MetalBackend::new(Config::default())?;
    let construction_ms = start.elapsed().as_secs_f64() * 1000.;
    let capabilities = backend.capabilities();
    let mut retained = Vec::new();
    let mut recreated = Vec::new();
    for _ in 0..53 {
        let start = Instant::now();
        let (out, _, _) = backend.execute(&plan, &inputs, 1, None)?;
        retained.push(serde_json::json!({"caller_wall_ms":start.elapsed().as_secs_f64()*1000.,"metrics":out.metrics}));
    }
    ensure!(
        backend.stats().pipeline_hits == 52,
        "persistent pipeline reuse"
    );
    ensure!(
        backend.stats().buffer_allocations == plan.input_count() as u64 + 1,
        "persistent allocation reuse"
    );
    drop(backend); // No retained pool overlaps the recreated-backend measurement phase.
    for _ in 0..53 {
        let start = Instant::now();
        let mut fresh = MetalBackend::new(Config::default())?;
        let (out, _, _) = fresh.execute(&plan, &inputs, 1, None)?;
        recreated.push(serde_json::json!({"caller_wall_ms":start.elapsed().as_secs_f64()*1000.,"metrics":out.metrics}));
    }
    println!(
        "{}",
        serde_json::json!({"pass":true,"kernel":plan.kind,"shape_mkn":[plan.m,plan.k,plan.n],"backend_construction_ms":construction_ms,"capabilities":capabilities,"persistent":retained,"recreated":recreated,"scope":"same raw Q1.31 integer inputs; full CPU reference enabled only with debug-validation feature; caller wall includes fresh backend construction in recreated case; persistent backend dropped before recreated phase; OS shader cache not flushed; first 3 samples are warmup; no serving-throughput claim"})
    );
    Ok(())
}

fn configured_runtime(capacity: usize) -> Result<Runtime> {
    let limit = match std::env::var("RUST_SERVER_IN_FLIGHT") {
        Ok(value) => value
            .parse::<usize>()
            .context("RUST_SERVER_IN_FLIGHT must be 2 or 3")?,
        Err(std::env::VarError::NotPresent) => 3,
        Err(error) => bail!("invalid RUST_SERVER_IN_FLIGHT: {error}"),
    };
    Runtime::with_in_flight(capacity, limit)
}

async fn cache_judge(path: &str) -> Result<()> {
    use rust_server::{
        cache::{Admission, Config, Controller, Identity, Pressure},
        sampled::{Selection, ValidationConfig},
    };
    let config: Config = serde_json::from_slice(&read_bounded(path, 65536)?)?;
    ensure!(
        config.capacity_blocks >= 2,
        "cache judge needs at least two blocks"
    );
    let bytes_per_block = config.bytes_per_block;
    let cache = Controller::new(config.clone())?;
    let identity = Identity {
        model: "SYNTHETIC-NONMODEL-v1".into(),
        tokenizer: "test-token-ids-v1".into(),
        quantization: "opaque-fixture-bytes".into(),
        namespace: "private-cache-judge".into(),
    };
    let root = cache.branch(None, identity.clone(), (0..16).collect())?;
    let first = match cache.request_prefill(root)? {
        Admission::Ready(id) => id,
        _ => bail!("empty cache admission unexpectedly queued"),
    };
    cache.fulfill(first, |request| {
        Ok(request
            .missing
            .iter()
            .map(|d| vec![d.key[0]; bytes_per_block])
            .collect())
    })?;
    let child = cache.branch(Some(root), identity, (16..32).collect())?;
    let second = match cache.request_prefill(child)? {
        Admission::Ready(id) => id,
        _ => bail!("child admission unexpectedly queued"),
    };
    cache.fulfill(second, |request| {
        ensure!(request.resident_prefix.len() == 1, "shared prefix missing");
        Ok(request
            .missing
            .iter()
            .map(|d| vec![d.key[0]; bytes_per_block])
            .collect())
    })?;
    let audit = cache.audit(1, &[child], 123, Pressure::Normal)?;
    ensure!(
        audit.declared_tokens == 32 && audit.resident_declared_tokens == 32,
        "exact coverage failed"
    );
    let runtime = Runtime::with_sampling(
        8,
        3,
        None,
        ValidationConfig {
            selection: Selection::DeterministicBench,
            every: 1,
        },
    )?;
    let trace = include_bytes!("../fixtures/elementwise.amvk").to_vec();
    let plan = ir::lower_trace(&trace)?;
    let receipt = runtime
        .submit_cached(
            ChainJob {
                traces: vec![trace],
                inputs: execute::seeded_inputs(&plan, 42),
            },
            cache.lease(child)?,
        )?
        .await?;
    ensure!(receipt.failure.is_none(), "cached runtime request failed");
    let validation = runtime.flush_validation().await?;
    ensure!(
        cache.stats()?.active_leases == 0 && validation.passed == 1,
        "completion or sample accounting"
    );
    cache.close_subtree(root)?;
    let released = cache.audit(2, &[], 0, Pressure::Critical)?;
    println!(
        "{}",
        serde_json::json!({"pass":true,"payload_source":"SYNTHETIC-NONMODEL","policy":"DETERMINISTIC-MODE","production_qualified":false,"configuration":config,"audit":audit,"released":released,"cache":cache.stats()?,"validation":validation,"receipt":receipt})
    );
    Ok(())
}
