use anyhow::{ensure, Result};
use rust_server::serving::{
    config::Config,
    kv::{Codec, TurboProfile},
    scheduler::*,
    ByteBudget,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

fn config() -> Config {
    Config {
        version: 1,
        max_queued: 8,
        max_sequences: 3,
        max_batch_tokens: 4,
        prefill_chunk_tokens: 2,
        max_context_tokens: 64,
        max_new_tokens: 32,
        model_memory_bytes: 65536,
        max_in_flight: 3,
    }
}
fn profile(width: usize) -> TurboProfile {
    let mut rotation_q30 = vec![0; width * width];
    for i in 0..width {
        rotation_q30[i * width + i] = 1 << 30;
    }
    TurboProfile {
        version: 1,
        width,
        bits: 2,
        rotation_q30,
        centers_q31: vec![-2147483647, -715827883, 715827883, 2147483647],
    }
}
#[test]
fn regular_exact_lifetimes_and_budget_pressure() {
    let c = Codec::regular(4).unwrap();
    let b = ByteBudget::new(32).unwrap();
    let x = [i32::MIN, i32::MAX, 0, -1];
    let f = c.encode(&x, &x, &b).unwrap();
    assert_eq!(c.decode(&f).unwrap().keys, x);
    assert_eq!(c.decode(&f).unwrap().values, x);
    let retained = f.clone();
    drop(f);
    assert_eq!(b.used(), 32);
    assert!(c.encode(&x, &x, &b).is_err());
    drop(retained);
    assert_eq!(b.used(), 0);
    for _ in 0..10_000 {
        let f = c.encode(&x, &x, &b).unwrap();
        drop(f);
    }
    assert_eq!(b.used(), 0);
    assert_eq!(b.peak(), 32);
    assert!(b.reserve(usize::MAX).is_err());
}
#[test]
fn turbo_integer_shapes_identity_and_untrusted_bytes() {
    let c = Codec::turbo_mse(profile(3)).unwrap();
    let b = ByteBudget::new(4096).unwrap();
    let f = c.encode(&[1, 2, 3], &[0, 0, 0], &b).unwrap();
    let d = c.decode(&f).unwrap();
    assert_eq!(d.values, [0, 0, 0]);
    assert_eq!(d.keys, [1, 2, 3]);
    assert!(Codec::regular(3).unwrap().decode(&f).is_err());
    for len in 0..c.frame_bytes() {
        assert!(c.decode_bytes(&vec![0; len]).is_err());
    }
    let mut bad = f.bytes().to_vec();
    *bad.last_mut().unwrap() |= 0xc0;
    assert!(c.decode_bytes(&bad).is_err());
    let mut bad = f.bytes().to_vec();
    bad[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(c.decode_bytes(&bad).is_err());
    let mut p = profile(3);
    p.rotation_q30[0] = 0;
    assert!(Codec::turbo_mse(p).is_err());
    let mut p = profile(3);
    p.centers_q31.swap(0, 1);
    assert!(Codec::turbo_mse(p).is_err());
    let mut p = profile(3);
    p.bits = 8;
    assert!(Codec::turbo_mse(p).is_err());
    let mut p = profile(3);
    p.rotation_q30[0] = i32::MIN;
    assert!(Codec::turbo_mse(p).is_err());
    let serialized = serde_json::to_string(&profile(3))
        .unwrap()
        .replace("\"bits\":2", "\"bits\":2,\"surprise\":1");
    assert!(serde_json::from_str::<TurboProfile>(&serialized).is_err());
}
#[test]
fn turbo_rotation_inverse_and_extreme_integer_oracle() {
    // A signed permutation is exactly orthogonal; hand-computed norm and reconstruction.
    let mut p = profile(2);
    p.rotation_q30 = vec![0, 1 << 30, -(1 << 30), 0];
    let c = Codec::turbo_mse(p).unwrap();
    let b = ByteBudget::new(4096).unwrap();
    let f = c.encode(&[9, 10], &[0, 1 << 30], &b).unwrap();
    let d = c.decode(&f).unwrap();
    assert_eq!(d.values, vec![357913942, 1073741824]); // nearest-center tie selects the lower index, then inverse rotation.
    let mut rng = 17u64;
    for _ in 0..10_000 {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = rng as i32;
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let y = rng as i32;
        let a = c.encode(&[x, y], &[x, y], &b).unwrap();
        let again = c.encode(&[x, y], &[x, y], &b).unwrap();
        assert_eq!(a.bytes(), again.bytes());
        assert_eq!(c.decode(&a).unwrap().keys, [x, y]);
    }
}
#[test]
fn global_byte_budget_concurrent_reservations() {
    let budget = ByteBudget::new(1024).unwrap();
    let mut threads = Vec::new();
    for _ in 0..8 {
        let b = budget.clone();
        threads.push(std::thread::spawn(move || {
            for _ in 0..10000 {
                if let Ok(c) = b.reserve(256) {
                    assert!(b.used() <= 1024);
                    std::thread::yield_now();
                    drop(c);
                }
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(budget.used(), 0);
    assert!(budget.peak() <= 1024);
}
#[derive(Clone)]
struct Oracle {
    boundaries: Arc<AtomicUsize>,
    boundary_failure: bool,
    codec: Codec,
    seen: Arc<Mutex<Vec<(u64, usize, usize)>>>,
    released: Arc<AtomicUsize>,
    fault: u8,
    gate: Option<Arc<tokio::sync::Semaphore>>,
    started: Arc<tokio::sync::Notify>,
}
fn oracle() -> Oracle {
    Oracle {
        boundaries: Arc::default(),
        boundary_failure: false,
        codec: Codec::regular(2).unwrap(),
        seen: Arc::default(),
        released: Arc::default(),
        fault: 0,
        gate: None,
        started: Arc::default(),
    }
}
impl ModelBackend for Oracle {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: "SYNTHETIC-NONMODEL-integer-oracle".into(),
            width: 2,
            vocabulary_size: 1000,
            max_context_tokens: 64,
            workspace_bytes: 32,
            max_in_flight: 3,
            eos_tokens: vec![999],
        }
    }
    fn validated_boundary(&mut self) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.boundaries.fetch_add(1, Ordering::SeqCst);
            ensure!(!self.boundary_failure, "injected audit failure");
            Ok(())
        })
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            self.started.notify_one();
            if let Some(gate) = &self.gate {
                gate.acquire().await?.forget();
            }
            let mut result = Vec::new();
            for input in batch {
                ensure!(input.position == input.history.len(), "history position");
                self.seen.lock().unwrap().push((
                    input.sequence,
                    input.position,
                    input.tokens.len(),
                ));
                let old = input
                    .history
                    .iter()
                    .map(|f| self.codec.decode(f).map(|d| d.keys[0] as u32))
                    .collect::<Result<Vec<_>>>()?
                    .iter()
                    .sum::<u32>();
                let new = input.tokens.iter().sum::<u32>();
                let token = input.prompt_complete.then_some((old + new + 1) % 1000);
                let kv = input
                    .tokens
                    .iter()
                    .map(|t| KvValues {
                        keys: vec![*t as i32; 2],
                        values: vec![*t as i32; 2],
                    })
                    .collect();
                let mut output = StepOutput {
                    sequence: input.sequence,
                    kv,
                    next_token: token,
                    state_bytes: 0,
                };
                match self.fault {
                    1 => output.sequence += 10000,
                    2 => {
                        output.kv.pop();
                    }
                    3 => output.next_token = Some(1000),
                    4 => output.next_token = None,
                    5 => anyhow::bail!("injected backend fault"),
                    6 => output.state_bytes = 1,
                    10 => panic!("injected model backend panic"),
                    _ => {}
                }
                result.push(output);
            }
            if self.fault == 8 {
                let id = result[0].sequence;
                for output in &mut result {
                    output.sequence = id;
                }
            }
            if self.fault == 9 {
                result.reverse();
            }
            Ok(result)
        })
    }
    fn release(&mut self, _id: u64) -> ReleaseFuture<'_> {
        Box::pin(async move {
            self.released.fetch_add(1, Ordering::Relaxed);
            if self.fault == 7 {
                anyhow::bail!("release failure");
            }
            Ok(())
        })
    }
}
#[tokio::test]
async fn scheduler_real_kv_consumption_chunk_invariance_and_drain() -> Result<()> {
    let model = oracle();
    let seen = model.seen.clone();
    let released = model.released.clone();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let a = engine.submit(vec![1, 2, 3, 4, 5], 4)?;
    let b = engine.submit(vec![8], 3)?;
    let a = a.result().await?;
    let b = b.result().await?;
    for generation in [&a, &b] {
        assert_eq!(generation.token_times_ms.len(), generation.tokens.len());
        assert!(generation
            .token_times_ms
            .iter()
            .all(|t| t.is_finite() && *t >= 0.));
        assert!(generation.token_times_ms.windows(2).all(|w| w[1] >= w[0]));
        assert!(generation.elapsed_ms >= *generation.token_times_ms.last().unwrap());
    }
    assert_eq!(a.tokens, [16, 32, 64, 128]);
    assert_eq!(b.tokens, [9, 18, 36]);
    assert_eq!(a.finish_reason, "length");
    assert!(!a.experimental_codec);
    assert_eq!(a.iterations, 6);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.memory_used, 0);
    assert_eq!(released.load(Ordering::Relaxed), 2);
    let trace = seen.lock().unwrap();
    assert!(trace.iter().all(|v| v.2 <= 2));
    assert!(trace.iter().take(3).any(|v| v.0 == 2));
    Ok(())
}
#[tokio::test]
async fn cancellation_retains_inflight_state_until_completion() -> Result<()> {
    let mut model = oracle();
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    model.gate = Some(gate.clone());
    let started = model.started.clone();
    let released = model.released.clone();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let request = engine.submit(vec![1, 2], 10)?;
    started.notified().await;
    assert!(engine.stats().memory_used > 0);
    drop(request);
    tokio::task::yield_now().await;
    assert!(engine.stats().memory_used > 0);
    assert_eq!(released.load(Ordering::Relaxed), 0);
    gate.add_permits(1);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.cancelled, 1);
    assert_eq!(stats.memory_used, 0);
    assert_eq!(released.load(Ordering::Relaxed), 1);
    Ok(())
}
#[tokio::test]
async fn malformed_backend_batches_and_release_errors_latch_admission() -> Result<()> {
    for fault in 1..=7 {
        let mut model = oracle();
        model.fault = fault;
        let engine = Engine::new(config(), model.codec.clone(), model)?;
        let request = engine.submit(vec![1], 1)?;
        assert!(request.result().await.is_err(), "fault {fault}");
        assert!(engine.submit(vec![1], 1).is_err());
        let stats = engine.shutdown().await?;
        assert!(stats.admission_closed);
        assert_eq!(stats.memory_used, 0);
        assert_eq!(stats.failed, 1);
    }
    Ok(())
}
#[tokio::test]
async fn scheduler_oom_invalid_requests_and_eos() -> Result<()> {
    let mut cfg = config();
    cfg.model_memory_bytes = 40;
    let model = oracle();
    let engine = Engine::new(cfg, model.codec.clone(), model)?;
    assert!(engine.submit(vec![], 1).is_err());
    assert!(engine.submit(vec![1000], 1).is_err());
    assert!(engine.submit(vec![1; 64], 2).is_err());
    assert!(engine.submit(vec![1], 0).is_err());
    assert!(engine.submit(vec![1, 2], 1)?.result().await.is_err());
    assert_eq!(engine.shutdown().await?.memory_used, 0);
    let model = oracle();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let g = engine.submit(vec![998], 10)?.result().await?;
    assert_eq!(g.tokens, [999]);
    assert_eq!(g.finish_reason, "eos");
    engine.shutdown().await?;
    Ok(())
}
#[tokio::test]
async fn ten_thousand_token_steps_remain_bounded() -> Result<()> {
    let model = oracle();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    for _ in 0..1000 {
        let result = engine.submit(vec![1, 2, 3], 10)?.result().await?;
        assert_eq!(result.tokens.len(), 10);
    }
    let stats = engine.shutdown().await?;
    assert_eq!(stats.completed, 1000);
    assert_eq!(stats.memory_used, 0);
    assert!(stats.memory_peak <= 65536);
    Ok(())
}
#[test]
fn config_unknown_fields_and_limits_are_explicit() {
    let mut c = config();
    assert!(c.validate().is_ok());
    let hash = c.fingerprint().unwrap();
    c.prefill_chunk_tokens = 1;
    assert_ne!(hash, c.fingerprint().unwrap());
    c.max_in_flight = 4;
    assert!(c.validate().is_err());
    c = config();
    c.prefill_chunk_tokens = 100;
    assert!(c.validate().is_err());
    let j = serde_json::to_string(&config()).unwrap().replace(
        "\"version\":1",
        "\"version\":1,\"automatic_production_sampling\":true",
    );
    assert!(serde_json::from_str::<Config>(&j).is_err());
}

