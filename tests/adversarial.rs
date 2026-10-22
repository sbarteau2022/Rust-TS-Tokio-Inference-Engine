use rust_server::{
    execute::{compare, execute, seeded_inputs, validate_inputs},
    ir::{lower, Plan},
    runtime::{Job, Runtime},
    wire::{decode, Op},
};
use std::collections::BTreeSet;
fn ops() -> Vec<Op> {
    decode(include_bytes!("../fixtures/square.amvk")).unwrap()
}
fn shape(m: u32, k: u32, n: u32) -> Plan {
    let mut x = ops();
    x[0] = Op::Load {
        name: "A".into(),
        rows: m,
        cols: k,
        out: 0,
    };
    x[1] = Op::Load {
        name: "B".into(),
        rows: k,
        cols: n,
        out: 1,
    };
    x[2] = Op::Load {
        name: "bias".into(),
        rows: 1,
        cols: n,
        out: 2,
    };
    lower(&x).unwrap()
}
#[test]
fn malformed_wire_never_panics() {
    let original = include_bytes!("../fixtures/square.amvk");
    for len in 0..original.len() {
        if let Ok(decoded) = decode(&original[..len]) {
            assert_eq!(decoded, ops(), "padding truncation changed graph at {len}");
        }
    }
    let mut seed = 7u32;
    for i in 0..10000 {
        let mut bytes = vec![0u8; 8 + i % 513];
        for b in &mut bytes {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *b = (seed >> 16) as u8;
        }
        let result = std::panic::catch_unwind(|| decode(&bytes));
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }
    let mut wrong = original.to_vec();
    wrong[4..8].copy_from_slice(b"EVIL");
    assert!(decode(&wrong).is_err());
    assert!(decode(&vec![0; 65537]).is_err());
}
#[test]
fn mutations_fail_closed() {
    let base = ops();
    let mut cases = Vec::new();
    let mut x = base.clone();
    x[0] = Op::Load {
        name: "A".into(),
        rows: 0,
        cols: 128,
        out: 0,
    };
    cases.push(x);
    let mut x = base.clone();
    x[0] = Op::Load {
        name: "A".into(),
        rows: 4097,
        cols: 128,
        out: 0,
    };
    cases.push(x);
    let mut x = base.clone();
    x[1] = Op::Load {
        name: "A".into(),
        rows: 128,
        cols: 128,
        out: 1,
    };
    cases.push(x);
    let mut x = base.clone();
    x[1] = Op::Load {
        name: "B".into(),
        rows: 127,
        cols: 128,
        out: 1,
    };
    cases.push(x);
    let mut x = base.clone();
    x[1] = Op::Load {
        name: "B".into(),
        rows: 128,
        cols: 128,
        out: 0,
    };
    cases.push(x);
    let mut x = base.clone();
    x[3] = Op::Dot {
        a: 44,
        b: 1,
        out: 3,
    };
    cases.push(x);
    let mut x = base.clone();
    x[4] = Op::Add { a: 2, b: 3, out: 4 };
    cases.push(x);
    let mut x = base.clone();
    x[5] = Op::Relu { a: 3, out: 5 };
    cases.push(x);
    let mut x = base.clone();
    x[6] = Op::Store {
        a: 5,
        name: "A".into(),
    };
    cases.push(x);
    let mut x = base.clone();
    x.push(Op::Store {
        a: 5,
        name: "D".into(),
    });
    cases.push(x);
    let mut x = base.clone();
    x[0] = Op::Load {
        name: "bad;shader".into(),
        rows: 128,
        cols: 128,
        out: 0,
    };
    cases.push(x);
    for x in cases {
        assert!(lower(&x).is_err(), "accepted {x:?}");
    }
    for i in 0..base.len() {
        let mut x = base.clone();
        x.remove(i);
        assert!(lower(&x).is_err());
    }
    let mut x = base.clone();
    x[0] = Op::Load {
        name: "A".into(),
        rows: 4096,
        cols: 4096,
        out: 0,
    };
    x[1] = Op::Load {
        name: "B".into(),
        rows: 4096,
        cols: 4096,
        out: 1,
    };
    x[2] = Op::Load {
        name: "bias".into(),
        rows: 1,
        cols: 4096,
        out: 2,
    };
    assert!(lower(&x).is_err());
}
#[test]
fn integer_json_and_binding_rejection() {
    let plan = shape(8, 8, 8);
    for raw in [
        "{\"A\":[0.5]}",
        "{\"A\":[2147483648]}",
        "{\"A\":[-2147483649]}",
        "{\"A\":[NaN]}",
    ] {
        assert!(serde_json::from_str::<rust_server::execute::Inputs>(raw).is_err());
    }
    let mut inputs = seeded_inputs(&plan, 42);
    inputs.get_mut("A").unwrap().pop();
    assert!(validate_inputs(&plan, &inputs).is_err());
    let mut inputs = seeded_inputs(&plan, 42);
    inputs.insert("injected".into(), vec![]);
    assert!(validate_inputs(&plan, &inputs).is_err());
    assert!(compare(&[1], &[0]).is_err());
    assert!(compare(&[], &[0]).is_err());
}
#[test]
fn gpu_edge_cases() {
    for (m, k, n) in [
        (1, 1, 1),
        (7, 9, 13),
        (8, 8, 8),
        (16, 17, 8),
        (73, 29, 91),
        (128, 128, 128),
        (31, 257, 19),
    ] {
        let plan = shape(m, k, n);
        let inputs = seeded_inputs(&plan, 123);
        let (out, _, _) = execute(&plan, &inputs, 1).unwrap();
        assert_eq!(out.values.len(), (m * n) as usize);
        assert!(out.values.iter().all(|v| *v >= 0));
    }
    let plan = shape(9, 11, 7);
    let mut inputs = seeded_inputs(&plan, 2);
    for v in inputs.values_mut() {
        v.fill(0)
    }
    inputs.get_mut("bias").unwrap().fill(-1);
    assert!(execute(&plan, &inputs, 1)
        .unwrap()
        .0
        .values
        .iter()
        .all(|v| *v == 0));
    let mut inputs = seeded_inputs(&plan, 2);
    inputs.get_mut("A").unwrap().fill(i32::MAX);
    inputs.get_mut("B").unwrap().fill(i32::MAX);
    assert!(execute(&plan, &inputs, 1).is_ok());
    assert!(execute(&plan, &inputs, 0).is_err());
}
#[test]
fn uma_alignment_and_budgets() {
    let gpu = rust_server::gpu::Gpu::new().unwrap();
    assert_eq!(gpu.identity_judge().unwrap()["pass"], true);
    for (len, alignment) in [
        (0, 64),
        (16, 0),
        (16, 1),
        (16, 63),
        (16, 65),
        (usize::MAX, 64),
    ] {
        assert!(rust_server::gpu::Uma::allocate(&gpu.device, len, alignment).is_err());
    }
    let mut b = rust_server::gpu::Uma::allocate(&gpu.device, 64, 64).unwrap();
    assert!(b.write(&[1; 63]).is_err());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pressure_failure_recovery_and_cancellation() {
    let runtime = Runtime::new(4).unwrap();
    let bytes = include_bytes!("../fixtures/square.amvk").to_vec();
    let plan = lower(&decode(&bytes).unwrap()).unwrap();
    let mut pending = Vec::new();
    let mut rejected = 0;
    for seed in 0..100 {
        match runtime.submit(Job {
            trace: bytes.clone(),
            inputs: seeded_inputs(&plan, seed),
        }) {
            Ok(rx) => pending.push(rx),
            Err(_) => rejected += 1,
        }
    }
    assert!(rejected > 0);
    let mut ids = BTreeSet::new();
    for rx in pending {
        let r = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .unwrap()
            .unwrap();
        assert!(r.failure.is_none(), "{:?}", r.failure);
        assert!(r.signpost_id > 0);
        assert!(ids.insert(r.job_id));
    }
    let mut malformed = bytes.clone();
    malformed[4] = 0;
    let failure = runtime
        .submit(Job {
            trace: malformed,
            inputs: seeded_inputs(&plan, 1),
        })
        .unwrap()
        .await
        .unwrap();
    assert_eq!(failure.failure.unwrap().stage, "decode");
    let mut bad = seeded_inputs(&plan, 1);
    bad.get_mut("A").unwrap().pop();
    let r = runtime
        .submit(Job {
            trace: bytes.clone(),
            inputs: bad,
        })
        .unwrap()
        .await
        .unwrap();
    assert_eq!(r.failure.unwrap().stage, "inputs");
    let cancelled = runtime
        .submit(Job {
            trace: bytes.clone(),
            inputs: seeded_inputs(&plan, 3),
        })
        .unwrap();
    drop(cancelled);
    let r = runtime
        .submit(Job {
            trace: bytes.clone(),
            inputs: seeded_inputs(&plan, 4),
        })
        .unwrap()
        .await
        .unwrap();
    assert!(r.failure.is_none());
    assert!(runtime
        .submit(Job {
            trace: vec![],
            inputs: Default::default()
        })
        .is_err());
    assert!(Runtime::new(0).is_err());
    assert!(Runtime::new(33).is_err());
    println!("pressure: rejected {rejected}/100; valid jobs completed, decode/input failures isolated, cancellation recovered");
}
#[test]
fn unsupported_version_union_and_operation_budget() {
    use rust_server::trace_generated::wire as w;
    for (version, count, kind) in [
        (1, 1, 0),
        (2, 1, 0),
        (3, 0, 0),
        (3, 65, 0),
        (3, 1, 255),
        (3, 1, 1),
    ] {
        let mut b = flatbuffers::FlatBufferBuilder::new();
        let op = w::Op::create(
            &mut b,
            &w::OpArgs {
                operation_type: w::Operation(kind),
                operation: None,
            },
        );
        let v = b.create_vector(&vec![op; count]);
        let t = w::Trace::create(
            &mut b,
            &w::TraceArgs {
                version,
                ops: Some(v),
            },
        );
        w::finish_trace_buffer(&mut b, t);
        assert!(decode(b.finished_data()).is_err());
    }
}
#[test]
fn residency_is_consumed_by_emitter() {
    let mut plan = shape(8, 8, 8);
    assert!(plan.msl().unwrap().contains("simd_shuffle_down"));
    plan.tiles.get_mut(&0).unwrap().residency = "device";
    assert!(plan.msl().is_err());
    let mut plan = shape(8, 8, 8);
    plan.tiles.get_mut(&0).unwrap().layout = "transposed";
    assert!(plan.msl().is_err());
}

#[test]
fn valid_wire_corruption_is_contained() {
    let original = include_bytes!("../fixtures/square.amvk");
    for index in 0..original.len() {
        for mask in [1, 0x80, 0xff] {
            let mut bytes = original.to_vec();
            bytes[index] ^= mask;
            let result = std::panic::catch_unwind(|| {
                let _ = rust_server::ir::lower_trace(&bytes);
                if let Ok(ops) = decode(&bytes) {
                    let _ = lower(&ops);
                }
            });
            assert!(result.is_ok(), "corruption panic at {index}");
        }
    }
}
#[test]
fn constructor_without_tokio_is_an_error() {
    assert!(Runtime::new(4).is_err());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_producers_are_bounded() {
    let runtime = std::sync::Arc::new(Runtime::new(8).unwrap());
    let mut producers = Vec::new();
    for producer in 0..8 {
        let runtime = runtime.clone();
        producers.push(tokio::spawn(async move {
            let bytes = include_bytes!("../fixtures/rectangular.amvk").to_vec();
            let plan = lower(&decode(&bytes).unwrap()).unwrap();
            let mut accepted = Vec::new();
            let mut rejected = 0;
            for request in 0..8 {
                match runtime.submit(Job {
                    trace: bytes.clone(),
                    inputs: seeded_inputs(&plan, producer * 8 + request),
                }) {
                    Ok(rx) => accepted.push(rx),
                    Err(_) => rejected += 1,
                }
            }
            let mut ids = Vec::new();
            for rx in accepted {
                let receipt = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
                    .await
                    .unwrap()
                    .unwrap();
                assert!(receipt.failure.is_none());
                ids.push(receipt.job_id);
            }
            (ids, rejected)
        }));
    }
    let mut ids = BTreeSet::new();
    let mut rejected = 0;
    for task in producers {
        let (accepted, n) = task.await.unwrap();
        rejected += n;
        for id in accepted {
            assert!(ids.insert(id));
        }
    }
    assert!(!ids.is_empty());
    assert!(rejected > 0);
    assert_eq!(ids.len() + rejected, 64);
    println!(
        "concurrent producers: {} accepted, {rejected} rejected, 64 total",
        ids.len()
    );
}

#[test]
fn tampered_plan_and_extreme_integer_cancellation() {
    let mut plan = shape(2, 2, 2);
    plan.m = 0;
    assert!(plan.msl().is_err());
    let mut plan = shape(2, 2, 2);
    plan.input_values[0] = 999;
    assert!(plan.msl().is_err());
    let plan = shape(1, 2, 1);
    let mut inputs = seeded_inputs(&plan, 1);
    inputs.insert("A".into(), vec![i32::MAX, i32::MAX]);
    inputs.insert("B".into(), vec![i32::MAX, i32::MIN]);
    inputs.insert("bias".into(), vec![0]);
    assert!(
        execute(&plan, &inputs, 1).is_ok(),
        "Full-range cancellation must be defined"
    );
    assert!(rust_server::execute::reference(&plan, &Default::default()).is_err());
}

#[test]
fn borrowed_lowering_matches_debug_projection() {
    for bytes in [
        include_bytes!("../fixtures/square.amvk").as_slice(),
        include_bytes!("../fixtures/rectangular.amvk").as_slice(),
    ] {
        let borrowed = rust_server::ir::lower_trace(bytes).unwrap();
        let projected = lower(&decode(bytes).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(borrowed).unwrap(),
            serde_json::to_value(projected).unwrap()
        );
    }
}
