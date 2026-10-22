---
title: "Apple hardware coverage and qualification roadmap"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T04:19:49Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: ".gitignore"
    lines: "1-9"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-452"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-28"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-54"
    role: "primary"
  - path: "fixtures/atan2.amvk"
    lines: null
    role: "test"
  - path: "fixtures/cache-config-bench.json"
    lines: "1-13"
    role: "test"
  - path: "fixtures/cos.amvk"
    lines: null
    role: "test"
  - path: "fixtures/elementwise.amvk"
    lines: null
    role: "test"
  - path: "fixtures/native-model.path"
    lines: "1-1"
    role: "test"
  - path: "fixtures/qwen-prompts-long.json"
    lines: "1-4"
    role: "test"
  - path: "fixtures/qwen-prompts.json"
    lines: "1-6"
    role: "test"
  - path: "fixtures/qwen-serving-long.json"
    lines: "1-21"
    role: "test"
  - path: "fixtures/qwen-serving.json"
    lines: "1-21"
    role: "test"
  - path: "fixtures/rectangular.amvk"
    lines: null
    role: "test"
  - path: "fixtures/sin.amvk"
    lines: null
    role: "test"
  - path: "fixtures/square.amvk"
    lines: null
    role: "test"
  - path: "fixtures/tanh.amvk"
    lines: null
    role: "test"
  - path: "judges/benchmark.py"
    lines: "1-51"
    role: "test"
  - path: "judges/cordic.py"
    lines: "1-30"
    role: "test"
  - path: "judges/generate_cordic.py"
    lines: "1-44"
    role: "test"
  - path: "judges/macro.py"
    lines: "1-14"
    role: "test"
  - path: "judges/model_ab.py"
    lines: "1-93"
    role: "test"
  - path: "judges/resources.py"
    lines: "1-35"
    role: "test"
  - path: "judges/run.py"
    lines: "1-202"
    role: "test"
  - path: "native/mlx_bridge.cpp"
    lines: "1-165"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-46"
    role: "primary"
  - path: "native/telemetry.cpp"
    lines: "1-23"
    role: "primary"
  - path: "package-lock.json"
    lines: "1-1072"
    role: "config"
  - path: "package.json"
    lines: "1-21"
    role: "config"
  - path: "requirements-bench.txt"
    lines: "1-2"
    role: "config"
  - path: "run.sh"
    lines: "1-5"
    role: "primary"
  - path: "schema/trace.fbs"
    lines: "1-12"
    role: "primary"
  - path: "src/backend.rs"
    lines: "1-614"
    role: "primary"
  - path: "src/cache/mod.rs"
    lines: "1-1068"
    role: "primary"
  - path: "src/chain.rs"
    lines: "1-50"
    role: "primary"
  - path: "src/completion.rs"
    lines: "1-218"
    role: "primary"
  - path: "src/cordic.rs"
    lines: "1-204"
    role: "primary"
  - path: "src/cordic_tables.rs"
    lines: "1-102"
    role: "primary"
  - path: "src/execute.rs"
    lines: "1-122"
    role: "primary"
  - path: "src/gpu.rs"
    lines: "1-440"
    role: "primary"
  - path: "src/ir.rs"
    lines: "1-646"
    role: "primary"
  - path: "src/lib.rs"
    lines: "1-30"
    role: "primary"
  - path: "src/macro_bench.rs"
    lines: "1-81"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-415"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/completion_ledger.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/config.rs"
    lines: "1-51"
    role: "primary"
  - path: "src/serving/kv.rs"
    lines: "1-280"
    role: "primary"
  - path: "src/serving/mod.rs"
    lines: "1-81"
    role: "primary"
  - path: "src/serving/native_mlx.rs"
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-563"
    role: "primary"
  - path: "src/serving/weights.rs"
    lines: "1-395"
    role: "primary"
  - path: "src/telemetry.rs"
    lines: "1-26"
    role: "primary"
  - path: "src/trace_generated.rs"
    lines: "1-1523"
    role: "primary"
  - path: "src/wire.rs"
    lines: "1-124"
    role: "primary"
  - path: "tests/adversarial.rs"
    lines: "1-456"
    role: "test"
  - path: "tests/async_completion.rs"
    lines: "1-192"
    role: "test"
  - path: "tests/cache_control.rs"
    lines: "1-419"
    role: "test"
  - path: "tests/cordic.rs"
    lines: "1-264"
    role: "test"
  - path: "tests/fixed_point.rs"
    lines: "1-146"
    role: "test"
  - path: "tests/grid.rs"
    lines: "1-41"
    role: "test"
  - path: "tests/native_execution.rs"
    lines: "1-486"
    role: "test"
  - path: "tests/native_mlx.rs"
    lines: "1-297"
    role: "test"
  - path: "tests/production_pipeline.rs"
    lines: "1-315"
    role: "test"
  - path: "tests/rapid_rpc.rs"
    lines: "1-76"
    role: "test"
  - path: "tests/resources.rs"
    lines: "1-341"
    role: "test"
  - path: "tests/serving.rs"
    lines: "1-426"
    role: "test"
  - path: "ts/generated/trace.ts"
    lines: "1-5"
    role: "primary"
  - path: "ts/generated/wire/add.ts"
    lines: "1-68"
    role: "primary"
  - path: "ts/generated/wire/cordic.ts"
    lines: "1-78"
    role: "primary"
  - path: "ts/generated/wire/dot.ts"
    lines: "1-68"
    role: "primary"
  - path: "ts/generated/wire/load.ts"
    lines: "1-81"
    role: "primary"
  - path: "ts/generated/wire/op.ts"
    lines: "1-61"
    role: "primary"
  - path: "ts/generated/wire/operation.ts"
    lines: "1-54"
    role: "primary"
  - path: "ts/generated/wire/relu.ts"
    lines: "1-58"
    role: "primary"
  - path: "ts/generated/wire/store.ts"
    lines: "1-61"
    role: "primary"
  - path: "ts/generated/wire/trace.ts"
    lines: "1-91"
    role: "primary"
  - path: "ts/generated/wire.ts"
    lines: "1-13"
    role: "primary"
  - path: "ts/smoke.ts"
    lines: "1-1"
    role: "primary"
  - path: "ts/trace.test.ts"
    lines: "1-45"
    role: "test"
  - path: "ts/trace.ts"
    lines: "1-73"
    role: "primary"
  - path: "tsconfig.json"
    lines: "1-1"
    role: "config"
