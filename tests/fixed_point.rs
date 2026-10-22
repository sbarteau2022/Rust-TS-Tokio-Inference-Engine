use rust_server::{
    backend::{Config, KernelBackend, MetalBackend},
    execute::{self, Inputs},
    ir::{self, Plan},
    wire::Op,
};
fn gemm(k: u32) -> Plan {
    ir::lower(&[
        Op::Load {
            name: "A".into(),
            rows: 1,
            cols: k,
            out: 0,
        },
        Op::Load {
            name: "B".into(),
            rows: k,
            cols: 1,
            out: 1,
        },
        Op::Load {
            name: "bias".into(),
            rows: 1,
            cols: 1,
            out: 2,
        },
        Op::Dot { a: 0, b: 1, out: 3 },
        Op::Add { a: 3, b: 2, out: 4 },
        Op::Relu { a: 4, out: 5 },
        Op::Store {
            a: 5,
            name: "C".into(),
        },
    ])
    .unwrap()
}
fn run(backend: &mut MetalBackend, a: Vec<i32>, b: Vec<i32>, bias: i32, expected: i32) {
    let p = gemm(a.len() as u32);
    let inputs = Inputs::from([
        ("A".into(), a),
        ("B".into(), b),
        ("bias".into(), vec![bias]),
    ]);
    assert_eq!(execute::reference(&p, &inputs).unwrap(), vec![expected]);
    let out = backend.execute(&p, &inputs, 3, None).unwrap().0;
    assert_eq!(out.values, vec![expected]);
    assert_eq!(
        out.metrics.max_abs_error,
        cfg!(feature = "debug-validation").then_some(0.)
    );
}
#[test]
fn explicit_widening_floor_shift_and_final_saturation() {
    let mut b = MetalBackend::new(Config::default()).unwrap();
    // Negative fractions floor rather than truncate; multiplication widens first.
    run(&mut b, vec![-1], vec![1], 2, 1);
    run(&mut b, vec![i32::MIN], vec![1], 3, 2);
    run(&mut b, vec![65535], vec![1], 2, 2);
    run(&mut b, vec![1 << 30], vec![1 << 30], 0, 1 << 29);
    run(&mut b, vec![i32::MIN], vec![i32::MIN], 0, i32::MAX);
    run(&mut b, vec![i32::MAX], vec![i32::MAX], i32::MAX, i32::MAX);
    run(&mut b, vec![i32::MIN], vec![i32::MAX], i32::MAX, 0);
    // Shift each product; a single shift after dot would yield 1 here.
    run(&mut b, vec![1, 1], vec![1 << 30, 1 << 30], 0, 0);
    // Large signed partials cross lane boundaries and 32-bit carries; final stays small.
    run(
        &mut b,
        vec![i32::MAX; 64],
        (0..64)
            .map(|i| if i % 2 == 0 { i32::MAX } else { -i32::MAX })
            .collect(),
        100,
        68,
    );
    // Full-range cancellation must occur before output saturation.
    run(&mut b, vec![i32::MAX; 2], vec![i32::MAX, -i32::MAX], 0, 0);
    run(
        &mut b,
        vec![i32::MIN; 4096],
        vec![i32::MIN; 4096],
        0,
        i32::MAX,
    );
    run(&mut b, vec![i32::MIN; 4096], vec![i32::MAX; 4096], 0, 0);
}
#[test]
fn integer_only_emission_and_truthful_contract() {
    let p = gemm(9);
    let source = p.msl().unwrap();
    assert!(source.contains("int64_t product=int64_t(a)*int64_t(b)"));
    assert!(source.contains("int64_t acc=0"));
    assert!(source.contains("device const int32_t*"));
    assert!(source.contains("simd_shuffle_down"));
    assert!(!source.contains("threadgroup int32_t"));
    assert!(!source.contains("threadgroup_barrier"));
    for forbidden in [
        "float",
        "half",
        "double",
        "simdgroup_multiply",
        "as_type<int32_t>",
    ] {
        assert!(!source.contains(forbidden), "{forbidden}");
    }
    let caps = MetalBackend::new(Config::default()).unwrap().capabilities();
    assert_eq!(caps.dtypes, ["q1_31_i32"]);
}
#[test]
fn full_range_randomized_gpu_parity_and_repeatability() {
    let mut backend = MetalBackend::new(Config::default()).unwrap();
    let mut state = 91u32;
    for k in [1, 7, 8, 9, 29, 31, 32, 33, 63, 64, 65, 257, 4096] {
        let p = gemm(k);
        for _ in 0..16 {
            let mut inputs = execute::seeded_inputs(&p, state);
            for values in inputs.values_mut() {
                for value in values {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    *value = state as i32;
                }
            }
            let expected = execute::reference(&p, &inputs).unwrap();
            let first = backend.execute(&p, &inputs, 1, None).unwrap().0;
            let second = backend.execute(&p, &inputs, 1, None).unwrap().0;
            assert_eq!(first.values, expected);
            assert_eq!(first.values, second.values);
        }
    }
}
#[test]
fn widened_elementwise_add_and_negative_relu() {
    let p = ir::lower_trace(include_bytes!("../fixtures/elementwise.amvk")).unwrap();
    let mut inputs = execute::seeded_inputs(&p, 1);
    for (i, value) in inputs.get_mut("A").unwrap().iter_mut().enumerate() {
        *value = if i % 2 == 0 { i32::MAX } else { i32::MIN };
    }
    inputs.get_mut("bias").unwrap().fill(i32::MAX);
    let output = MetalBackend::new(Config::default())
        .unwrap()
        .execute(&p, &inputs, 1, None)
        .unwrap()
        .0;
    for (i, x) in output.values.iter().enumerate() {
        assert_eq!(*x, if i % 2 == 0 { i32::MAX } else { 0 });
    }
}