#[tokio::test]
async fn bounded_admission_does_not_spawn_unbounded_work() -> Result<()> {
    let mut cfg = config();
    cfg.max_queued = 1;
    cfg.max_sequences = 1;
    let mut model = oracle();
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    model.gate = Some(gate.clone());
    let started = model.started.clone();
    let engine = Engine::new(cfg, model.codec.clone(), model)?;
    let first = engine.submit(vec![1], 4)?;
    started.notified().await;
    let second = engine.submit(vec![2], 4)?;
    assert!(engine.submit(vec![3], 4).is_err());
    drop(first);
    drop(second);
    gate.add_permits(1);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.admitted, 2);
    assert_eq!(stats.cancelled, 2);
    assert_eq!(stats.memory_used, 0);
    Ok(())
}
#[tokio::test]
async fn runtime_accepts_both_typed_host_codecs() -> Result<()> {
    for codec in [Codec::regular(2)?, Codec::turbo_mse(profile(2))?] {
        let mut model = oracle();
        model.codec = codec.clone();
        let experimental = codec.experimental();
        let engine = Engine::new(config(), codec, model)?;
        let result = engine.submit(vec![1, 2, 3], 4)?.result().await?;
        assert_eq!(result.tokens, [7, 14, 28, 56]);
        assert_eq!(result.experimental_codec, experimental);
        assert_eq!(engine.shutdown().await?.memory_used, 0);
    }
    Ok(())
}

