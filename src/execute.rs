use crate::ir::Plan;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub type Inputs = BTreeMap<String, Vec<i32>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub validation_performed: bool,
    pub kernel_dispatches: usize,
    pub compile_ms: f64,
    pub dispatch_wall_ms: f64,
    pub gpu_ms: f64,
    pub max_abs_error: Option<f64>,
    pub prepare_ms: f64,
    pub validation_ms: f64,
    pub execution_wall_ms: f64,
    pub pipeline_cache_hit: bool,
    pub resources: crate::backend::Stats,
}
#[derive(Debug, Serialize)]
pub struct Output {
    pub values: Vec<i32>,
    pub metrics: Metrics,
}
pub const FRACTIONAL_BITS: u32 = 31;
/// Independent wide integer judge: division floors, including negative products.
pub fn reference(plan: &Plan, inputs: &Inputs) -> Result<Vec<i32>> {
    validate_inputs(plan, inputs)?;
    let (m, k, n) = (plan.m as usize, plan.k as usize, plan.n as usize);
    let a = &inputs[&plan.inputs[0]];
    if plan.kind.is_cordic() {
        return Ok(a
            .iter()
            .enumerate()
            .map(|(i, x)| {
                crate::cordic::evaluate(
                    plan.kind,
                    *x,
                    if plan.kind == crate::ir::KernelKind::Atan2 {
                        inputs[&plan.inputs[1]][i]
                    } else {
                        0
                    },
                )
            })
            .collect());
    }
    let bias = &inputs[plan.inputs.last().unwrap()];
    let mut out = vec![0; m * n];
    for i in 0..m {
        for j in 0..n {
            let sum = if plan.kind == crate::ir::KernelKind::AddRelu {
                i128::from(a[i * n + j])
            } else {
                let b = &inputs[&plan.inputs[1]];
                (0..k)
                    .map(|x| {
                        (i128::from(a[i * k + x]) * i128::from(b[x * n + j]))
                            .div_euclid(1i128 << FRACTIONAL_BITS)
                    })
                    .sum::<i128>()
            };
            out[i * n + j] = (sum + i128::from(bias[j])).clamp(0, i128::from(i32::MAX)) as i32;
        }
    }
    Ok(out)
}
pub fn validate_inputs(plan: &Plan, inputs: &Inputs) -> Result<()> {
    plan.validate()?;
    ensure!(
        inputs.len() == plan.input_count(),
        "input binding count mismatch"
    );
    for (name, len) in plan.inputs.iter().zip(plan.input_lengths()) {
        let x = inputs
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("missing input {name}"))?;
        ensure!(x.len() == len, "wrong length for {name}");
    }
    Ok(())
}
pub fn compare(actual: &[i32], expected: &[i32]) -> Result<f64> {
    ensure!(actual.len() == expected.len(), "result length mismatch");
    for (i, (&a, &b)) in actual.iter().zip(expected).enumerate() {
        ensure!(a == b, "integer mismatch at {i}: {a} vs {b}");
    }
    Ok(0.)
}
pub fn execute(
    plan: &Plan,
    inputs: &Inputs,
    repeats: usize,
) -> Result<(Output, Vec<crate::gpu::Timing>, String)> {
    execute_capture(plan, inputs, repeats, None)
}
pub fn execute_capture(
    plan: &Plan,
    inputs: &Inputs,
    repeats: usize,
    capture: Option<&std::path::Path>,
) -> Result<crate::backend::Execution> {
    use crate::backend::KernelBackend;
    crate::backend::MetalBackend::new(crate::backend::Config::default())?
        .execute(plan, inputs, repeats, capture)
}
pub fn seeded_inputs(plan: &Plan, seed: u32) -> Inputs {
    let lengths = plan.input_lengths();
    let mut state = seed;
    plan.inputs
        .iter()
        .zip(lengths)
        .map(|(name, len)| {
            let values = (0..len)
                .map(|_| {
                    state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                    state as i32
                })
                .collect();
            (name.clone(), values)
        })
        .collect()
}
