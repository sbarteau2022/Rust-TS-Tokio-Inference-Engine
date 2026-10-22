use rust_server::{
    backend::{Config, KernelBackend, MetalBackend},
    execute, ir,
    runtime::Runtime,
};
#[test]
fn all_widths_families_and_tails() {
    for width in [64, 128, 256, 512] {
        let mut backend = MetalBackend::new(Config {
            threadgroup_threads: Some(width),
            ..Config::default()
        })
        .unwrap();
        for directory in [
            "square",
            "rectangular",
            "elementwise",
            "sin",
            "cos",
            "atan2",
            "tanh",
        ] {
            let trace = std::fs::read(format!("fixtures/{directory}.amvk")).unwrap();
            let plan = ir::lower_trace(&trace).unwrap();
            let inputs = execute::seeded_inputs(&plan, 42);
            let (out, _, _) = backend.execute(&plan, &inputs, 2, None).unwrap();
            assert_eq!(out.values, execute::reference(&plan, &inputs).unwrap());
        }
    }
}
#[tokio::test]
async fn reject_invalid_width_before_workers() {
    for width in [0, 1, 32, 63, 65, 1024, u64::MAX] {
        assert!(Runtime::with_grid(32, 2, Some(width)).is_err());
        assert!(MetalBackend::new(Config {
            threadgroup_threads: Some(width),
            ..Config::default()
        })
        .is_err());
    }
}