#[tokio::test]
async fn batch_validation_is_atomic_and_response_order_is_not_assumed() -> Result<()> {
    for fault in [8, 9] {
        let mut model = oracle();
        model.fault = fault;
        let engine = Engine::new(config(), model.codec.clone(), model)?;
        let first = engine.submit(vec![1], 1)?;
        let second = engine.submit(vec![2], 1)?;
        assert_eq!(first.result().await.is_ok(), fault == 9);
        assert_eq!(second.result().await.is_ok(), fault == 9);
        let stats = engine.shutdown().await?;
        assert_eq!(stats.completed, if fault == 9 { 2 } else { 0 });
        assert_eq!(stats.failed, if fault == 8 { 2 } else { 0 });
        assert_eq!(stats.memory_used, 0);
    }
    Ok(())
}

#[tokio::test]
async fn panicked_backend_closes_admission_and_reports_unfinished_work() -> Result<()> {
    let mut model = oracle();
    model.fault = 10;
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    assert!(engine.submit(vec![1], 1)?.result().await.is_err());
    assert!(engine.submit(vec![1], 1).is_err());
    assert!(engine.stats().admission_closed);
    assert_eq!(engine.stats().memory_used, 0);
    assert!(engine.shutdown().await.is_err());
    Ok(())
}

