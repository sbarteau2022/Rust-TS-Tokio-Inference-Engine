//! Worker-owned Metal resources. No pooled buffer is shared across active jobs.
use crate::{
    execute::{self, Inputs, Metrics, Output},
    gpu::{Capture, Gpu, Timing, Uma},
    ir::{KernelKind, Plan},
};
use anyhow::{ensure, Result};
use metal::ComputePipelineState;
use serde::Serialize;
use std::{collections::VecDeque, path::Path, time::Instant};

pub type Execution = (Output, Vec<Timing>, String);
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub threadgroup_threads: Option<u64>,
    pub pipeline_entries: usize,
    pub pipeline_source_bytes: usize,
    pub resident_bytes: usize,
    pub idle_bytes: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            threadgroup_threads: None,
            pipeline_entries: 8,
            pipeline_source_bytes: 1024 * 1024,
            resident_bytes: 64 * 1024 * 1024,
            idle_bytes: 16 * 1024 * 1024,
        }
    }
}
impl Config {
    fn validate(self) -> Result<Self> {
        ensure!(
            self.threadgroup_threads
                .is_none_or(|n| [64, 128, 256, 512].contains(&n)),
            "unsupported threadgroup width"
        );
        ensure!(
            (1..=64).contains(&self.pipeline_entries),
            "pipeline entry bound"
        );
        ensure!(
            (4096..=4 * 1024 * 1024).contains(&self.pipeline_source_bytes),
            "pipeline source bound"
        );
        ensure!(
            self.resident_bytes > 0
                && self.resident_bytes <= 64 * 1024 * 1024
                && self.idle_bytes <= self.resident_bytes,
            "UMA budget configuration"
        );
        Ok(self)
    }
}
#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
pub struct Stats {
    pub pipeline_hits: u64,
    pub pipeline_misses: u64,
    pub pipeline_evictions: u64,
    pub pipeline_entries: usize,
    pub pipeline_source_bytes: usize,
    pub buffer_allocations: u64,
    pub buffer_reuses: u64,
    pub buffer_evictions: u64,
    pub idle_bytes: usize,
    pub peak_resident_bytes: usize,
    pub device_current_allocated_bytes: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct Capabilities {
    pub backend: &'static str,
    pub device: String,
    pub device_registry_id: u64,
    pub unified_memory: bool,
    pub kernels: Vec<KernelKind>,
    pub dtypes: Vec<&'static str>,
    pub layouts: Vec<&'static str>,
    pub amx_execution: bool,
    pub ane_execution: bool,
}
pub trait KernelBackend: Send {
    fn capabilities(&self) -> Capabilities;
    fn execute(
        &mut self,
        plan: &Plan,
        inputs: &Inputs,
        repeats: usize,
        capture: Option<&Path>,
    ) -> Result<Execution>;
    fn stats(&self) -> Stats;
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Key {
    source: String,
    entry: &'static str,
    device: u64,
    compiler: &'static str,
    dtype: &'static str,
    layout: &'static str,
    ir_version: u32,
}
struct Entry {
    key: Key,
    pipeline: ComputePipelineState,
}
pub struct MetalBackend {
    gpu: Gpu,
    config: Config,
    cache: VecDeque<Entry>,
    free: Vec<Uma>,
    stats: Stats,
}
impl MetalBackend {
    pub fn new(config: Config) -> Result<Self> {
        let config = config.validate()?;
        let gpu = Gpu::new()?;
        ensure!(
            gpu.device.has_unified_memory(),
            "shared UMA backend requires unified memory"
        );
        Ok(Self {
            gpu,
            config,
            cache: VecDeque::new(),
            free: Vec::new(),
            stats: Stats::default(),
        })
    }
    fn pipeline(&mut self, plan: &Plan, source: &str) -> Result<(ComputePipelineState, bool, f64)> {
        let key = Key {
            source: source.into(),
            entry: plan.kind.entry(),
            device: self.gpu.device.registry_id(),
            compiler: "fast_math=false",
            dtype: "q1_31_i32",
            layout: "row_major",
            ir_version: 3,
        };
        if let Some(index) = self.cache.iter().position(|e| e.key == key) {
            let entry = self.cache.remove(index).expect("index checked");
            let pipeline = entry.pipeline.clone();
            self.cache.push_back(entry);
            self.stats.pipeline_hits += 1;
            return Ok((pipeline, true, 0.));
        }
        ensure!(
            source.len() <= self.config.pipeline_source_bytes,
            "shader source budget"
        );
        self.stats.pipeline_misses += 1;
        let start = Instant::now();
        let pipeline = self.gpu.compile(source, key.entry)?;
        let compile_ms = start.elapsed().as_secs_f64() * 1000.;
        while self.cache.len() >= self.config.pipeline_entries
            || self.stats.pipeline_source_bytes + source.len() > self.config.pipeline_source_bytes
        {
            let old = self.cache.pop_front().expect("cache accounting invariant");
            self.stats.pipeline_source_bytes -= old.key.source.len();
            self.stats.pipeline_evictions += 1;
        }
        self.stats.pipeline_source_bytes += source.len();
        self.cache.push_back(Entry {
            key,
            pipeline: pipeline.clone(),
        });
        self.stats.pipeline_entries = self.cache.len();
        Ok((pipeline, false, compile_ms))
    }
    fn take(&mut self, len: usize, held: usize) -> Result<Uma> {
        if let Some(i) = self.free.iter().position(|b| b.len() == len) {
            let buffer = self.free.swap_remove(i);
            self.stats.idle_bytes -= buffer.allocated_size();
            self.stats.buffer_reuses += 1;
            return Ok(buffer);
        }
        let reserved = Uma::allocation_size(&self.gpu.device, len)?;
        ensure!(
            held + reserved <= self.config.resident_bytes,
            "live UMA budget"
        );
        while held + self.stats.idle_bytes + reserved > self.config.resident_bytes {
            let old = self.free.pop().expect("pool accounting invariant");
            self.stats.idle_bytes -= old.allocated_size();
            self.stats.buffer_evictions += 1;
        }
        let buffer = Uma::allocate(&self.gpu.device, len, 64)?;
        ensure!(
            buffer.allocated_size() <= reserved,
            "Metal allocation exceeded reserved size"
        );
        self.stats.buffer_allocations += 1;
        self.stats.peak_resident_bytes = self
            .stats
            .peak_resident_bytes
            .max(held + self.stats.idle_bytes + buffer.allocated_size());
        Ok(buffer)
    }
    fn recycle(&mut self, buffer: Uma) {
        if self.stats.idle_bytes + buffer.allocated_size() <= self.config.idle_bytes {
            self.stats.idle_bytes += buffer.allocated_size();
            self.free.push(buffer);
        } else {
            self.stats.buffer_evictions += 1;
        }
    }
}
impl KernelBackend for MetalBackend {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            backend: "metal",
            device: self.gpu.device.name().into(),
            device_registry_id: self.gpu.device.registry_id(),
            unified_memory: true,
            kernels: vec![
                KernelKind::GemmBiasRelu,
                KernelKind::AddRelu,
                KernelKind::Sin,
                KernelKind::Cos,
                KernelKind::Atan2,
                KernelKind::Tanh,
            ],
            dtypes: vec!["q1_31_i32"],
            layouts: vec!["row_major"],
            amx_execution: false,
            ane_execution: false,
        }
    }
    fn stats(&self) -> Stats {
        let mut stats = self.stats.clone();
        stats.device_current_allocated_bytes = self.gpu.device.current_allocated_size();
        stats
    }
    fn execute(
        &mut self,
        plan: &Plan,
        inputs: &Inputs,
        repeats: usize,
        capture: Option<&Path>,
    ) -> Result<Execution> {
        let start = Instant::now();
        execute::validate_inputs(plan, inputs)?;
        ensure!((1..=1000).contains(&repeats), "repeat budget");
        let mut lengths: Vec<usize> = plan.input_lengths().iter().map(|x| x * 4).collect();
        lengths.push(plan.m as usize * plan.n as usize * 4);
        ensure!(
            lengths
                .iter()
                .map(|len| Uma::allocation_size(&self.gpu.device, *len))
                .collect::<Result<Vec<_>>>()?
                .iter()
                .sum::<usize>()
                <= self.config.resident_bytes,
            "job UMA budget"
        );
        let source = plan.msl()?;
        let (pipeline, cache_hit, compile_ms) = self.pipeline(plan, &source)?;
        ensure!(
            pipeline.thread_execution_width() == 32,
            "expected SIMD width 32"
        );
        let mut buffers = Vec::new();
        let result = (|| -> Result<_> {
            let mut held = 0;
            for len in lengths {
                let buffer = self.take(len, held)?;
                held += buffer.allocated_size();
                buffers.push(buffer);
            }
            for (i, name) in plan.inputs.iter().enumerate() {
                buffers[i].write_i32(&inputs[name])?;
            }
            buffers.last_mut().expect("output exists").poison();
            let prepare_ms = start.elapsed().as_secs_f64() * 1000.;
            let _capture = capture
                .map(|path| Capture::start(&self.gpu, path))
                .transpose()?;
            let (groups, threads) = self.grid(plan);
            let refs: Vec<_> = buffers.iter().collect();
            let mut samples = Vec::new();
            for _ in 0..repeats {
                samples.push(self.gpu.dispatch(&pipeline, &refs, groups, threads)?);
            }
            let validation = Instant::now();
            let values = buffers.last().expect("output exists").read_i32()?;
            #[cfg(any(test, feature = "debug-validation"))]
            let max_abs_error = execute::compare(&values, &execute::reference(plan, inputs)?)?;
            #[cfg(not(any(test, feature = "debug-validation")))]
            let max_abs_error = 0.;
            let validation_ms = if cfg!(any(test, feature = "debug-validation")) {
                validation.elapsed().as_secs_f64() * 1000.
            } else {
                0.
            };
            Ok((values, samples, prepare_ms, validation_ms, max_abs_error))
        })();
        // dispatch waits for completion on success AND command errors before any recycling.
        for buffer in buffers {
            self.recycle(buffer);
        }
        let (values, samples, prepare_ms, validation_ms, max_abs_error) = result?;
        let output = Output {
            values,
            metrics: Metrics {
                validation_performed: cfg!(any(test, feature = "debug-validation")),
                kernel_dispatches: 1,
                compile_ms,
                dispatch_wall_ms: samples[0].dispatch_wall_ms,
                gpu_ms: samples[0].gpu_ms,
                max_abs_error: cfg!(any(test, feature = "debug-validation"))
                    .then_some(max_abs_error),
                prepare_ms,
                validation_ms,
                execution_wall_ms: start.elapsed().as_secs_f64() * 1000.,
                pipeline_cache_hit: cache_hit,
                resources: self.stats(),
            },
        };
        Ok((output, samples, source))
    }
}

