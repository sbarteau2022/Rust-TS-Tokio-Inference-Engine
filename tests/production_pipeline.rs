use rust_server::{
    execute::{self, Inputs},
    ir,
    runtime::{ChainJob, Job, Runtime},
};
use std::time::Duration;
fn bytes(name: &str) -> Vec<u8> {
    std::fs::read(format!("fixtures/{name}.amvk")).unwrap()
}
fn expected(names: &[&str], external: &Inputs) -> Vec<i32> {
    let mut previous = None;
    for name in names {
        let plan = ir::lower_trace(&bytes(name)).unwrap();
        let mut inputs = Inputs::new();
        for (slot, name) in plan.inputs.iter().enumerate() {
            inputs.insert(
                name.clone(),
                if slot == 0 && previous.is_some() {
                    previous.take().unwrap()
                } else {
                    external[name].clone()
                },
            );
        }
        previous = Some(execute::reference(&plan, &inputs).unwrap());
    }
    previous.unwrap()
}
#[tokio::test]
async fn chains_are_exact_at_all_grids_and_both_caps() {
    for limit in [2, 3] {
        for width in [64, 128, 256, 512] {
            let runtime = Runtime::with_grid(8, limit, Some(width)).unwrap();
            for names in [
                vec!["rectangular", "sin", "cos", "tanh"],
                vec!["elementwise", "atan2", "sin"],
            ] {
                let plan = ir::lower_trace(&bytes(names[0])).unwrap();
                let mut inputs = execute::seeded_inputs(&plan, 42);
                if names.contains(&"atan2") {
                    inputs.insert(
                        "B".into(),
                        execute::seeded_inputs(&ir::lower_trace(&bytes("atan2")).unwrap(), 77)["B"]
                            .clone(),
                    );
                }
                let expected = expected(&names, &inputs);
                for _ in 0..4 {
                    let before = runtime.flight_stats().submitted;
                    let receipt = runtime
                        .submit_chain(ChainJob {
                            traces: names.iter().map(|name| bytes(name)).collect(),
                            inputs: inputs.clone(),
                        })
                        .unwrap()
                        .await
                        .unwrap();
                    assert!(receipt.failure.is_none(), "{:?}", receipt.failure);
                    let out = receipt.output.unwrap();
                    assert_eq!(out.values, expected);
                    assert_eq!(out.metrics.kernel_dispatches, names.len());
                    assert_eq!(
                        out.metrics.validation_performed,
                        cfg!(feature = "debug-validation")
                    );
                    if !cfg!(feature = "debug-validation") {
                        assert_eq!(out.metrics.validation_ms, 0.);
                    }
                    assert_eq!(runtime.flight_stats().submitted, before + 1);
                    assert_eq!(runtime.flight_stats().outstanding, 0);
                }
            }
        }
    }
}
#[tokio::test]
async fn maximum_chain_and_canceled_receipts_keep_resources_alive() {
    let runtime = Runtime::new(32).unwrap();
    let names = vec!["sin"; 16];
    let inputs = execute::seeded_inputs(&ir::lower_trace(&bytes("sin")).unwrap(), 5);
    let wanted = expected(&names, &inputs);
    for _ in 0..8 {
        drop(
            runtime
                .submit_chain(ChainJob {
                    traces: names.iter().map(|name| bytes(name)).collect(),
                    inputs: inputs.clone(),
                })
                .unwrap(),
        );
    }
    let out = runtime
        .submit_chain(ChainJob {
            traces: names.iter().map(|name| bytes(name)).collect(),
            inputs,
        })
        .unwrap()
        .await
        .unwrap();
    assert!(out.failure.is_none(), "{:?}", out.failure);
    assert_eq!(out.output.unwrap().values, wanted);
    tokio::time::timeout(Duration::from_secs(10), async {
        while runtime.flight_stats().completed != 9 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    let stats = runtime.flight_stats();
    assert_eq!(stats.active, 0);
    assert_eq!(stats.acquired, stats.released);
    assert_eq!(stats.submitted, 9);
}
#[tokio::test]
async fn rejects_chain_bounds_shapes_and_bindings_then_recovers() {
    let runtime = Runtime::new(8).unwrap();
    for traces in [vec![], vec![bytes("sin"); 17]] {
        assert!(runtime
            .submit_chain(ChainJob {
                traces,
                inputs: Inputs::new()
            })
            .is_err());
    }
    for names in [vec!["square", "sin"], vec!["sin", "atan2"]] {
        let inputs = execute::seeded_inputs(&ir::lower_trace(&bytes(names[0])).unwrap(), 42);
        let receipt = runtime
            .submit_chain(ChainJob {
                traces: names.iter().map(|name| bytes(name)).collect(),
                inputs,
            })
            .unwrap()
            .await
            .unwrap();
        assert_eq!(receipt.failure.unwrap().stage, "inputs");
    }
    assert_eq!(runtime.flight_stats().submitted, 0);
    let trace = bytes("elementwise");
    let inputs = execute::seeded_inputs(&ir::lower_trace(&trace).unwrap(), 42);
    assert!(runtime
        .submit(Job { trace, inputs })
        .unwrap()
        .await
        .unwrap()
        .failure
        .is_none());
}
#[tokio::test]
async fn samples_exactly_every_thousand_successful_receipts() {
    let runtime = Runtime::new(8).unwrap();
    assert_eq!(runtime.flight_stats().limit, 3);
    let trace = bytes("elementwise");
    let inputs = execute::seeded_inputs(&ir::lower_trace(&trace).unwrap(), 13);
    for i in 1..=2000 {
        let receipt = runtime
            .submit(Job {
                trace: trace.clone(),
                inputs: inputs.clone(),
            })
            .unwrap()
            .await
            .unwrap();
        assert!(receipt.failure.is_none());
        if i == 999 {
            assert_eq!(runtime.validation_stats().scheduled, 0);
        }
        if i == 1000 || i == 2000 {
            let stats = tokio::time::timeout(Duration::from_secs(10), runtime.flush_validation())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(stats.successful_receipts, i);
            assert_eq!(stats.scheduled, i / 1000);
            assert_eq!(stats.passed, i / 1000);
            assert_eq!(stats.failed, 0);
        }
    }
}

fn unary_trace(rows: u32, cols: u32) -> Vec<u8> {
    use rust_server::trace_generated::wire as w;
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let name = builder.create_string("A");
    let load = w::Load::create(
        &mut builder,
        &w::LoadArgs {
            name: Some(name),
            rows,
            cols,
            out: 0,
        },
    );
    let load = w::Op::create(
        &mut builder,
        &w::OpArgs {
            operation_type: w::Operation::Load,
            operation: Some(load.as_union_value()),
        },
    );
    let cordic = w::Cordic::create(
        &mut builder,
        &w::CordicArgs {
            function: 3,
            a: 0,
            b: 0,
            out: 1,
        },
    );
    let cordic = w::Op::create(
        &mut builder,
        &w::OpArgs {
            operation_type: w::Operation::Cordic,
            operation: Some(cordic.as_union_value()),
        },
    );
    let name = builder.create_string("C");
    let store = w::Store::create(
        &mut builder,
        &w::StoreArgs {
            a: 1,
            name: Some(name),
        },
    );
    let store = w::Op::create(
        &mut builder,
        &w::OpArgs {
            operation_type: w::Operation::Store,
            operation: Some(store.as_union_value()),
        },
    );
    let ops = builder.create_vector(&[load, cordic, store]);
    let trace = w::Trace::create(
        &mut builder,
        &w::TraceArgs {
            version: 3,
            ops: Some(ops),
        },
    );
    w::finish_trace_buffer(&mut builder, trace);
    builder.finished_data().to_vec()
}
#[tokio::test]
async fn aggregate_chain_residency_rejected_before_dispatch() {
    let runtime = Runtime::new(8).unwrap();
    let trace = unary_trace(4096, 256);
    let inputs = Inputs::from([("A".into(), vec![0; 4096 * 256])]);
    let receipt = runtime
        .submit_chain(ChainJob {
            traces: vec![trace; 16],
            inputs,
        })
        .unwrap()
        .await
        .unwrap();
    assert!(receipt.failure.unwrap().error.contains("UMA budget"));
    assert_eq!(runtime.flight_stats().submitted, 0);
    assert_eq!(runtime.flight_stats().active, 0);
}
#[tokio::test]
async fn sampled_chain_checks_all_intermediate_stages() {
    let runtime = Runtime::new(8).unwrap();
    let names = ["elementwise", "sin", "cos", "tanh"];
    let inputs = execute::seeded_inputs(&ir::lower_trace(&bytes(names[0])).unwrap(), 111);
    let wanted = expected(&names, &inputs);
    for _ in 0..1000 {
        let receipt = runtime
            .submit_chain(ChainJob {
                traces: names.iter().map(|name| bytes(name)).collect(),
                inputs: inputs.clone(),
            })
            .unwrap()
            .await
            .unwrap();
        assert!(receipt.failure.is_none());
        assert_eq!(receipt.output.unwrap().values, wanted);
    }
    let stats = tokio::time::timeout(Duration::from_secs(10), runtime.flush_validation())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stats.scheduled, 1);
    assert_eq!(stats.passed, 1);
    assert_eq!(runtime.flight_stats().submitted, 1000);
}

#[tokio::test]
async fn late_chain_encoding_error_submits_nothing_and_releases_permit() {
    use rust_server::{
        completion::FlightGate,
        gpu::{Dispatch, Gpu, Uma},
    };
    let gate = FlightGate::new(3).unwrap();
    let gpu = Gpu::with_gate(gate.clone()).unwrap();
    let pipeline=gpu.compile("#include <metal_stdlib>\nusing namespace metal;kernel void proof(device int* p [[buffer(0)]],uint i [[thread_position_in_grid]]){if(i==0)p[0]=42;}","proof").unwrap();
    let buffer = Uma::allocate(&gpu.device, 4, 64).unwrap();
    let stages = [
        Dispatch {
            pipeline: &pipeline,
            buffers: vec![&buffer],
            groups: metal::MTLSize::new(1, 1, 1),
            threads: 32,
        },
        Dispatch {
            pipeline: &pipeline,
            buffers: vec![&buffer],
            groups: metal::MTLSize::new(1, 1, 1),
            threads: 0,
        },
    ];
    assert!(gpu
        .submit_chain(&stages, gate.acquire().await.unwrap())
        .is_err());
    assert_eq!(gate.stats().submitted, 0);
    assert_eq!(gate.stats().active, 0);
}