related_issues: []
related_documents: ["RESOURCE_BACKEND_DESIGN.md", "../README.md", "NEXT_STEPS.md"]
tags: [apple-hardware, metal, dynamic-inference, backend]
---
> Numerical update: float/matrix-kernel descriptions and measurements below refer to the original prototype. The active integer/shuffle contract is documented in [FIXED_POINT.md](FIXED_POINT.md) and README.md. The user confirmed Q1.31 storage; sin, cos, atan2 and tanh are now wired through the runtime. The runtime now has semaphore-capped 2/3-slot completion callbacks; see ASYNC_COMPLETION.md. Earlier single-worker descriptions below are historical.

# Context

The user explicitly defines the target as the gold standard for throughput and dynamic inference across macOS, GPU/Metal, CPU AMX and ANE: the Triton for Apple hardware. The target is broader than the initial one-kernel spike. AMX was confirmed as the intended correction to ANX. The goal is saved as a user preference; it is not an achieved performance claim.

# Decision

Expose precise capabilities and register only execution paths that have been tested. Backend extension comes before automatic routing. Every lane must pass numerical and placement checks with the same workload, and report compile/conversion, setup, execution, memory and synchronization overhead separately. An accelerator request must fail explicitly if that backend is unavailable; silent CPU fallback must not be labeled ANE execution.

| Lane | Current status | Candidate public route | Required next proof |
| --- | --- | --- | --- |
| Metal GPU | Implemented and tested for two f32 families | MTL shared buffers, shader compiler, SIMD-group matrix operations, scalar grid and capture | Model operators, low precision, sustained dynamic workloads and independent incumbent comparison |
| CPU / AMX target | No execution backend registered | Accelerate BLAS/BNNS/BNNSGraph candidates | CPU numerical parity, end-to-end costs, supported dtype/shape coverage, and evidence for any AMX-specific utilization claim |
| ANE target | No execution backend registered | Core ML eligible compiled models, compute-unit selection and compute-plan inspection | A converted workload that runs correctly; supported/preferred devices recorded; actual device execution observed with profiling; compilation/fallback costs reported |

Apple documents Accelerate as a CPU abstraction whose implementations choose instructions for the available processor; calling Accelerate alone does not establish a portable guarantee of AMX placement. Core ML's compute-unit selection expresses permitted devices. The installed SDK describes compute-plan device usage as anticipated execution and preferred devices, not a runtime trace. Therefore a requested setting or anticipated plan is not sufficient to claim that ANE executed the work.

# Qualification order

1. Preserve the qualified Metal path and add model repository metadata plus exact forward/operator coverage before serving.
2. Design and implement small Accelerate CPU and Core ML candidate proofs independently. Record availability, supported operations, shape changes, precision, conversion/compile time and runtime fallback behavior.
3. Qualify each backend on the same deterministic inputs and correctness tolerance, including tails, nonfinite values and concurrency/ownership tests. Use actual device observations where the platform exposes them; retain uncertainty otherwise.
4. Only then add measured routing policy by shape, precision, deadline, memory and thermal state. Account for synchronization and conversion costs. Shared physical memory does not remove synchronization or format requirements.
5. Qualify dynamic model inference, continuous batching, exact KV and streaming on held-out workload traces against the incumbent lanes. Claim gold-standard performance only after reproducible comparisons, not because an API exists.

# Consequences

Current capabilities intentionally report amx_execution=false and ane_execution=false. This makes the gap visible while establishing a backend seam. It avoids blocking the working GPU baseline on device-specific conversion and profiling work. No private AMX instructions, private ANE framework or undocumented accelerator capability is assumed.

# Alternatives considered

Labeling an Accelerate call as direct AMX control or a Core ML compute setting as proven ANE execution would make placement claims without evidence. Automatically mixing unqualified backends would obscure correctness and timing. Both are rejected.

# Literature grounding and owning references

This roadmap uses API documentation, not a published research speedup. Primary references checked during this run:

- Apple, Accelerate overview: https://developer.apple.com/accelerate/ — CPU numerical/ML APIs and processor abstraction.
- Apple, CPU inference with BNNSGraph: https://developer.apple.com/videos/play/wwdc2024/10211/ — candidate CPU inference route.
- Apple, MLComputePlan: https://developer.apple.com/documentation/coreml/mlcomputeplan — compute planning API.
- Apple, MLComputePlanDeviceUsage: https://developer.apple.com/documentation/coreml/mlcomputeplandeviceusage — supported/preferred device information.
- Apple, cpuAndNeuralEngine: https://developer.apple.com/documentation/coreml/mlcomputeunits/cpuandneuralengine — permitted compute-unit setting.

The local Xcode SDK owning headers `CoreML.framework/Headers/MLComputePlan.h`, `MLComputePlanDeviceUsage.h` and `MLModelConfiguration.h` were inspected directly. They establish API availability and anticipated-device semantics; no Core ML model was executed in this stage.

# Changelog

1.0.0 — 2026-10-21 — Codex — Saved goal, clarified AMX, truthful current capabilities and hardware qualification gates.
