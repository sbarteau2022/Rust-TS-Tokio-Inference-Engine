#![allow(unexpected_cfgs)]
use anyhow::{anyhow, ensure, Result};
use metal::*;
use objc::{msg_send, sel, sel_impl};
#[derive(Debug, Clone, serde::Serialize)]
pub struct Timing {
    pub dispatch_wall_ms: f64,
    pub gpu_ms: f64,
}

use std::time::Instant;

unsafe extern "C" {
    fn rs_host_page_size() -> u64;
}
pub const MAX_BUFFER_BYTES: usize = 64 * 1024 * 1024;
pub struct Dispatch<'a> {
    pub pipeline: &'a ComputePipelineStateRef,
    pub buffers: Vec<&'a Uma>,
    pub groups: MTLSize,
    pub threads: u64,
}
pub struct Uma {
    buffer: Buffer,
    len: usize,
}
impl Uma {
    pub fn allocate(device: &DeviceRef, len: usize, alignment: usize) -> Result<Self> {
        ensure!(
            alignment >= 64 && alignment.is_power_of_two(),
            "alignment must be power of two >=64"
        );
        ensure!(len > 0 && len <= MAX_BUFFER_BYTES, "buffer budget");
        let buffer = device.new_buffer(len as u64, MTLResourceOptions::StorageModeShared);
        ensure!(!(buffer.contents().is_null()), "null UMA allocation");
        ensure!(
            (buffer.contents() as usize).is_multiple_of(alignment),
            "UMA address misaligned"
        );
        Ok(Self { buffer, len })
    }
    pub fn allocation_size(device: &DeviceRef, len: usize) -> Result<usize> {
        ensure!(len > 0 && len <= MAX_BUFFER_BYTES, "buffer budget");
        let heap_required = device
            .heap_buffer_size_and_align(len as u64, MTLResourceOptions::StorageModeShared)
            .size as usize;
        ensure!(
            heap_required >= len,
            "Metal allocation-size query unavailable"
        );
        let page = unsafe { rs_host_page_size() } as usize;
        ensure!(
            page > 0 && page.is_power_of_two(),
            "host page size unavailable"
        );
        // Heap requirements alone under-reserve direct shared buffers on this Mac.
        // Reserve whole host pages and verify actual allocatedSize before admitting.
        Ok(heap_required.max(len).div_ceil(page) * page)
    }
    pub fn allocated_size(&self) -> usize {
        self.buffer.allocated_size() as usize
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn write_i32(&mut self, values: &[i32]) -> Result<()> {
        ensure!(values.len() * 4 == self.len, "i32 byte length mismatch");
        // Shared allocation is 64-byte aligned; exclusive host writes precede submission.
        unsafe {
            std::ptr::copy_nonoverlapping(
                values.as_ptr(),
                self.buffer.contents().cast::<i32>(),
                values.len(),
            );
        }
        Ok(())
    }
    pub fn poison(&mut self) {
        // Exclusive host access, no outstanding command owns this pooled allocation.
        unsafe {
            std::ptr::write_bytes(self.buffer.contents().cast::<u8>(), 0xff, self.len);
        }
    }
    pub fn read_i32(&self) -> Result<Vec<i32>> {
        ensure!(self.len.is_multiple_of(4), "not i32 storage");
        // Reads follow command completion; callers cannot obtain the underlying buffer.
        Ok(unsafe {
            std::slice::from_raw_parts(self.buffer.contents().cast::<i32>(), self.len / 4)
        }
        .to_vec())
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        ensure!(bytes.len() == self.len, "input byte length mismatch");
        // Exclusive host access. Buffers are not submitted until all writes complete.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.contents().cast(), self.len);
        }
        Ok(())
    }
    pub fn read(&self) -> Vec<u8> {
        // Only called after command completion; ownership prevents concurrent host mutation.
        unsafe {
            std::slice::from_raw_parts(self.buffer.contents().cast::<u8>(), self.len).to_vec()
        }
    }
}
pub struct Gpu {
    pub device: Device,
    pub queue: CommandQueue,
    gate: crate::completion::FlightGate,
}
impl Gpu {
    pub fn new() -> Result<Self> {
        Self::with_gate(crate::completion::FlightGate::new(3)?)
    }
    pub fn with_gate(gate: crate::completion::FlightGate) -> Result<Self> {
        let device = Device::system_default().ok_or_else(|| anyhow!("Metal device absent"))?;
        let queue = device.new_command_queue();
        Ok(Self {
            device,
            queue,
            gate,
        })
    }
    pub fn compile(&self, source: &str, name: &str) -> Result<ComputePipelineState> {
        objc::rc::autoreleasepool(|| self.compile_inner(source, name))
    }
    fn compile_inner(&self, source: &str, name: &str) -> Result<ComputePipelineState> {
        let options = CompileOptions::new();
        options.set_fast_math_enabled(false);
        let lib = self
            .device
            .new_library_with_source(source, &options)
            .map_err(|e| anyhow!(e))?;
        let function = lib.get_function(name, None).map_err(|e| anyhow!(e))?;
        self.device
            .new_compute_pipeline_state_with_function(&function)
            .map_err(|e| anyhow!(e))
    }
    pub fn dispatch(
        &self,
        pipeline: &ComputePipelineStateRef,
        buffers: &[&Uma],
        groups: MTLSize,
        threads: u64,
    ) -> Result<Timing> {
        self.submit(pipeline, buffers, groups, threads, self.gate.try_acquire()?)?
            .wait_blocking()
    }
    pub fn submit(
        &self,
        pipeline: &ComputePipelineStateRef,
        buffers: &[&Uma],
        groups: MTLSize,
        threads: u64,
        permit: crate::completion::FlightPermit,
    ) -> Result<crate::completion::Submission> {
        self.submit_inner(pipeline, buffers, groups, threads, permit, None)
    }
    fn submit_inner(
        &self,
        pipeline: &ComputePipelineStateRef,
        buffers: &[&Uma],
        groups: MTLSize,
        threads: u64,
        permit: crate::completion::FlightPermit,
        wait: Option<&EventRef>,
    ) -> Result<crate::completion::Submission> {
        self.submit_stages(
            &[Dispatch {
                pipeline,
                buffers: buffers.to_vec(),
                groups,
                threads,
            }],
            permit,
            wait,
        )
    }
    pub fn submit_chain(
        &self,
        stages: &[Dispatch<'_>],
        permit: crate::completion::FlightPermit,
    ) -> Result<crate::completion::Submission> {
        self.submit_stages(stages, permit, None)
    }
    fn submit_stages(
        &self,
        stages: &[Dispatch<'_>],
        permit: crate::completion::FlightPermit,
        wait: Option<&EventRef>,
    ) -> Result<crate::completion::Submission> {
        objc::rc::autoreleasepool(|| {
            ensure!(
                permit.belongs_to(&self.gate),
                "permit belongs to a different gate"
            );
            ensure!((1..=16).contains(&stages.len()), "chain stage bound 1..16");
            for stage in stages {
                ensure!(
                    stage.threads > 0
                        && stage.threads <= stage.pipeline.max_total_threads_per_threadgroup(),
                    "thread limit"
                );
                ensure!(
                    stage.groups.width > 0 && stage.groups.height > 0 && stage.groups.depth > 0,
                    "empty grid"
                );
                ensure!(
                    stage.buffers.len() <= 4 && (stages.len() == 1 || !stage.buffers.is_empty()),
                    "stage buffer count"
                );
            }
            let start = Instant::now();
            let command = self.queue.new_command_buffer();
            if let Some(event) = wait {
                command.encode_wait_for_event(event, 1);
            }
            let encoder = command.new_compute_command_encoder();
            for (index, stage) in stages.iter().enumerate() {
                if index > 0 {
                    let activation = stages[index - 1].buffers.last().expect("validated stage");
                    encoder.memory_barrier_with_resources(&[&activation.buffer]);
                }
                encoder.set_compute_pipeline_state(stage.pipeline);
                for (i, b) in stage.buffers.iter().enumerate() {
                    let usage = if i + 1 == stage.buffers.len() {
                        MTLResourceUsage::Write
                    } else {
                        MTLResourceUsage::Read
                    };
                    encoder.use_resource(&b.buffer, usage);
                    encoder.set_buffer(i as u64, Some(&b.buffer), 0);
                }
                encoder.dispatch_thread_groups(stage.groups, MTLSize::new(stage.threads, 1, 1));
            }
            encoder.end_encoding();
            let retained = stages
                .iter()
                .flat_map(|s| s.buffers.iter().map(|b| b.buffer.clone()))
                .collect();
            let (bridge, submission) = crate::completion::Bridge::new(permit, retained);
            let callback = block::ConcreteBlock::new(move |completed: &CommandBufferRef| {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if completed.status() != MTLCommandBufferStatus::Completed {
                        return Err(format!("Metal command failed: {:?}", completed.status()));
                    }
                    let gpu_start: f64 = unsafe { msg_send![completed, GPUStartTime] };
                    let gpu_end: f64 = unsafe { msg_send![completed, GPUEndTime] };
                    if !(gpu_start > 0. && gpu_end >= gpu_start) {
                        return Err("GPU timestamps unavailable".into());
                    }
                    Ok(Timing {
                        dispatch_wall_ms: start.elapsed().as_secs_f64() * 1000.,
                        gpu_ms: (gpu_end - gpu_start) * 1000.,
                    })
                }))
                .unwrap_or_else(|_| Err("completion callback panic contained".into()));
                bridge.complete(result);
            })
            .copy();
            command.add_completed_handler(&callback);
            self.gate.submitted();
            command.commit();
            Ok(submission)
        })
    }
    pub fn identity_judge(&self) -> Result<serde_json::Value> {
        let len = 4096;
        let pattern: Vec<u8> = (0..len).map(|i| ((i * 37 + 11) % 256) as u8).collect();
        let mut a = Uma::allocate(&self.device, len, 64)?;
        a.write(&pattern)?;
        let b = Uma::allocate(&self.device, len, 64)?;
        let source="#include <metal_stdlib>\nusing namespace metal; kernel void identity(device const uint* a [[buffer(0)]],device uint* b [[buffer(1)]],uint i [[thread_position_in_grid]]) { if(i < 1024) b[i]=a[i]; }";
        let pipeline = self.compile(source, "identity")?;
        self.dispatch(&pipeline, &[&a, &b], MTLSize::new(128, 1, 1), 32)?;
        ensure!(b.read() == pattern, "identity bytes differ");
        ensure!(
            Uma::allocate(&self.device, len, 3).is_err(),
            "misalignment accepted"
        );
        ensure!(
            Uma::allocate(&self.device, 0, 64).is_err(),
            "empty allocation accepted"
        );
        Ok(
            serde_json::json!({"pass":true,"bytes":len,"storage":"shared","cpu_address_mod_64":a.buffer.contents() as usize%64,"misalignment_rejected":true,"device":self.device.name()}),
        )
    }
}

