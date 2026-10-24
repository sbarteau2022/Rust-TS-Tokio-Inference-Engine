use super::*;
use std::{fs, path::PathBuf};
struct Fixture {
    path: PathBuf,
    manifest: Arc<Manifest>,
    c: Architecture,
}
impl Fixture {
    fn new() -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "qwen-hybrid-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        let c = Architecture {
            hidden: 32,
            layers: 4,
            vocab: 64,
            q_heads: 2,
            kv_heads: 1,
            head_dim: 16,
            key_heads: 1,
            value_heads: 2,
            key_dim: 16,
            value_dim: 16,
            conv: 4,
            interval: 4,
            experts: 4,
            top: 2,
            intermediate: 32,
            shared: 32,
            group: 32,
            bits: 4,
            context: 512,
            chunk: 8,
            rope_dims: 4,
            activation_precision: 1,
            layer_group: 1,
            fused_recurrent: 0,
            execution_timing: 0,
            compiled_decode: 0,
            operator_profile: 0,
            fused_attention: 0,
            validation_mode: 2,
            validation_interval: 32,
            epsilon: 1e-6,
            theta: 10000.,
        };
        let mut tensors: BTreeMap<String, (Vec<i32>, &str, Vec<u8>)> = BTreeMap::new();
        fn f32_tensor(
            t: &mut BTreeMap<String, (Vec<i32>, &'static str, Vec<u8>)>,
            name: String,
            shape: Vec<i32>,
            value: f32,
        ) {
            let n: usize = shape.iter().map(|v| *v as usize).product();
            t.insert(
                name,
                (
                    shape,
                    "F32",
                    (0..n).flat_map(|_| value.to_le_bytes()).collect(),
                ),
            );
        }
        fn linear(
            t: &mut BTreeMap<String, (Vec<i32>, &'static str, Vec<u8>)>,
            p: &str,
            n: i32,
            k: i32,
            experts: Option<i32>,
        ) {
            let e = experts.unwrap_or(1);
            let mut packed = vec![0u32; (e * n * k / 8) as usize];
            for ex in 0..e {
                for row in 0..n {
                    for j in 0..k {
                        packed[((ex * n + row) * k / 8 + j / 8) as usize] |=
                            (((j * 7 + row * 3 + ex * 5) % 16) as u32) << ((j % 8) * 4);
                    }
                }
            }
            let shape = if experts.is_some() {
                vec![e, n, k / 8]
            } else {
                vec![n, k / 8]
            };
            t.insert(
                format!("{p}.weight"),
                (
                    shape,
                    "U32",
                    packed.iter().flat_map(|n| n.to_le_bytes()).collect(),
                ),
            );
            let shape = if experts.is_some() {
                vec![e, n, k / 32]
            } else {
                vec![n, k / 32]
            };
            f32_tensor(t, format!("{p}.scales"), shape.clone(), 0.01);
            f32_tensor(t, format!("{p}.biases"), shape, -0.075);
        }
        linear(
            &mut tensors,
            "language_model.model.embed_tokens",
            c.vocab,
            c.hidden,
            None,
        );
        // Real Combined embedding affine parameters are BF16. Their arithmetic
        // must widen before dequantization, not merely cast the result to F32.
        for suffix in ["scales", "biases"] {
            let tensor = tensors
                .get_mut(&format!("language_model.model.embed_tokens.{suffix}"))
                .expect("fixture embedding affine parameters");
            tensor.1 = "BF16";
            tensor.2 = tensor
                .2
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|value| {
                    let bits = u32::from_le_bytes(*value);
                    ((bits >> 16) as u16).to_le_bytes()
                })
                .collect();
        }
        linear(
            &mut tensors,
            "language_model.lm_head",
            c.vocab,
            c.hidden,
            None,
        );
        f32_tensor(
            &mut tensors,
            "language_model.model.norm.weight".into(),
            vec![32],
            1.,
        );
        for layer in 0..4 {
            let p = format!("language_model.model.layers.{layer}");
            for norm in ["input_layernorm", "post_attention_layernorm"] {
                f32_tensor(&mut tensors, format!("{p}.{norm}.weight"), vec![32], 1.);
            }
            if layer == 3 {
                for (name, n, k) in [
                    ("q_proj", 64, 32),
                    ("k_proj", 16, 32),
                    ("v_proj", 16, 32),
                    ("o_proj", 32, 32),
                ] {
                    linear(&mut tensors, &format!("{p}.self_attn.{name}"), n, k, None);
                }
                for norm in ["q_norm", "k_norm"] {
                    f32_tensor(
                        &mut tensors,
                        format!("{p}.self_attn.{norm}.weight"),
                        vec![16],
                        1.,
                    );
                }
            } else {
                for (name, n, k) in [
                    ("in_proj_qkv", 64, 32),
                    ("in_proj_z", 32, 32),
                    ("in_proj_a", 2, 32),
                    ("in_proj_b", 2, 32),
                    ("out_proj", 32, 32),
                ] {
                    linear(&mut tensors, &format!("{p}.linear_attn.{name}"), n, k, None);
                }
                f32_tensor(
                    &mut tensors,
                    format!("{p}.linear_attn.conv1d.weight"),
                    vec![64, 4, 1],
                    0.25,
                );
                f32_tensor(
                    &mut tensors,
                    format!("{p}.linear_attn.norm.weight"),
                    vec![16],
                    1.,
                );
                f32_tensor(&mut tensors, format!("{p}.linear_attn.A_log"), vec![2], -1.);
                f32_tensor(
                    &mut tensors,
                    format!("{p}.linear_attn.dt_bias"),
                    vec![2],
                    0.,
                );
            }
            linear(&mut tensors, &format!("{p}.mlp.gate"), 4, 32, None);
            linear(
                &mut tensors,
                &format!("{p}.mlp.shared_expert_gate"),
                1,
                32,
                None,
            );
            for name in ["gate_proj", "up_proj", "down_proj"] {
                linear(
                    &mut tensors,
                    &format!("{p}.mlp.shared_expert.{name}"),
                    32,
                    32,
                    None,
                );
                linear(
                    &mut tensors,
                    &format!("{p}.mlp.switch_mlp.{name}"),
                    32,
                    32,
                    Some(4),
                );
            }
        }
        let mut header = serde_json::Map::new();
        let mut mapping = serde_json::Map::new();
        let mut payload = Vec::new();
        for (name, (shape, dtype, data)) in tensors {
            let start = payload.len();
            payload.extend(data);
            header.insert(name.clone(),serde_json::json!({"shape":shape,"dtype":dtype,"data_offsets":[start,payload.len()]}));
            mapping.insert(name, serde_json::json!("model.safetensors"));
        }
        let header = serde_json::to_vec(&header)?;
        let mut file = (header.len() as u64).to_le_bytes().to_vec();
        file.extend(header);
        file.extend(payload);
        fs::write(path.join("model.safetensors"), file)?;
        fs::write(
            path.join("model.safetensors.index.json"),
            serde_json::to_vec(&serde_json::json!({"weight_map":mapping}))?,
        )?;
        fs::write(
            path.join("config.json"),
            br#"{"quantization":{"bits":4,"group_size":32}}"#,
        )?;
        let manifest = Arc::new(Manifest::read(&path)?);
        Ok(Self { path, manifest, c })
    }
    fn open(&self, device: Device) -> Result<QwenExecutor> {
        QwenExecutor::start(device, self.manifest.clone(), self.c)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
fn near(a: &[f32], b: &[f32]) {
    assert_eq!(a.len(), b.len());
    for (a, b) in a.iter().zip(b) {
        assert!((a - b).abs() < 3e-4 * (1. + b.abs()), "{a} vs {b}");
    }
}
#[tokio::test]
async fn complete_hybrid_cpu_gpu_and_chunked_state_equivalence() -> Result<()> {
    let fixture = Fixture::new()?;
    let cpu = fixture.open(Device::Cpu)?;
    let gpu = fixture.open(Device::Gpu)?;
    let tokens = vec![3, 5, 8, 11, 13, 17];
    let a = cpu.step(&cpu.empty_state(), tokens.clone()).await?;
    let b = gpu.step(&gpu.empty_state(), tokens.clone()).await?;
    assert!(b.trace.is_empty());
    let traced = gpu
        .execute(
            &gpu.empty_state(),
            ModelStepIr::compile(StepKind::Prefill, 0, tokens.clone(), 64)?
                .with_projection_checks()
                .with_operator_checks(),
        )
        .await?;
    assert_eq!(traced.trace.len(), 4 * 5 * tokens.len() * 32);
    assert!(traced.trace.iter().all(|v| v.is_finite()));
    near(&b.logits, &traced.logits);
    near(&b.state.values, &traced.state.values);
    let full_checked = gpu
        .execute(
            &gpu.empty_state(),
            ModelStepIr::compile(StepKind::Prefill, 0, tokens.clone(), 64)?.with_full_reference(),
        )
        .await?;
    near(&b.logits, &full_checked.logits);
    near(&b.state.values, &full_checked.state.values);

    near(&a.logits, &b.logits);
    near(&a.state.values, &b.state.values);
    let first = gpu.step(&gpu.empty_state(), tokens[..2].to_vec()).await?;
    let second = gpu.step(&first.state, tokens[2..].to_vec()).await?;
    near(&b.logits, &second.logits);
    near(&b.state.values, &second.state.values);
    assert_eq!(first.state.position(), 2);
    assert!(gpu.step(&a.state, vec![1]).await.is_err());
    assert!(gpu.step(&b.state, vec![64]).await.is_err());
    let next = gpu.step(&b.state, vec![19]).await?;
    let full = gpu
        .step(&gpu.empty_state(), vec![3, 5, 8, 11, 13, 17, 19])
        .await?;
    near(&next.logits, &full.logits);
    near(&next.state.values, &full.state.values);
    assert_eq!(cpu.shutdown().await?.leases, 0);
    assert_eq!(gpu.shutdown().await?.leases, 0);
    Ok(())
}
#[tokio::test]
async fn scheduler_rows_atomic_duplicate_stale_and_cancellation() -> Result<()> {
    let fixture = Fixture::new()?;
    let executor = fixture.open(Device::Gpu)?;
    let mut backend = ScheduledQwen::new(executor, vec![]);
    let input = |sequence, position, tokens| StepInput {
        remaining_output_tokens: 8,
        kind: StepKind::Prefill,
        sequence,
        position,
        tokens,
        prompt_complete: true,
        history: vec![],
        allowed_tokens: None,
    };
    let out = backend
        .step(vec![input(1, 0, vec![2, 3]), input(2, 0, vec![5, 7])])
        .await?;
    assert_eq!(out.len(), 2);
    let old = backend.states.get(&1).unwrap().clone();
    assert!(backend
        .step(vec![input(1, 2, vec![9]), input(1, 2, vec![9])])
        .await
        .is_err());
    assert!(backend.step(vec![input(1, 1, vec![9])]).await.is_err());
    // First candidate succeeds but the second is rejected before native admission: no partial commit.
    assert!(backend
        .step(vec![input(1, 2, vec![9]), input(2, 2, vec![999])])
        .await
        .is_err());
    assert_eq!(backend.states.get(&1).unwrap().offset, 2);
    assert!(Arc::ptr_eq(
        &old.values,
        &backend.states.get(&1).unwrap().values
    ));
    let reference = backend.executor.step(&old, vec![9]).await?;
    let reordered = backend
        .step(vec![input(2, 2, vec![11]), input(1, 2, vec![9])])
        .await?;
    assert_eq!(reordered[1].next_token, Some(reference.greedy_token()));
    let snapshot = backend.states.get(&1).unwrap().clone();
    let mut future = Box::pin(backend.step(vec![input(1, 3, vec![12])]));
    use std::future::Future;
    let w = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(w);
    let pending = future.as_mut().poll(&mut cx).is_pending();
    drop(future);
    if pending {
        assert!(Arc::ptr_eq(
            &snapshot.values,
            &backend.states.get(&1).unwrap().values
        ));
    }
    let mut left = input(1, backend.states[&1].position(), vec![13]);
    left.allowed_tokens = Some(vec![21]);
    let mut right = input(2, backend.states[&2].position(), vec![14]);
    right.allowed_tokens = Some(vec![22]);
    let masked = backend.step(vec![left, right]).await?;
    assert_eq!(masked[0].next_token, Some(21));
    assert_eq!(masked[1].next_token, Some(22));
    let old = backend.states[&1].clone();
    let mut bad = input(1, old.position(), vec![15]);
    bad.allowed_tokens = Some(vec![]);
    assert!(backend.step(vec![bad]).await.is_err());
    assert!(Arc::ptr_eq(&old.values, &backend.states[&1].values));
    backend.release(1).await?;
    assert!(!backend.states.contains_key(&1));
    backend.shutdown().await?;
    assert_eq!(backend.executor.resources().leases, 0);
    Ok(())
}
#[tokio::test]
async fn context_bound_and_scheduler_drain_with_three_requests() -> Result<()> {
    use crate::serving::{config::Config, scheduler::Engine};
    let fixture = Fixture::new()?;
    let executor = fixture.open(Device::Gpu)?;
    let mut state = executor.empty_state();
    for _ in 0..64 {
        state = executor
            .step(&state, vec![1, 2, 3, 4, 5, 6, 7, 8])
            .await?
            .state;
    }
    assert_eq!(state.position(), 512);
    assert!(executor.step(&state, vec![1]).await.is_err());
    assert_eq!(state.position(), 512);
    let mut adapter = ScheduledQwen::new(executor, vec![]);
    let storage = adapter.storage();
    let input = |sequence| StepInput {
        remaining_output_tokens: 8,
        kind: StepKind::Prefill,
        sequence,
        position: 0,
        tokens: vec![2],
        prompt_complete: true,
        history: vec![],
        allowed_tokens: None,
    };
    adapter.step(vec![input(1), input(2), input(3)]).await?;
    assert!(adapter.step(vec![input(4)]).await.is_err());
    adapter.release(2).await?;
    adapter.step(vec![input(4)]).await?;
    adapter.release(1).await?;
    adapter.release(3).await?;
    adapter.release(4).await?;
    let config = Config {
        version: 1,
        max_queued: 3,
        max_sequences: 3,
        max_batch_tokens: 24,
        prefill_chunk_tokens: 8,
        max_context_tokens: 512,
        max_new_tokens: 8,
        model_memory_bytes: 1024 * 1024 * 1024,
        max_in_flight: 3,
    };
    let engine = Engine::new(config, storage, adapter)?;
    let first = engine.submit(vec![1, 2, 3], 4)?;
    let cancelled = engine.submit(vec![5, 6, 7], 4)?;
    let last = engine.submit(vec![9, 10, 11], 4)?;
    cancelled.cancel();
    drop(cancelled);
    let a = first.result().await?;
    let b = last.result().await?;
    assert_eq!(a.tokens.len(), 4);
    assert_eq!(b.tokens.len(), 4);
    assert_ne!(a.sequence, b.sequence);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.cancelled, 1);
    assert_eq!(stats.memory_used, 0);
    assert!(!stats.admission_closed);
    Ok(())
}