#[tokio::test]
async fn bounded_stream_matches_final_receipt_and_nonstreaming_output() -> Result<()> {
    let model = oracle();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    assert!(engine.submit_stream(vec![1], 4, 0).is_err());
    assert!(engine.submit_stream(vec![1], 4, 129).is_err());
    let mut request = engine.submit_stream(vec![1, 2, 3], 8, 1)?;
    let mut events = Vec::new();
    while let Some(event) = request.next_token().await {
        assert_eq!(event.index, events.len());
        if let Some(previous) = events.last() {
            let previous: &TokenEvent = previous;
            assert_eq!(event.sequence, previous.sequence);
            assert!(event.elapsed_ms >= previous.elapsed_ms);
        }
        events.push(event);
    }
    let generation = request.result().await?;
    assert_eq!(
        generation.tokens,
        events.iter().map(|e| e.token).collect::<Vec<_>>()
    );
    assert_eq!(
        generation.token_times_ms,
        events.iter().map(|e| e.elapsed_ms).collect::<Vec<_>>()
    );
    let baseline = engine.submit(vec![1, 2, 3], 8)?.result().await?;
    assert_eq!(generation.tokens, baseline.tokens);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.memory_used, 0);
    Ok(())
}

#[tokio::test]
async fn backpressured_stream_yields_other_lanes_and_result_drains() -> Result<()> {
    let model = oracle();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let mut slow = engine.submit_stream(vec![1], 10, 1)?;
    assert!(slow.next_token().await.is_some());
    let fast = engine.submit(vec![2], 4)?;
    let result = tokio::time::timeout(std::time::Duration::from_secs(2), fast.result()).await??;
    assert_eq!(result.tokens.len(), 4);
    let iterations = engine.stats().iterations;
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        engine.stats().iterations,
        iterations,
        "full stream must pause native encoding"
    );
    let rest = tokio::time::timeout(std::time::Duration::from_secs(2), slow.result()).await??;
    assert_eq!(rest.tokens.len(), 10);
    let stats = engine.shutdown().await?;
    assert_eq!(stats.completed, 2);
    assert_eq!(stats.memory_used, 0);
    Ok(())
}

