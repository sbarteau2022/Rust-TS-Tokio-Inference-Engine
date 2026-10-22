use rust_server::{
    backend::{Config, KernelBackend, MetalBackend},
    execute::{compare, seeded_inputs},
    ir::{self, KernelKind, Plan},
    runtime::{Job, Runtime},
    wire::{decode, Op},
};
fn matrix(m: u32, k: u32, n: u32) -> Plan {
    let mut ops = decode(include_bytes!("../fixtures/square.amvk")).unwrap();
    ops[0] = Op::Load {
        name: "A".into(),
        rows: m,
        cols: k,
        out: 0,
    };
    ops[1] = Op::Load {
        name: "B".into(),
        rows: k,
        cols: n,
        out: 1,
    };
    ops[2] = Op::Load {
        name: "bias".into(),
        rows: 1,
        cols: n,
        out: 2,
    };
    ir::lower(&ops).unwrap()
}
fn elementwise(m: u32, n: u32) -> Plan {
    let ops = vec![
        Op::Load {
            name: "A".into(),
            rows: m,
            cols: n,
            out: 0,
        },
        Op::Load {
            name: "bias".into(),
            rows: 1,
            cols: n,
            out: 1,
        },
        Op::Add { a: 0, b: 1, out: 2 },
        Op::Relu { a: 2, out: 3 },
        Op::Store {
            a: 3,
            name: "C".into(),
        },
    ];
    ir::lower(&ops).unwrap()
}
#[test]
fn warmed_jobs_reuse_pipeline_and_every_buffer() {
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    let plan = matrix(73, 29, 91);
    let mut previous = None;
    for seed in 0..10 {
        let inputs = seeded_inputs(&plan, seed);
        let (out, _, _) = backend.execute(&plan, &inputs, 1, None).unwrap();
        assert_eq!(out.metrics.pipeline_cache_hit, seed != 0);
        if seed != 0 {
            assert_eq!(out.metrics.compile_ms, 0.);
        }
        if let Some(values) = previous {
            assert_ne!(out.values, values);
        }
        previous = Some(out.values);
    }
    let stats = backend.stats();
    assert_eq!(stats.pipeline_misses, 1);
    assert_eq!(stats.pipeline_hits, 9);
    assert_eq!(stats.buffer_allocations, 4);
    assert_eq!(stats.buffer_reuses, 36);
    assert!(stats.peak_resident_bytes <= Config::default().resident_bytes);
}
#[test]
fn lru_and_kernel_family_separation() {
    let mut backend = MetalBackend::new(Config {
        pipeline_entries: 2,
        ..Config::default()
    })
    .unwrap();
    let (a, b, c) = (matrix(8, 8, 8), matrix(9, 8, 8), elementwise(8, 8));
    for plan in [&a, &b, &a, &c, &b] {
        backend
            .execute(plan, &seeded_inputs(plan, 7), 1, None)
            .unwrap();
    }
    let stats = backend.stats();
    assert_eq!(stats.pipeline_hits, 1);
    assert_eq!(stats.pipeline_misses, 4);
    assert_eq!(stats.pipeline_evictions, 2);
    assert_eq!(stats.pipeline_entries, 2);
    let mut renamed = a.clone();
    renamed.inputs = vec!["Q".into(), "W".into(), "Z".into()];
    renamed.output = "Result".into();
    backend
        .execute(&renamed, &seeded_inputs(&renamed, 9), 1, None)
        .unwrap();
    // Renamed bindings have identical canonical shader semantics but a was evicted.
    assert_eq!(backend.stats().pipeline_misses, 5);
    backend
        .execute(&a, &seeded_inputs(&a, 10), 1, None)
        .unwrap();
    assert_eq!(backend.stats().pipeline_hits, 2);
}
#[test]
fn hard_memory_bound_dynamic_shapes_and_recovery() {
    let gpu = rust_server::gpu::Gpu::new().unwrap();
    let budget = 4 * rust_server::gpu::Uma::allocation_size(&gpu.device, 512).unwrap();
    let config = Config {
        resident_bytes: budget,
        idle_bytes: budget,
        ..Config::default()
    };
    let mut backend = MetalBackend::new(config).unwrap();
    for plan in [
        matrix(64, 64, 64),
        matrix(63, 64, 63),
        elementwise(40, 100),
        matrix(64, 64, 64),
    ] {
        backend
            .execute(&plan, &seeded_inputs(&plan, 13), 1, None)
            .unwrap();
        let stats = backend.stats();
        assert!(stats.idle_bytes <= budget);
        assert!(stats.peak_resident_bytes <= budget);
    }
    assert!(backend.stats().buffer_evictions > 0);
    let too_big = matrix(128, 128, 128);
    assert!(backend
        .execute(&too_big, &seeded_inputs(&too_big, 3), 1, None)
        .is_err());
    let small = elementwise(1, 1);
    backend
        .execute(&small, &seeded_inputs(&small, 9), 1, None)
        .unwrap();
}
#[test]
fn zero_retention_and_failed_jobs_never_leak_pool_accounting() {
    let mut backend = MetalBackend::new(Config {
        idle_bytes: 0,
        ..Config::default()
    })
    .unwrap();
    let p = matrix(8, 8, 8);
    for seed in 0..2 {
        backend
            .execute(&p, &seeded_inputs(&p, seed), 1, None)
            .unwrap();
        assert_eq!(backend.stats().idle_bytes, 0);
    }
    assert_eq!(backend.stats().buffer_allocations, 8);
    assert_eq!(backend.stats().buffer_evictions, 8);
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    assert!(backend
        .execute(
            &p,
            &seeded_inputs(&p, 0),
            1,
            Some(std::path::Path::new("relative.gputrace"))
        )
        .is_err());
    assert_eq!(backend.stats().buffer_allocations, 4);
    backend.execute(&p, &seeded_inputs(&p, 1), 1, None).unwrap();
    assert_eq!(backend.stats().buffer_allocations, 4);
    assert_eq!(backend.stats().buffer_reuses, 4);
    let p = matrix(1, 2, 1);
    let mut inputs = seeded_inputs(&p, 1);
    inputs.insert("A".into(), vec![i32::MAX; 2]);
    inputs.insert("B".into(), vec![i32::MAX, i32::MIN]);
    assert!(backend.execute(&p, &inputs, 1, None).is_ok());
    backend.execute(&p, &seeded_inputs(&p, 2), 1, None).unwrap();
}
#[test]
fn second_family_tails_extremes_and_no_stale_output() {
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    for (m, n) in [(1, 1), (1, 257), (7, 13), (73, 91), (128, 128)] {
        let p = elementwise(m, n);
        let inputs = seeded_inputs(&p, 31);
        let out = backend.execute(&p, &inputs, 1, None).unwrap().0;
        assert_eq!(out.values.len(), (m * n) as usize);
        compare(
            &out.values,
            &rust_server::execute::reference(&p, &inputs).unwrap(),
        )
        .unwrap();
        let mut zeros = seeded_inputs(&p, 0);
        zeros.get_mut("A").unwrap().fill(0);
        zeros.get_mut("bias").unwrap().fill(-1);
        assert!(backend
            .execute(&p, &zeros, 1, None)
            .unwrap()
            .0
            .values
            .iter()
            .all(|x| *x == 0));
        let mut overflow = seeded_inputs(&p, 0);
        overflow.get_mut("A").unwrap().fill(i32::MAX);
        overflow.get_mut("bias").unwrap().fill(i32::MAX);
        assert!(backend.execute(&p, &overflow, 1, None).is_ok());
        backend.execute(&p, &inputs, 1, None).unwrap();
    }
}
#[test]
fn elementwise_wire_validation_and_capabilities() {
    let bytes = include_bytes!("../fixtures/elementwise.amvk");
    let p = ir::lower_trace(bytes).unwrap();
    assert_eq!(p.kind, KernelKind::AddRelu);
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(ir::lower(&decode(bytes).unwrap()).unwrap()).unwrap()
    );
    for index in 0..bytes.len() {
        let mut bad = bytes.to_vec();
        bad[index] ^= 0xff;
        assert!(std::panic::catch_unwind(|| ir::lower_trace(&bad)).is_ok());
    }
    let mut ops = decode(bytes).unwrap();
    ops[2] = Op::Add { a: 1, b: 0, out: 2 };
    assert!(ir::lower(&ops).is_err());
    let mut ops = decode(bytes).unwrap();
    ops[1] = Op::Load {
        name: "bias".into(),
        rows: 2,
        cols: 91,
        out: 1,
    };
    assert!(ir::lower(&ops).is_err());
    let caps = MetalBackend::new(Config::default()).unwrap().capabilities();
    assert!(caps.unified_memory);
    assert_eq!(caps.kernels.len(), 6);
    assert!(!caps.amx_execution && !caps.ane_execution);
}
#[test]
fn invalid_configs_are_refused_before_allocating() {
    for config in [
        Config {
            pipeline_entries: 0,
            ..Config::default()
        },
        Config {
            pipeline_entries: 65,
            ..Config::default()
        },
        Config {
            pipeline_source_bytes: 4095,
            ..Config::default()
        },
        Config {
            resident_bytes: 0,
            idle_bytes: 0,
            ..Config::default()
        },
        Config {
            resident_bytes: 1024,
            idle_bytes: 1025,
            ..Config::default()
        },
    ] {
        assert!(MetalBackend::new(config).is_err());
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn runtime_routes_both_families_and_retains_resources() {
    let runtime = Runtime::new(4).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let mut hits = 0;
    for bytes in [
        include_bytes!("../fixtures/square.amvk").as_slice(),
        include_bytes!("../fixtures/elementwise.amvk").as_slice(),
        include_bytes!("../fixtures/square.amvk").as_slice(),
        include_bytes!("../fixtures/elementwise.amvk").as_slice(),
        include_bytes!("../fixtures/square.amvk").as_slice(),
        include_bytes!("../fixtures/elementwise.amvk").as_slice(),
        include_bytes!("../fixtures/square.amvk").as_slice(),
        include_bytes!("../fixtures/elementwise.amvk").as_slice(),
    ] {
        let plan = ir::lower_trace(bytes).unwrap();
        let receipt = runtime
            .submit(Job {
                trace: bytes.to_vec(),
                inputs: seeded_inputs(&plan, 4),
            })
            .unwrap()
            .await
            .unwrap();
        assert!(receipt.failure.is_none());
        assert_eq!(receipt.kernel, plan.kind.entry());
        let metrics = receipt.output.unwrap().metrics;
        let expected_hit = !seen.insert((receipt.worker_id, plan.kind.entry()));
        assert_eq!(metrics.pipeline_cache_hit, expected_hit);
        hits += usize::from(metrics.pipeline_cache_hit);
    }
    assert!(
        hits >= 2,
        "three lanes can have at most six distinct family cache misses"
    );
}
#[test]
fn pipeline_source_bytes_are_bounded_independently_of_entry_count() {
    let mut backend = MetalBackend::new(Config {
        pipeline_entries: 64,
        pipeline_source_bytes: 4096,
        ..Config::default()
    })
    .unwrap();
    for m in 1..10 {
        let plan = matrix(m, 8, 8);
        backend
            .execute(&plan, &seeded_inputs(&plan, m), 1, None)
            .unwrap();
        assert!(backend.stats().pipeline_source_bytes <= 4096);
    }
    assert!(backend.stats().pipeline_evictions > 0);
    assert!(backend.stats().pipeline_entries < 9);
}
#[test]
fn page_granularity_is_reserved_before_admission() {
    let gpu = rust_server::gpu::Gpu::new().unwrap();
    let unit = rust_server::gpu::Uma::allocation_size(&gpu.device, 4).unwrap();
    let mut backend = MetalBackend::new(Config {
        resident_bytes: unit * 3,
        idle_bytes: unit * 3,
        ..Config::default()
    })
    .unwrap();
    let p = matrix(1, 1, 1);
    assert!(backend.execute(&p, &seeded_inputs(&p, 1), 1, None).is_err());
    assert_eq!(backend.stats().buffer_allocations, 0);
    let p = elementwise(1, 1);
    backend.execute(&p, &seeded_inputs(&p, 2), 1, None).unwrap();
    assert!(backend.stats().peak_resident_bytes <= unit * 3);
    for len in [4, 256, 512, 777, 16384, 16388, 65536] {
        let reservation = rust_server::gpu::Uma::allocation_size(&gpu.device, len).unwrap();
        let b = rust_server::gpu::Uma::allocate(&gpu.device, len, 64).unwrap();
        assert!(b.allocated_size() <= reservation);
    }
}
