use rust_server::{
    backend::{Config, KernelBackend, MetalBackend},
    execute::{self, Inputs},
    ir::{self, KernelKind},
    runtime::{Job, Runtime},
    wire::{self, Op},
};
const SCALE: f64 = 2147483648.;
fn trace(kind: KernelKind) -> &'static [u8] {
    match kind {
        KernelKind::Sin => include_bytes!("../fixtures/sin.amvk"),
        KernelKind::Cos => include_bytes!("../fixtures/cos.amvk"),
        KernelKind::Atan2 => include_bytes!("../fixtures/atan2.amvk"),
        KernelKind::Tanh => include_bytes!("../fixtures/tanh.amvk"),
        _ => unreachable!(),
    }
}
fn cases(plan: &ir::Plan) -> Inputs {
    let mut inputs = execute::seeded_inputs(plan, 327);
    let edges = [
        i32::MIN,
        i32::MIN + 1,
        -1073741824,
        -1073741823,
        -65536,
        -2,
        -1,
        0,
        1,
        2,
        65536,
        1073741823,
        1073741824,
        i32::MAX - 1,
        i32::MAX,
    ];
    for (index, values) in inputs.values_mut().enumerate() {
        for (i, value) in values
            .iter_mut()
            .take(edges.len() * edges.len())
            .enumerate()
        {
            *value = if index == 0 {
                edges[i % edges.len()]
            } else {
                edges[i / edges.len()]
            };
        }
    }
    inputs
}
fn accuracy(kind: KernelKind, inputs: &Inputs, values: &[i32]) -> f64 {
    let mut max_error = 0f64;
    for (i, raw) in values.iter().enumerate() {
        let a = f64::from(inputs["A"][i]) / SCALE;
        let reference = match kind {
            KernelKind::Sin => (a * std::f64::consts::PI).sin(),
            KernelKind::Cos => (a * std::f64::consts::PI).cos(),
            KernelKind::Tanh => a.tanh(),
            KernelKind::Atan2 => {
                let b = f64::from(inputs["B"][i]) / SCALE;
                if a == 0. && b == 0. {
                    0.
                } else {
                    a.atan2(b) / std::f64::consts::PI
                }
            }
            _ => unreachable!(),
        };
        let error = (f64::from(*raw) / SCALE - reference).abs();
        let error = if kind == KernelKind::Atan2 {
            error.min((2. - error).abs())
        } else {
            error
        };
        max_error = max_error.max(error * SCALE);
        assert!(
            error * SCALE <= 2.01,
            "{kind:?} at {i}: raw={raw}, reference={reference}, error raw={}",
            error * SCALE
        );
    }
    max_error
}
#[test]
fn all_cordic_gpu_outputs_exact_and_accurate() {
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    for kind in [
        KernelKind::Sin,
        KernelKind::Cos,
        KernelKind::Atan2,
        KernelKind::Tanh,
    ] {
        let plan = ir::lower_trace(trace(kind)).unwrap();
        let inputs = cases(&plan);
        let expected = execute::reference(&plan, &inputs).unwrap();
        let first = backend.execute(&plan, &inputs, 2, None).unwrap().0;
        let second = backend.execute(&plan, &inputs, 1, None).unwrap().0;
        assert_eq!(first.values, expected);
        assert_eq!(first.values, second.values);
        let error = accuracy(kind, &inputs, &first.values);
        println!(
            "{kind:?}: {} exact GPU outputs; max libm error {error} raw units",
            first.values.len()
        );
        assert!(second.metrics.pipeline_cache_hit);
        let source = plan.msl().unwrap();
        for token in source.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            assert!(
                !matches!(
                    token,
                    "float" | "double" | "half" | "sin" | "cos" | "tanh" | "atan2"
                ),
                "forbidden MSL token: {token}"
            );
        }
    }
    assert_eq!(backend.stats().pipeline_misses, 4);
    assert_eq!(backend.stats().pipeline_hits, 4);
}
#[test]
fn quadrant_axes_tiny_vectors_and_tanh_monotonicity() {
    use rust_server::cordic::evaluate as e;
    assert_eq!(e(KernelKind::Sin, 0, 0), 0);
    assert_eq!(e(KernelKind::Cos, 0, 0), i32::MAX);
    assert_eq!(e(KernelKind::Sin, 1 << 30, 0), i32::MAX);
    assert_eq!(e(KernelKind::Sin, -(1 << 30), 0), i32::MIN);
    assert_eq!(e(KernelKind::Cos, i32::MIN, 0), i32::MIN);
    assert_eq!(e(KernelKind::Atan2, 0, 0), 0);
    assert_eq!(e(KernelKind::Atan2, 0, -1), i32::MIN);
    assert_eq!(e(KernelKind::Atan2, 1, 0), 1 << 30);
    assert_eq!(e(KernelKind::Atan2, -1, 0), -(1 << 30));
    for y in -16..=16 {
        for x in -16..=16 {
            let inputs = Inputs::from([("A".into(), vec![y]), ("B".into(), vec![x])]);
            accuracy(KernelKind::Atan2, &inputs, &[e(KernelKind::Atan2, y, x)]);
        }
    }
    let mut previous = i32::MIN;
    for i in 0..=8192 {
        let raw = (i64::from(i32::MIN) + i64::from(i) * ((1i64 << 32) - 1) / 8192) as i32;
        let value = e(KernelKind::Tanh, raw, 0);
        assert!(value >= previous);
        previous = value;
    }
}
#[test]
fn cordic_graph_mutations_fail_closed() {
    for kind in [
        KernelKind::Sin,
        KernelKind::Cos,
        KernelKind::Atan2,
        KernelKind::Tanh,
    ] {
        let bytes = trace(kind);
        let ops = wire::decode(bytes).unwrap();
        assert_eq!(
            serde_json::to_value(ir::lower(&ops).unwrap()).unwrap(),
            serde_json::to_value(ir::lower_trace(bytes).unwrap()).unwrap()
        );
        let position = ops.len() - 2;
        for function in [4, 255] {
            let mut invalid = ops.clone();
            if let Op::Cordic { function: f, .. } = &mut invalid[position] {
                *f = function;
            }
            assert!(ir::lower(&invalid).is_err());
        }
        for bad in [0, 999] {
            let mut invalid = ops.clone();
            if let Op::Cordic { out, .. } = &mut invalid[position] {
                *out = bad;
            }
            assert!(ir::lower(&invalid).is_err());
        }
        for index in 0..bytes.len() {
            let mut changed = bytes.to_vec();
            changed[index] ^= 255;
            assert!(std::panic::catch_unwind(|| ir::lower_trace(&changed)).is_ok());
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_cordic_receipts_and_failure_recovery() {
    let runtime = Runtime::new(8).unwrap();
    let mut receivers = Vec::new();
    for kind in [
        KernelKind::Sin,
        KernelKind::Cos,
        KernelKind::Atan2,
        KernelKind::Tanh,
    ] {
        let p = ir::lower_trace(trace(kind)).unwrap();
        receivers.push((
            kind,
            runtime
                .submit(Job {
                    trace: trace(kind).to_vec(),
                    inputs: cases(&p),
                })
                .unwrap(),
        ));
    }
    for (kind, receiver) in receivers {
        let result = receiver.await.unwrap();
        assert!(result.failure.is_none(), "{:?}", result.failure);
        assert_eq!(result.kernel, kind.entry());
    }
    let p = ir::lower_trace(trace(KernelKind::Sin)).unwrap();
    let bad = runtime
        .submit(Job {
            trace: trace(KernelKind::Sin).to_vec(),
            inputs: Inputs::new(),
        })
        .unwrap()
        .await
        .unwrap();
    assert_eq!(bad.failure.unwrap().stage, "inputs");
    let good = runtime
        .submit(Job {
            trace: trace(KernelKind::Sin).to_vec(),
            inputs: cases(&p),
        })
        .unwrap()
        .await
        .unwrap();
    assert!(good.failure.is_none());
}
#[test]
fn cordic_dynamic_shapes_tails_and_pool_reuse() {
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    for kind in [
        KernelKind::Sin,
        KernelKind::Cos,
        KernelKind::Atan2,
        KernelKind::Tanh,
    ] {
        for (m, n) in [
            (1, 1),
            (1, 31),
            (1, 32),
            (1, 33),
            (1, 255),
            (1, 256),
            (1, 257),
            (7, 13),
        ] {
            let mut ops = wire::decode(trace(kind)).unwrap();
            for op in &mut ops {
                if let Op::Load { rows, cols, .. } = op {
                    *rows = m;
                    *cols = n;
                }
            }
            let p = ir::lower(&ops).unwrap();
            let inputs = cases(&p);
            let first = backend.execute(&p, &inputs, 1, None).unwrap().0;
            let second = backend.execute(&p, &inputs, 1, None).unwrap().0;
            assert_eq!(first.values, second.values);
            assert!(second.metrics.pipeline_cache_hit);
            accuracy(kind, &inputs, &first.values);
        }
    }
}