/// Owned across await; the backend remains exclusive to one persistent runtime lane.
pub(crate) struct PendingExecution {
    completion: crate::completion::Submission,
    buffers: Vec<Uma>,
    plans: Vec<Plan>,
    inputs: Inputs,
    start: Instant,
    compile_ms: f64,
    prepare_ms: f64,
    cache_hit: bool,
}
pub(crate) struct CompletedExecution {
    pending: PendingExecution,
    timing: Result<Timing>,
}
impl PendingExecution {
    pub(crate) async fn wait(mut self) -> CompletedExecution {
        // Move the receiver out while retaining every tensor allocation through completion.
        let timing = self.completion.wait_ref().await;
        CompletedExecution {
            pending: self,
            timing,
        }
    }
}
impl MetalBackend {
    pub(crate) fn with_gate(config: Config, gate: crate::completion::FlightGate) -> Result<Self> {
        let config = config.validate()?;
        let gpu = Gpu::with_gate(gate)?;
        ensure!(
            gpu.device.has_unified_memory(),
            "shared UMA backend requires unified memory"
        );
        Ok(Self {
            gpu,
            config,
            cache: VecDeque::new(),
            free: Vec::new(),
            stats: Stats::default(),
        })
    }
    fn grid(&self, plan: &Plan) -> (metal::MTLSize, u64) {
        let gemm = plan.kind == KernelKind::GemmBiasRelu;
        let threads = self
            .config
            .threadgroup_threads
            .unwrap_or(if gemm { 32 } else { 256 });
        let groups = if gemm {
            metal::MTLSize::new(
                u64::from(plan.n).div_ceil(threads / 32),
                u64::from(plan.m),
                1,
            )
        } else {
            metal::MTLSize::new(
                (u64::from(plan.m) * u64::from(plan.n)).div_ceil(threads),
                1,
                1,
            )
        };
        (groups, threads)
    }
    #[cfg(test)]
    pub(crate) fn submit_owned(
        &mut self,
        plan: Plan,
        inputs: Inputs,
        permit: crate::completion::FlightPermit,
    ) -> Result<PendingExecution> {
        self.submit_chain_owned(vec![plan], inputs, permit)
    }
    pub(crate) fn submit_chain_owned(
        &mut self,
        plans: Vec<Plan>,
        inputs: Inputs,
        permit: crate::completion::FlightPermit,
    ) -> Result<PendingExecution> {
        let start = Instant::now();
        crate::chain::validate(&plans, &inputs)?;
        let mut lengths = Vec::new();
        let mut bindings = Vec::new();
        let mut writes = Vec::new();
        let mut previous = None;
        for plan in &plans {
            let mut bound = Vec::new();
            for (slot, (name, len)) in plan.inputs.iter().zip(plan.input_lengths()).enumerate() {
                if let Some(index) = previous.filter(|_| slot == 0) {
                    bound.push(index);
                } else {
                    bound.push(lengths.len());
                    writes.push((lengths.len(), name));
                    lengths.push(len * 4);
                }
            }
            let output = lengths.len();
            lengths.push(plan.m as usize * plan.n as usize * 4);
            bound.push(output);
            previous = Some(output);
            bindings.push(bound);
        }
        let reserved = lengths
            .iter()
            .map(|len| Uma::allocation_size(&self.gpu.device, *len))
            .collect::<Result<Vec<_>>>()?
            .iter()
            .sum::<usize>();
        ensure!(reserved <= self.config.resident_bytes, "job UMA budget");
        let mut pipelines = Vec::new();
        let mut compile_ms = 0.;
        let mut cache_hit = true;
        for plan in &plans {
            let source = plan.msl()?;
            let (pipeline, hit, ms) = self.pipeline(plan, &source)?;
            ensure!(
                pipeline.thread_execution_width() == 32,
                "expected SIMD width 32"
            );
            pipelines.push(pipeline);
            cache_hit &= hit;
            compile_ms += ms;
        }
        let mut buffers = Vec::new();
        let result = (|| -> Result<_> {
            let mut held = 0;
            for len in lengths {
                let buffer = self.take(len, held)?;
                held += buffer.allocated_size();
                buffers.push(buffer);
            }
            for (index, name) in writes {
                buffers[index].write_i32(&inputs[name])?;
            }
            for bound in &bindings {
                buffers[*bound.last().unwrap()].poison();
            }
            let stages: Vec<_> = plans
                .iter()
                .zip(&pipelines)
                .zip(&bindings)
                .map(|((plan, pipeline), bound)| {
                    let (groups, threads) = self.grid(plan);
                    crate::gpu::Dispatch {
                        pipeline,
                        buffers: bound.iter().map(|i| &buffers[*i]).collect(),
                        groups,
                        threads,
                    }
                })
                .collect();
            let prepare_ms = start.elapsed().as_secs_f64() * 1000.;
            let completion = self.gpu.submit_chain(&stages, permit)?;
            Ok((completion, prepare_ms))
        })();
        match result {
            Ok((completion, prepare_ms)) => Ok(PendingExecution {
                completion,
                buffers,
                plans,
                inputs,
                start,
                compile_ms,
                prepare_ms,
                cache_hit,
            }),
            Err(error) => {
                for buffer in buffers {
                    self.recycle(buffer);
                }
                Err(error)
            }
        }
    }
    #[cfg(test)]
    pub(crate) fn finish(&mut self, completed: CompletedExecution) -> Result<Output> {
        Ok(self.finish_with_validation_input(completed)?.0)
    }
    pub(crate) fn finish_with_validation_input(
        &mut self,
        completed: CompletedExecution,
    ) -> Result<(Output, crate::chain::ValidationInput)> {
        let CompletedExecution { pending, timing } = completed;
        let result = (|| -> Result<_> {
            let timing = timing?;
            let validation = Instant::now();
            let values = pending.buffers.last().expect("output exists").read_i32()?;
            #[cfg(any(test, feature = "debug-validation"))]
            let error = execute::compare(
                &values,
                &crate::chain::reference(&pending.plans, &pending.inputs)?,
            )?;
            #[cfg(not(any(test, feature = "debug-validation")))]
            let error = 0.;
            Ok((
                values,
                timing,
                error,
                if cfg!(any(test, feature = "debug-validation")) {
                    validation.elapsed().as_secs_f64() * 1000.
                } else {
                    0.
                },
            ))
        })();
        for b in pending.buffers {
            self.recycle(b);
        }
        let (values, timing, max_abs_error, validation_ms) = result?;
        Ok((
            Output {
                values,
                metrics: Metrics {
                    validation_performed: cfg!(any(test, feature = "debug-validation")),
                    kernel_dispatches: pending.plans.len(),
                    compile_ms: pending.compile_ms,
                    dispatch_wall_ms: timing.dispatch_wall_ms,
                    gpu_ms: timing.gpu_ms,
                    max_abs_error: cfg!(any(test, feature = "debug-validation"))
                        .then_some(max_abs_error),
                    prepare_ms: pending.prepare_ms,
                    validation_ms,
                    execution_wall_ms: pending.start.elapsed().as_secs_f64() * 1000.,
                    pipeline_cache_hit: pending.cache_hit,
                    resources: self.stats(),
                },
            },
            crate::chain::ValidationInput {
                plans: pending.plans,
                inputs: pending.inputs,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compile_failure_does_not_poison_cache() {
        let mut backend = MetalBackend::new(Config::default()).unwrap();
        let plan = crate::ir::lower_trace(include_bytes!("../fixtures/square.amvk")).unwrap();
        assert!(backend
            .pipeline(&plan, "this is deliberately invalid Metal source")
            .is_err());
        assert_eq!(backend.stats().pipeline_entries, 0);
        backend
            .execute(&plan, &execute::seeded_inputs(&plan, 9), 1, None)
            .unwrap();
        assert_eq!(backend.stats().pipeline_entries, 1);
    }
    #[tokio::test]
    async fn completion_error_recycles_only_after_native_completion() {
        let gate = crate::completion::FlightGate::new(2).unwrap();
        let mut backend = MetalBackend::with_gate(Config::default(), gate.clone()).unwrap();
        let plan = crate::ir::lower_trace(include_bytes!("../fixtures/square.amvk")).unwrap();
        let inputs = execute::seeded_inputs(&plan, 31);
        let pending = backend
            .submit_owned(plan.clone(), inputs.clone(), gate.acquire().await.unwrap())
            .unwrap();
        let mut completed = pending.wait().await;
        assert_eq!(gate.stats().completed, 1);
        completed.timing = Err(anyhow::anyhow!("injected native completion error"));
        assert!(backend.finish(completed).is_err());
        assert_eq!(backend.stats().buffer_allocations, 4);
        assert!(backend.stats().idle_bytes > 0);
        backend.execute(&plan, &inputs, 1, None).unwrap();
        assert_eq!(backend.stats().buffer_allocations, 4);
        assert_eq!(backend.stats().buffer_reuses, 4);
    }
    #[test]
    fn key_identity_covers_every_compile_dimension() {
        let key = Key {
            source: "source".into(),
            entry: "mvk",
            device: 1,
            compiler: "fast_math=false",
            dtype: "q1_31_i32",
            layout: "row_major",
            ir_version: 3,
        };
        let mut variants = vec![key.clone(); 7];
        variants[0].source = "other".into();
        variants[1].entry = "add_relu";
        variants[2].device = 2;
        variants[3].compiler = "fast_math=true";
        variants[4].dtype = "f16";
        variants[5].layout = "transposed";
        variants[6].ir_version = 4;
        for changed in variants {
            assert_ne!(changed, key);
        }
    }
}