#[tokio::test]
async fn malformed_ir_and_native_layer_program_reject_before_publication() -> Result<()> {
    use super::super::model_ir::ModelStepIr;
    let fixture = Fixture::new()?;
    let executor = fixture.open(Device::Cpu)?;
    let empty = executor.empty_state();
    for ir in [
        ModelStepIr::compile(StepKind::Decode, 1, vec![1], 64)?,
        ModelStepIr::compile(StepKind::Decode, 0, vec![1], 63)?,
    ] {
        assert!(executor.execute(&empty, ir).await.is_err());
    }
    assert_eq!(executor.resources().leases, 0);
    assert_eq!(executor.program().layers().len(), 4);
    assert_eq!(executor.shutdown().await?.leases, 0);
    let weights = NativeWeights::open(&fixture.manifest)?;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct WireLayer {
        index: u32,
        mixer: u32,
    }
    let good: Vec<_> = (0..4)
        .map(|i| WireLayer {
            index: i,
            mixer: u32::from(i == 3),
        })
        .collect();
    let mut state = vec![0.; fixture.c.state_len(1)];
    let mut logits = vec![0.; 64];
    let tokens = [1u32];
    for fault in 0..4 {
        let mut layers = good.clone();
        let mut count = 4;
        match fault {
            0 => count = 3,
            1 => layers[0].index = 1,
            2 => layers[0].mixer = 2,
            _ => layers[0].mixer = 1,
        }
        let mut e = [0 as c_char; 1024];
        let status = unsafe {
            rs_qwen_forward(
                weights.0,
                0,
                &fixture.c,
                layers.as_ptr().cast(),
                count,
                tokens.as_ptr(),
                1,
                0,
                std::ptr::null(),
                0,
                state.as_mut_ptr(),
                state.len(),
                logits.as_mut_ptr(),
                logits.len(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                e.as_mut_ptr(),
                e.len(),
            )
        };
        assert_ne!(status, 0);
        assert!(state.iter().all(|v| *v == 0.));
    }
    for (pointer, length) in [
        (std::ptr::null_mut(), 640usize),
        (std::ptr::null_mut(), usize::MAX),
    ] {
        let mut error = [0 as c_char; 1024];
        let status = unsafe {
            rs_qwen_forward(
                weights.0,
                0,
                &fixture.c,
                good.as_ptr().cast(),
                4,
                tokens.as_ptr(),
                1,
                0,
                std::ptr::null(),
                0,
                state.as_mut_ptr(),
                state.len(),
                logits.as_mut_ptr(),
                logits.len(),
                0,
                std::ptr::null(),
                0,
                pointer,
                length,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        assert_ne!(status, 0);
        assert!(state.iter().all(|v| *v == 0.));
    }
    let mut e = [0 as c_char; 1024];
    let status = unsafe {
        rs_qwen_forward(
            weights.0,
            0,
            &fixture.c,
            good.as_ptr().cast(),
            4,
            tokens.as_ptr(),
            1,
            0,
            std::ptr::null(),
            0,
            state.as_mut_ptr(),
            state.len(),
            logits.as_mut_ptr(),
            logits.len(),
            0,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            0,
            e.as_mut_ptr(),
            e.len(),
        )
    };
    native_status(status, &e)?;
    assert!(logits.iter().all(|v| v.is_finite()));
    Ok(())
}