#[tokio::test]
async fn undrained_stream_shutdown_reports_gap_and_drop_cancels() -> Result<()> {
    let model = oracle();
    let seen = model.seen.clone();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let mut blocked = engine.submit_stream(vec![1], 10, 1)?;
    assert!(blocked.next_token().await.is_some());
    while seen.lock().unwrap().len() < 2 {
        tokio::task::yield_now().await;
    }
    let stats =
        tokio::time::timeout(std::time::Duration::from_secs(2), engine.shutdown()).await??;
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.memory_used, 0);
    assert!(blocked.result().await.is_err());
    let model = oracle();
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let mut canceled = engine.submit_stream(vec![1], 10, 1)?;
    assert!(canceled.next_token().await.is_some());
    drop(canceled);
    let stats =
        tokio::time::timeout(std::time::Duration::from_secs(2), engine.shutdown()).await??;
    assert_eq!(stats.cancelled, 1);
    assert_eq!(stats.memory_used, 0);
    let mut model = oracle();
    model.fault = 1;
    let engine = Engine::new(config(), model.codec.clone(), model)?;
    let mut bad = engine.submit_stream(vec![1], 4, 1)?;
    assert!(
        bad.next_token().await.is_none(),
        "unvalidated batch leaked a token"
    );
    assert!(bad.result().await.is_err());
    assert!(engine.shutdown().await?.admission_closed);
    Ok(())
}

#[tokio::test]
async fn validated_boundary_precedes_publication_and_fails_closed() -> Result<()> {
    for fault in [0, 1] {
        let mut backend = oracle();
        backend.fault = fault;
        let count = backend.boundaries.clone();
        let engine = Engine::new(config(), backend.codec.clone(), backend)?;
        let mut request = engine.submit_stream(vec![1], 1, 1)?;
        if fault == 0 {
            assert!(request.next_token().await.is_some());
            assert_eq!(count.load(Ordering::SeqCst), 1);
            assert_eq!(request.result().await?.tokens.len(), 1);
        } else {
            assert!(request.next_token().await.is_none());
            assert!(request.result().await.is_err());
            assert_eq!(count.load(Ordering::SeqCst), 0);
        }
        let stats = engine.shutdown().await?;
        assert_eq!(stats.memory_used, 0);
    }
    let mut backend = oracle();
    backend.boundary_failure = true;
    let count = backend.boundaries.clone();
    let engine = Engine::new(config(), backend.codec.clone(), backend)?;
    let mut request = engine.submit_stream(vec![1], 2, 1)?;
    assert!(request.next_token().await.is_none());
    assert!(request.result().await.is_err());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    let stats = engine.shutdown().await?;
    assert!(stats.admission_closed);
    assert_eq!(stats.memory_used, 0);
    Ok(())
}