/// Explicit opt-in capture. Dropping the guard stops capture even on a job error.
pub struct Capture;
impl Capture {
    pub fn start(gpu: &Gpu, path: &std::path::Path) -> Result<Self> {
        ensure!(
            std::env::var("METAL_CAPTURE_ENABLED").as_deref() == Ok("1"),
            "capture requires METAL_CAPTURE_ENABLED=1 before process startup"
        );
        ensure!(
            path.is_absolute() && !path.exists(),
            "capture path must be absolute and absent"
        );
        let manager = CaptureManager::shared();
        ensure!(!manager.is_capturing(), "capture already active");
        ensure!(
            manager.supports_destination(MTLCaptureDestination::GpuTraceDocument),
            "GPU capture unsupported"
        );
        let desc = CaptureDescriptor::new();
        desc.set_capture_command_queue(&gpu.queue);
        desc.set_destination(MTLCaptureDestination::GpuTraceDocument);
        desc.set_output_url(path);
        manager.start_capture(&desc).map_err(|e| anyhow!(e))?;
        Ok(Self)
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        CaptureManager::shared().stop_capture();
    }
}

#[cfg(test)]
mod completion_tests {
    use super::*;
    use crate::completion::FlightGate;
    struct ReleaseOnDrop(SharedEvent);
    impl Drop for ReleaseOnDrop {
        fn drop(&mut self) {
            self.0.set_signaled_value(1);
        }
    }
    #[tokio::test]
    async fn native_completion_holds_two_or_three_permits_and_resources() {
        for limit in [2, 3] {
            let gate = FlightGate::new(limit).unwrap();
            let gpu = Gpu::with_gate(gate.clone()).unwrap();
            let config =
                serde_json::from_str(include_str!("../fixtures/cache-config-bench.json")).unwrap();
            let cache = crate::cache::Controller::new(config).unwrap();
            let branch = cache
                .branch(
                    None,
                    crate::cache::Identity {
                        model: "native-test".into(),
                        tokenizer: "test".into(),
                        quantization: "test".into(),
                        namespace: "private-test".into(),
                    },
                    vec![1; 16],
                )
                .unwrap();
            let ticket = match cache.request_prefill(branch).unwrap() {
                crate::cache::Admission::Ready(id) => id,
                _ => panic!("empty pool"),
            };
            cache
                .fulfill(ticket, |request| {
                    Ok(request.missing.iter().map(|_| vec![0; 64]).collect())
                })
                .unwrap();

            let pipeline=gpu.compile("#include <metal_stdlib>\nusing namespace metal; kernel void identity(device const uint* A [[buffer(0)]],device uint* B [[buffer(1)]],uint i [[thread_position_in_grid]]) {if(i==0) B[0]=A[0];}","identity").unwrap();
            // RAII always unblocks the native queue, including an assertion unwind.
            let release = ReleaseOnDrop(gpu.device.new_shared_event());
            let mut owned = Vec::new();
            let mut submissions = Vec::new();
            for i in 0..limit {
                let mut a = Uma::allocate(&gpu.device, 4, 64).unwrap();
                a.write_i32(&[i as i32 + 71]).unwrap();
                let mut b = Uma::allocate(&gpu.device, 4, 64).unwrap();
                b.poison();
                let mut permit = gate.acquire().await.unwrap();
                permit.retain(cache.lease(branch).unwrap());
                submissions.push(
                    gpu.submit_inner(
                        &pipeline,
                        &[&a, &b],
                        MTLSize::new(1, 1, 1),
                        32,
                        permit,
                        Some(&release.0),
                    )
                    .unwrap(),
                );
                owned.push((a, b));
            }
            let s = gate.stats();
            assert_eq!(s.outstanding, limit);
            assert_eq!(s.peak_outstanding, limit);
            assert_eq!(s.active, limit);
            assert!(gate.try_acquire().is_err());
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(5), gate.acquire())
                    .await
                    .is_err()
            );
            // A dropped future must not free a committed slot or its retained GPU buffers.
            let mut canceled = submissions.pop().unwrap();
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(5), canceled.wait_ref())
                    .await
                    .is_err()
            );
            drop(canceled);
            assert_eq!(cache.stats().unwrap().active_leases, limit);
            cache.close_subtree(branch).unwrap();
            assert!(cache.boundary().is_err());

            assert_eq!(gate.stats().active, limit);
            drop(owned.remove(0).0); // Retained by native submission through completion.
            release.0.set_signaled_value(1);
            for submission in submissions {
                tokio::time::timeout(std::time::Duration::from_secs(5), submission.wait())
                    .await
                    .unwrap()
                    .unwrap();
            }
            tokio::time::timeout(std::time::Duration::from_secs(5), gate.drain())
                .await
                .unwrap();
            assert_eq!(cache.stats().unwrap().active_leases, 0);
            cache.boundary().unwrap();
            for (i, (_, b)) in owned.iter().enumerate() {
                assert_eq!(b.read_i32().unwrap(), vec![i as i32 + 72]);
            }
            let s = gate.stats();
            assert_eq!(s.active, 0);
            assert_eq!(s.outstanding, 0);
            assert_eq!(s.submitted, limit);
            assert_eq!(s.completed, limit);
            assert_eq!(s.acquired, s.released);
            assert_eq!(s.completion_errors, 0);
        }
    }
}
