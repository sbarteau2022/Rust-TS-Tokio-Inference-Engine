use rust_server::{
    completion::FlightGate,
    execute,
    gpu::{Gpu, Uma},
    ir,
    runtime::{Job, Runtime},
};
use std::time::Duration;
#[tokio::test]
async fn semaphore_caps_and_canceled_waiters_do_not_leak() {
    for limit in [2, 3] {
        let gate = FlightGate::new(limit).unwrap();
        let mut permits = Vec::new();
        for _ in 0..limit {
            permits.push(gate.acquire().await.unwrap());
        }
        assert!(gate.try_acquire().is_err());
        assert!(
            tokio::time::timeout(Duration::from_millis(5), gate.acquire())
                .await
                .is_err()
        );
        drop(permits.pop());
        let acquired = tokio::time::timeout(Duration::from_secs(1), gate.acquire())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(gate.stats().active, limit);
        drop(acquired);
        drop(permits);
        let s = gate.stats();
        assert_eq!(s.active, 0);
        assert_eq!(s.peak, limit);
        assert_eq!(s.acquired, s.released);
    }
}
#[tokio::test]
async fn wrong_gate_and_encoding_errors_return_permits_without_submission() {
    let gate = FlightGate::new(2).unwrap();
    let other = FlightGate::new(3).unwrap();
    let gpu = Gpu::with_gate(gate.clone()).unwrap();
    let pipeline=gpu.compile("#include <metal_stdlib>\nusing namespace metal;kernel void proof(device int* A [[buffer(0)]],uint i [[thread_position_in_grid]]){if(i==0)A[0]=42;}","proof").unwrap();
    let b = Uma::allocate(&gpu.device, 4, 64).unwrap();
    let groups = metal::MTLSize::new(1, 1, 1);
    assert!(gpu
        .submit(&pipeline, &[&b], groups, 32, other.acquire().await.unwrap())
        .is_err());
    assert_eq!(other.stats().active, 0);
    assert_eq!(gate.stats().submitted, 0);
    assert!(gpu
        .submit(&pipeline, &[&b], groups, 0, gate.acquire().await.unwrap())
        .is_err());
    assert_eq!(gate.stats().active, 0);
    assert_eq!(gate.stats().submitted, 0);
    gpu.submit(&pipeline, &[&b], groups, 32, gate.acquire().await.unwrap())
        .unwrap()
        .wait()
        .await
        .unwrap();
    assert_eq!(b.read_i32().unwrap(), vec![42]);
    assert_eq!(gate.stats().completed, 1);
}
#[tokio::test(flavor = "current_thread")]
async fn both_limits_route_six_families_under_pressure_and_recover() {
    let traces: [&[u8]; 6] = [
        include_bytes!("../fixtures/square.amvk"),
        include_bytes!("../fixtures/elementwise.amvk"),
        include_bytes!("../fixtures/sin.amvk"),
        include_bytes!("../fixtures/cos.amvk"),
        include_bytes!("../fixtures/atan2.amvk"),
        include_bytes!("../fixtures/tanh.amvk"),
    ];
    for limit in [2, 3] {
        let runtime = Runtime::with_in_flight(8, limit).unwrap();
        let mut accepted = 0;
        let mut rejected = 0;
        let mut pending = Vec::new();
        for seed in 0..100 {
            let trace = traces[seed as usize % 6];
            let plan = ir::lower_trace(trace).unwrap();
            let inputs = execute::seeded_inputs(&plan, seed);
            let expected = execute::reference(&plan, &inputs).unwrap();
            match runtime.submit(Job {
                trace: trace.to_vec(),
                inputs,
            }) {
                Ok(receiver) => {
                    accepted += 1;
                    pending.push((receiver, expected));
                }
                Err(_) => rejected += 1,
            }
        }
        assert!(rejected > 0);
        assert!(accepted >= 6);
        // Drop a reply while work is accepted; it must still drain through completion.
        drop(pending.pop());
        for (receiver, expected) in pending {
            let receipt = tokio::time::timeout(Duration::from_secs(20), receiver)
                .await
                .unwrap()
                .unwrap();
            assert!(receipt.failure.is_none(), "{:?}", receipt.failure);
            assert_eq!(receipt.output.as_ref().unwrap().values, expected);
            assert!(receipt.worker_id < limit);
            assert!(receipt.flight.peak <= limit);
            assert!(receipt.flight.peak_outstanding <= limit);
            assert!(
                receipt
                    .output
                    .unwrap()
                    .metrics
                    .resources
                    .peak_resident_bytes
                    <= 64 * 1024 * 1024 / limit
            );
        }
        let trace = traces[2];
        let plan = ir::lower_trace(trace).unwrap();
        let failure = runtime
            .submit(Job {
                trace: trace.to_vec(),
                inputs: Default::default(),
            })
            .unwrap()
            .await
            .unwrap();
        assert_eq!(failure.failure.unwrap().stage, "inputs");
        let recovered = runtime
            .submit(Job {
                trace: trace.to_vec(),
                inputs: execute::seeded_inputs(&plan, 81),
            })
            .unwrap()
            .await
            .unwrap();
        assert!(recovered.failure.is_none());
        // A dropped reply can still be finishing on another lane: await its callback too.
        let s = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let s = runtime.flight_stats();
                if s.completed == accepted + 1 && s.active == 0 {
                    break s;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(s.submitted, accepted + 1);
        assert_eq!(s.completed, s.submitted);
        assert_eq!(s.active, 0);
        assert_eq!(s.acquired, s.released);
        assert_eq!(s.completion_errors, 0);
        println!("limit {limit}: {accepted} admitted, {rejected} rejected; {} callbacks; peak encoded {}, peak outstanding {}",s.completed,s.peak,s.peak_outstanding);
    }
}
#[tokio::test]
async fn dropping_runtime_drains_accepted_jobs() {
    let runtime = Runtime::with_in_flight(8, 3).unwrap();
    let bytes = include_bytes!("../fixtures/tanh.amvk");
    let plan = ir::lower_trace(bytes).unwrap();
    let mut pending = Vec::new();
    for seed in 0..8 {
        pending.push(
            runtime
                .submit(Job {
                    trace: bytes.to_vec(),
                    inputs: execute::seeded_inputs(&plan, seed),
                })
                .unwrap(),
        );
    }
    drop(runtime);
    for rx in pending {
        let receipt = tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .unwrap()
            .unwrap();
        assert!(receipt.failure.is_none());
    }
}
#[tokio::test]
async fn unsupported_limits_fail_before_work_starts() {
    let default_runtime = Runtime::new(8).unwrap();
    assert_eq!(default_runtime.flight_stats().limit, 3);
    drop(default_runtime);
    for limit in [0, 1, 4, usize::MAX] {
        assert!(FlightGate::new(limit).is_err());
        assert!(Runtime::with_in_flight(8, limit).is_err());
    }
}
