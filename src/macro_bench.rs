//! Fixed request-count windows measured at the public Rust admission/receipt boundary.
use crate::{
    execute, ir,
    runtime::{Job, Runtime},
};
use anyhow::{ensure, Result};
use std::{collections::VecDeque, time::Instant};

pub async fn run(directory: &str, threads: u64, limit: usize, count: usize) -> Result<()> {
    ensure!([1000, 1500, 2000].contains(&count), "unsupported window");
    let trace = std::fs::read(format!("{directory}/trace.amvk"))?;
    let plan = ir::lower_trace(&trace)?;
    let inputs = execute::seeded_inputs(&plan, 42);
    let runtime = Runtime::with_grid(32, limit, Some(threads))?;
    let mut elapsed = 0.;
    let mut wall = Vec::new();
    let mut gpu = Vec::new();
    let mut validation = Vec::new();
    for (warm, total) in [(true, 32), (false, count)] {
        let start = Instant::now();
        let mut pending = VecDeque::new();
        for submitted in 0..total {
            pending.push_back(runtime.submit(Job {
                trace: trace.clone(),
                inputs: inputs.clone(),
            })?);
            if pending.len() == 32 || submitted + 1 == total {
                let drain = if submitted + 1 == total {
                    pending.len()
                } else {
                    1
                };
                for _ in 0..drain {
                    let receipt = pending.pop_front().unwrap().await?;
                    ensure!(
                        receipt.failure.is_none(),
                        "receipt failure: {:?}",
                        receipt.failure
                    );
                    let out = receipt
                        .output
                        .ok_or_else(|| anyhow::anyhow!("missing output"))?;
                    ensure!(
                        out.metrics.max_abs_error.is_none_or(|error| error == 0.),
                        "integer mismatch"
                    );
                    if !warm {
                        ensure!(out.metrics.pipeline_cache_hit, "cold timed pipeline");
                        wall.push(receipt.wall_ms);
                        gpu.push(out.metrics.gpu_ms);
                        validation.push(out.metrics.validation_ms);
                    }
                }
            }
        }
        if !warm {
            elapsed = start.elapsed().as_secs_f64();
        }
    }
    let background = runtime.flush_validation().await?;
    let flight = runtime.flight_stats();
    ensure!(
        flight.active == 0
            && flight.outstanding == 0
            && flight.acquired == flight.released
            && flight.submitted == flight.completed
            && flight.completed == (count + 32)
            && flight.completion_errors == 0,
        "unbalanced completion counters"
    );
    let rate = count as f64 / elapsed;
    println!(
        "{}",
        serde_json::json!({"run_label":"BENCH-MODE","sampling_policy":"DETERMINISTIC-MODE","production_sampling_claims":false,"threads":threads,"limit":limit,"count":count,"shape":[plan.m,plan.k,plan.n],"seconds":elapsed,"requests_per_second":rate,"integer_mac_per_second":rate*f64::from(plan.m)*f64::from(plan.k)*f64::from(plan.n),"receipt_p50_ms":percentile(&mut wall,0.5),"receipt_p95_ms":percentile(&mut wall,0.95),"receipt_p99_ms":percentile(&mut wall,0.99),"gpu_p50_ms":percentile(&mut gpu,0.5),"validation_p50_ms":percentile(&mut validation,0.5),"flight":flight,"background_validation":background,"full_host_validation":cfg!(feature="debug-validation"),"sample_interval":1000,"pass":true})
    );
    Ok(())
}
fn percentile(values: &mut [f64], fraction: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((values.len() - 1) as f64 * fraction).ceil() as usize]
}
