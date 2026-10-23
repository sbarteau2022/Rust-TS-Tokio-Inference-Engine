---
title: "Rust Server — Apple native kernel compiler and runtime"
document_type: "implementation"
version: "1.29.0"
date: "2026-10-23T05:21:07Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: ".gitignore"
    lines: "1-9"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-1015"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-73"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-63"
    role: "primary"
  - path: "examples/native_automaton_smoke.rs"
    lines: "1-394"
    role: "primary"
  - path: "examples/native_device_pressure.rs"
    lines: "1-184"
    role: "primary"
  - path: "examples/native_device_reference.rs"
    lines: "1-166"
    role: "primary"
  - path: "examples/native_discriminators.rs"
    lines: "1-145"
    role: "primary"
  - path: "examples/native_hidden_smoke.rs"
    lines: "1-170"
    role: "primary"
  - path: "examples/native_mtp_automaton_smoke.rs"
    lines: "1-218"
    role: "primary"
  - path: "examples/native_mtp_bench.rs"
    lines: "1-130"
    role: "primary"
  - path: "examples/native_mtp_decode_smoke.rs"
    lines: "1-160"
    role: "primary"
  - path: "examples/native_mtp_smoke.rs"
    lines: "1-186"
    role: "primary"
  - path: "examples/native_speculative_smoke.rs"
    lines: "1-370"
    role: "primary"
  - path: "examples/native_suite_reference.rs"
    lines: "1-198"
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
  - path: "fixtures/native-choice-heldout.tsv"
    lines: "1-4"
    role: "test"
  - path: "fixtures/native-domain-heldout.tsv"
    lines: "1-24"
    role: "test"
  - path: "fixtures/native-heldout.tsv"
    lines: "1-16"
    role: "test"
  - path: "fixtures/native-interpretation-probes.tsv"
    lines: "1-4"
    role: "test"
  - path: "fixtures/native-model.path"
    lines: "1-1"
    role: "test"
  - path: "fixtures/native-speculative-prompts.tsv"
    lines: "1-4"
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
  - path: "judges/attention_identical_inputs.py"
    lines: "1-72"
    role: "test"
  - path: "judges/attention_profile.py"
    lines: "1-44"
    role: "test"
  - path: "judges/benchmark.py"
    lines: "1-51"
    role: "test"
  - path: "judges/cordic.py"
    lines: "1-30"
    role: "test"
  - path: "judges/dense_precision.cpp"
    lines: "1-21"
    role: "test"
  - path: "judges/device_abi.cpp"
    lines: "1-49"
    role: "test"
  - path: "judges/device_smoke_report.py"
    lines: "1-38"
    role: "test"
  - path: "judges/generate_cordic.py"
    lines: "1-44"
    role: "test"
  - path: "judges/layer_reference.py"
    lines: "1-185"
    role: "test"
  - path: "judges/macro.py"
    lines: "1-14"
    role: "test"
  - path: "judges/model_ab.py"
    lines: "1-93"
    role: "test"
  - path: "judges/operator_reference.cpp"
    lines: "1-29"
    role: "test"
  - path: "judges/projection_reference.cpp"
    lines: "1-53"
    role: "test"
  - path: "judges/resources.py"
    lines: "1-35"
    role: "test"
  - path: "judges/run.py"
    lines: "1-230"
    role: "test"
  - path: "judges/speculative_report.py"
    lines: "1-74"
    role: "test"
  - path: "judges/state_table.cpp"
    lines: "1-21"
    role: "test"
  - path: "judges/suite_reference.py"
    lines: "1-248"
    role: "test"
  - path: "judges/test_attention_profile.py"
    lines: "1-51"
    role: "test"
  - path: "judges/test_device_smoke_report.py"
    lines: "1-26"
    role: "test"
  - path: "judges/test_layer_reference.py"
    lines: "1-56"
    role: "test"
  - path: "judges/test_speculative_report.py"
    lines: "1-34"
    role: "test"
  - path: "judges/test_suite_reference.py"
    lines: "1-107"
    role: "test"
  - path: "native/attention_reference.h"
    lines: "1-21"
    role: "primary"
  - path: "native/mlx_bridge.cpp"
    lines: "1-178"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-112"
    role: "primary"
  - path: "native/operator_probe.inc"
    lines: "1-28"
    role: "primary"
  - path: "native/operator_reference.h"
    lines: "1-58"
    role: "primary"
  - path: "native/projection_probe.inc"
    lines: "1-29"
    role: "primary"
  - path: "native/projection_reference.h"
    lines: "1-30"
    role: "primary"
  - path: "native/qwen_device.inc"
    lines: "1-59"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-173"
    role: "primary"
  - path: "native/qwen_mtp.inc"
    lines: "1-55"
    role: "primary"
  - path: "native/state_table.h"
    lines: "1-38"
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
  - path: "src/cache/drafts.rs"
    lines: "1-599"
    role: "primary"
  - path: "src/cache/mod.rs"
    lines: "1-1078"
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
    lines: "1-453"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/choices.rs"
    lines: "1-123"
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
    lines: "1-107"
    role: "primary"
  - path: "src/serving/model_ir.rs"
    lines: "1-244"
    role: "primary"
  - path: "src/serving/native_automaton.rs"
    lines: "1-328"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-809"
    role: "primary"
  - path: "src/serving/native_device_scheduler.rs"
    lines: "1-178"
    role: "primary"
  - path: "src/serving/native_device_smoke.rs"
    lines: "1-162"
    role: "primary"
  - path: "src/serving/native_diagnose.rs"
    lines: "1-104"
    role: "primary"
  - path: "src/serving/native_extended.rs"
    lines: "1-214"
    role: "primary"
  - path: "src/serving/native_ingestion.rs"
    lines: "1-272"
    role: "primary"
  - path: "src/serving/native_layer_reference.rs"
    lines: "1-93"
    role: "primary"
  - path: "src/serving/native_mlx.rs"
    lines: "1-950"
    role: "primary"
  - path: "src/serving/native_mtp.rs"
    lines: "1-468"
    role: "primary"
  - path: "src/serving/native_mtp_automaton.rs"
    lines: "1-98"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-149"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-436"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-690"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-382"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-213"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-566"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-645"
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
  - path: "tests/draft_admission.rs"
    lines: "1-475"
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
  - path: "tests/output_choices.rs"
    lines: "1-110"
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
    lines: "1-435"
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
related_documents: ["docs/DESIGN.md", "docs/SOURCE_SPEC.md", "docs/ADVERSARIAL_REVIEW.md", "docs/NEXT_STEPS.md", "docs/FILE_MAP.md"]
tags: [rust, tokio, metal, typescript-7.1, tsrs, flatbuffers]
---
# Rust Server

## Executive Summary

A TypeScript trace reaches pure integer Metal kernels through Rust, shared buffers and Tokio receipts. The default build now also has an in-process Rust-owned MLX tensor executor with a compiled C++ bridge, typed graphs and completion-owned resource admission; it uses no Python worker or JSON transport. The earlier external model adapter is disabled by default. Full native Qwen generation, tensor batching and AMX/ANE remain unqualified; see [native backend design and framework synthesis](docs/NATIVE_MLX_BACKEND.md).

Spec 15 now owns opt-in proposer admission and deterministic arbitration: bounded qualification/probes, measured cost per committed token, and audit-epoch trust decay. Native draft leases survive waiter cancellation through backend completion. Retrieval is the supplied plugin; no trained MTP is active. See [design, configuration and evidence](docs/SPEC15_DRAFT_ADMISSION.md). Cache declared-context coverage and speculative committed tokens per target call are separate audit figures.

## Problem Statement

The original float prototype could not satisfy the requested deterministic fixed-point execution. The numerical contract and its judges needed to change together.

## Solution Overview

`TypeScript 7.1 → FlatBuffers version 3 → verified borrowed views → Rust SSA → integer MSL → shared Metal buffers → Tokio receipt`.

The currently tested integer baseline is **signed Q1.31 i32**: raw values represent value/2147483648. Each operand is cast to `int64_t` before multiplication; each product is floor-shifted by 31, then summed in 64 bits. Bias, ReLU and final saturation to INT32_MAX follow. Every output must match exactly. The user confirmed 32-bit storage and 31 fractional bits. CORDIC sin, cos, atan2 and tanh are implemented through the same pipeline.

Each GEMM output uses one 32-thread SIMD-group. Lanes accumulate strided K terms, then reduce signed 64-bit partials through five shuffle-down stages, shuffling both uint32 halves and reconstructing the full word. No threadgroup scratch or barriers occur in this reduction. One SIMD-group is dispatched per threadgroup/output; cross-group reduction is not needed. Runtime checks the pipeline execution width is 32. This establishes correctness, not a measured speedup over other grid choices.

The six supported graph families are fixed-point GEMM+bias+ReLU, add+bias+ReLU, CORDIC sin, cos, atan2 and tanh. Subtraction is not a standalone graph operation; CORDIC uses integer add/sub internally. Unsupported graphs fail closed. See [contract](docs/FIXED_POINT.md).

### CORDIC units and limits

Sin/cos inputs and atan2 outputs are normalized radians/π. Raw 0 means zero angle; 1073741824 means π/2; INT32_MIN means -π. Positive π wraps to -π; atan2(0,0)=0. Atan2 inputs are Y then X. Tanh accepts dimensionless Q1.31 inputs in [-1,1); wider activation ranges need a separate scale contract. Raw outputs are Q1.31. Positive unity saturates to INT32_MAX.

CORDIC uses 48 circular iterations and hyperbolic iterations 1–47 with repeated steps 4,13,40, Q60 int64 internal state and restoring integer division for tanh. Constants are committed and regenerated with 90-digit Decimal arithmetic. CPU/GPU results must match bit for bit; analytic accuracy has a separate 2.01-raw-unit gate. This is deterministic approximation, not exact transcendental arithmetic.

## Build and use

Run from `/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine`:

```sh
# package.json, ts/trace.ts and src/main.rs — build and execute raw integer inputs.
npm ci
npx tsrs -p tsconfig.json
npx tsc --version
cargo build --release --locked
npx tsx ts/trace.ts
./run.sh kernel-judge artifacts
./run.sh run artifacts/trace.amvk artifacts/inputs.json
./run.sh capabilities
# ts/trace.ts — choose sin, cos, atan2 or tanh; dimensions M x N.
npx tsx ts/trace.ts artifacts/tanh 73 1 91 tanh
./run.sh kernel-judge artifacts/tanh
./run.sh run artifacts/tanh/trace.amvk artifacts/tanh/inputs.json
```

`run TRACE INPUTS.json` accepts a JSON object mapping input names to flat **raw i32 integer arrays**. Float values and integers outside i32 range are rejected. Trace versions 1 and 2 are rejected so old float traces cannot silently acquire integer semantics. The schema's legacy default remains 1 deliberately; the current producer explicitly writes 3.

TypeScript is pinned to **7.1.0-beta**, tsrs to **0.18.0-ts7.1.0-dev.20260929**. Both compiler gates are required. Flatc is 25.12.19; Rust and Node dependencies have lockfiles. Current judges use Python's standard library, not MLX. `requirements-bench.txt` records the historical float comparison dependency only.

```sh
# src/gpu.rs — native capture, destination must be absent.
METAL_CAPTURE_ENABLED=1 ./run.sh kernel-judge artifacts "$PWD/artifacts/manual.gputrace"
```

## Implementation Details

Exact file line references are in [FILE_MAP.md](docs/FILE_MAP.md). The full source document was read and preserved in SOURCE_SPEC.md. User destination, toolchain and numerical requirements take precedence over that reference document.

| Language | Component | Reason |
| --- | --- | --- |
| TypeScript 7.1 | Trace builder | Typed graph recording |
| Rust | Validation, memory, backend and Tokio | Ownership and bounded admission |
| MSL | Integer kernels | Native GPU arithmetic and SIMD shuffles |
| C++ | Signpost bridge | Native Apple telemetry |
| Python | Independent integer oracle and judges | Arbitrary-precision arithmetic outside runtime |

This implementation was not directly grounded in published literature. It applies a defined fixed-point contract and established serialization, ownership and reduction patterns. Apple references: [64-bit signed type](https://developer.apple.com/documentation/metal/mtldatatype/long), [SIMD instructions](https://developer.apple.com/videos/play/tech-talks/10876/). Compilation and exact parity on the local M5 Pro gate hardware behavior.

### Resource ownership and bounds

Metal allocates `StorageModeShared` buffers directly. Page-rounded admission budgets are checked against actual `allocatedSize`. Exact-size buffers are pooled after command completion; there is no ring buffer or suballocation. Each persistent worker lane retains its own device/queue, bounded pipeline LRU and pool. Inputs are overwritten and outputs poisoned before use. Driver allocations are reported separately from the buffer cap.

Trace limit 64 KiB, dimensions 1–4096, M×K×N <=536870912, at most 32 external arrays totaling 4 MiB at runtime admission (single kernels use up to three). Queue capacity 1–32 plus three active worker lanes by default, optionally two. The existing 64 MiB resident and 16 MiB idle buffer budgets are divided across lanes; pipeline caches retain up to 8 entries per lane. CLI JSON limit 32 MiB; dispatch repeat limit 1000. Runtime lanes await completion callbacks asynchronously; CPU preparation and output reading run in at most one blocking task per lane. Full reference validation is test/feature gated; production checks every 1000th successful receipt in a bounded background Tokio worker. No application path calls Metal waitUntilCompleted. Synchronous judges wait on a bounded Rust callback channel.

The independent matrix reference uses i128 products, Euclidean division and accumulation. CORDIC uses a deterministic integer CPU reference plus independent analytic accuracy judges. The kernel's i64 bound is safe for full-range i32 and K<=4096. Validation remains enabled in caller wall times. Receipt construction copies results into Rust storage.

## Bounded asynchronous completion

`Runtime::new(capacity)` defaults to **3** in-flight slots. `Runtime::with_in_flight(capacity, limit)` accepts **2 or 3**. CLI runtime commands read `RUST_SERVER_IN_FLIGHT`; unsupported values fail before GPU work:

```sh
# src/main.rs — select three slots for the existing typed runtime.
RUST_SERVER_IN_FLIGHT=3 ./run.sh run artifacts/tanh/trace.amvk artifacts/tanh/inputs.json
```

A shared Tokio semaphore gates preparation/encoding across all worker lanes. The copied Metal completion block owns the permit and retained buffer handles until native completion. It then releases those resources and wakes a Tokio oneshot. Cancellation of a receipt receiver does not release or recycle committed buffers. Dropping Runtime closes admission and drains accepted jobs. A lane returns to its pool only after completion and result validation.

Receipts include `worker_id` and shared `flight` counters. Pool/cache counters are lane-local; device allocation reports cover the process's Metal device resources. Splitting the total buffer budget can reduce the maximum admitted single-job size; compiled pipelines may be duplicated across lanes. Driver/pipeline allocations remain outside the buffer cap. See [design, ownership and evidence](docs/ASYNC_COMPLETION.md).

## Testing Evidence

| File | Coverage |
| --- | --- |
| tests/async_completion.rs, src/completion.rs, src/gpu.rs | Semaphore caps, native callback ownership, canceled waiters, errors, queue drain, two/three-lane pressure |
| tests/cordic.rs | Circular/hyperbolic accuracy, axes/quadrants/tiny vectors, repeats, graph mutations, dynamic shapes and runtime recovery |
| tests/fixed_point.rs | Explicit widening, floor shifts, saturation, full-range cancellation, K up to 4096, SIMD boundaries, randomized exact parity and repeatability |
| tests/adversarial.rs | 10,000 malformed traces, graph/layout mutations, input rejection, tails, pressure, cancellation and producer concurrency |
| tests/resources.rs, src/backend.rs | Cache/pool ownership, eviction, budgets, recovery and mixed-family routing |
| ts/trace.test.ts | Five trace and handle validation tests, including version 3 |
| judges/benchmark.py | Independent arbitrary-precision Python comparison of every saved output |
| tests/serving.rs, tests/rapid_rpc.rs | Typed KV, integer codec, bounded iterations, panic/cancellation/error/pressure and hostile subprocess protocol |
| judges/model_ab.py | Separately recorded real Qwen regular/TurboQuant A/B and resumed-decode logit comparisons |
| judges/run.py | Codegen, Rust/TS gates, CLI, receipts, native signposts, GPU capture and source manifest |

The final ordered run passed **83 Rust tests in each normal/debug-validation posture, 5 TypeScript tests**, Clippy, formatting, both TypeScript 7.1 compiler gates, exact Python parity, 24 native signpost pairs, CLI rejection checks and GPU capture. At each in-flight setting, the 24-job runtime judge completed all 24 callbacks with zero leaked permits; observed peak outstanding command buffers were exactly 2 and 3. Run `python3 judges/run.py` for the complete suite. Current evidence is in `evidence/`; `judge-summary.json` and `source-sha256.json` bind it to source. Historical float evidence is preserved in `evidence-float-v1/` and does not qualify integer performance. The separate model diagnostic compares regular and TurboQuant prefix storage under MLX; no full integer-versus-MLX model comparison, sustained thermal qualification or integer speedup is claimed.

The independent analytic judge checked **6,643 outputs per CORDIC function**. Maximum observed error was below one raw Q1.31 unit in that saved workload; the separate edge/random Rust/Metal test reached one raw unit. All GPU values matched the deterministic integer reference exactly. These sampled results are not an exhaustive proof over every input pair.

## Was It Proven Necessary?

Integer storage and arithmetic are necessary for the user's requested contract. This does not establish model-quality equivalence or prove a throughput advantage over floating-point hardware.

## Deployment / Rollout

Local prototype only; no merge, deployment or existing service replacement. Documents remain draft with single-author review. Rollback must restore the numerical implementation and matching wire producer together. Per-job receipts and native signposts expose failures and timings. Native process crashes and GPU hangs still need stronger isolation.

## Next Stage of Implementation

Qualify the real model path on held-out quality/SLO workloads, implement tensor-batched hybrid state handling, then completion-safe prefix reuse, packed attention and serving transports. [Model integration](docs/MODEL_INTEGRATION.md) records the actual boundary; [accelerator roadmap](docs/ACCELERATOR_ROADMAP.md) covers AMX and ANE.

## What We Learned

The location of each fixed-point shift is part of the contract. Reordering exact additions is safe only while the accumulator bound holds. Shuffling both halves preserves signed 64-bit partials without shared scratch. Local measurements are required before claiming speed.

Known dependency limitation: `block 0.1.6` is flagged future-incompatible by Cargo; current compiler checks pass.

## Changelog

1.10.0 — 2026-10-22 — Persistent native weights, mixed affine quantization, attention/recurrent/MoE tensor operators and transactional recurrent batching; full model generation remains unqualified.

1.8.0 — 2026-10-22 — Real Qwen/Rapid integration, bounded token scheduling, typed KV, regular/TurboQuant diagnostic A/B, 83-test Rust gates and explicit production limits.

| Version | Date | Author | Change |
| --- | --- | --- | --- |
| 1.4.0 | 2026-10-22 | Codex | Semaphore-capped 2/3 command buffers, native callback-to-Tokio completion and owned worker lanes |
| 1.3.0 | 2026-10-21 | Codex | Integer baseline, explicit int64 operands and SIMD shuffle reduction; Q1.31 confirmed; circular/hyperbolic CORDIC wired and qualified |
| 1.1.0 | 2026-10-21 | Codex | Persistent resources and second kernel family |
| 1.0.0 | 2026-10-21 | Codex | Original float compiler-to-GPU proof |

## Threadgroup macro throughput experiment

[Design and measurement contract](docs/GRID_THROUGHPUT.md): explicit 64/128/256/512-thread groups at command-buffer caps 2/3, independent 1000/1500/2000-request Rust boundary windows, three repetitions. GEMM assigns each SIMD-group its own output column with guarded tails. Defaults remain unchanged. The original full-validation measurements are archived in `evidence-macro-validated-v1/`; the historical BENCH-MODE rerun uses deterministic background one-in-1000 sampling, described below. Run `cargo build --release --locked` then `python3 judges/macro.py`; historical benchmark measurements are archived in `evidence/macro-throughput.json`. These measurements describe the 128-cube kernel runtime, not model token throughput.

Measured 72 windows / 108,000 validated requests: [median rates and limits](docs/GRID_THROUGHPUT.md#observed-results). All requested widths passed; CPU reference validation dominated that historical boundary. Detailed latency and native completion counters are archived with each window.

Final verification for this experiment: **45 Rust tests and 5 TypeScript tests passed**, plus the complete ordered compiler, numerical, native completion, signpost, and Metal capture gates.

## Production receipts and GPU chains

Default in-flight cap is **3**. Full synchronous CPU Q1.31 validation is compiled into backend execution only under `cfg(test)` or `--features debug-validation`. Production receipts explicitly report `validation_performed: false`, `max_abs_error: null`, and `validation_ms: 0`.

Every 1000th successful runtime receipt is checked asynchronously by a bounded Tokio background worker. `Runtime::validation_stats()` exposes scheduled/passed/failed counts and the last error; call `flush_validation().await` after awaiting all receipts and before shutting down Tokio. A mismatch latches admission unhealthy. No sampled check is silently dropped. Returned receipts precede background-check completion.

`Runtime::submit_chain(ChainJob { traces, inputs })` accepts 1–16 stage traces. Each later stage's first input consumes the preceding output with exact shape matching; remaining inputs bind by external name. Stages encode in **one Metal command buffer and one compute encoder**, declaring read/write resource usage and explicit activation memory barriers. Existing GEMM+bias+ReLU remains fused. Intermediates stay in shared GPU buffers until completion.

CLI example after the standard judge generates fixture artifacts:

```sh
./run.sh run-chain artifacts/rectangular/inputs.json artifacts/rectangular/trace.amvk artifacts/sin/trace.amvk artifacts/cos/trace.amvk artifacts/tanh/trace.amvk
cargo test --locked --features debug-validation
```

[Design, limits, and verification](docs/PRODUCTION_PIPELINE.md). Previous full-validation macro evidence is retained in `evidence-macro-validated-v1/`; production macro rows explicitly record the sampling policy and drained sample counters. Sampling uses additional bounded host snapshots (3 queued, one active, up to one pending per lane); these are separate from the 64 MiB GPU allocation budget.

Historical kernel-chain-stage verification: **54 Rust tests passed in both production and debug-validation builds; 5 TypeScript tests passed**. Production macro rerun: 72 windows / 108,000 timed requests, 96 background checks passed. [Production results and limits](docs/PRODUCTION_PIPELINE.md#production-macro-results).

## Spec 15 cache controller

[Design and limits](docs/SPEC15_IMPLEMENTATION.md) implements exact 16-token prefix blocks joined to plan branches: SHA-256 identity isolation, immutable tail forks, reference/step/anchor holds, boundary-only leaf reclamation, epoch Q1.31 round-to-nearest-even decay, anchor budgets, continuation-first bounded admission, queue expiry, and separate unknown-output reservations.

Every cache constant is explicit configuration; `fixtures/cache-config-bench.json` is an **experimental test profile**, not production tuning. Payloads are opaque bytes supplied by `Controller::fulfill` through a prefill callback. These opaque controller blocks are not yet connected to the actual Qwen producer/consumer; the new Qwen backend owns its separate real model state. `Runtime::submit_cached` retains a cache step lease through native GPU completion; existing kernels do not consume these cache payloads as model KV.

```sh
./run.sh cache-judge fixtures/cache-config-bench.json
cargo test --locked --test cache_control
```

The CLI judge labels its payloads SYNTHETIC-NONMODEL. Cache telemetry reports production_qualified=false. Ready prefill tickets reserve bytes before computing; duplicate shared-prefix work queues behind its producer, and resident context never calls the producer again. Close branches, finish native steps, then commit boundaries; use one audit call per successive epoch. Unknown estimates remain separate from exact coverage and explicit capacity reservations.

**Sampling ruling:** proposed random-policy rates/thresholds remain inactive. `Runtime::with_sampling` accepts explicit deterministic bench configuration; `DynamicDraft` is refused. Receipts/counters label DETERMINISTIC-MODE and production_sampling_claims=false. Earlier fixed-cadence throughput results are BENCH-MODE observations, not approved production sampling claims. See [policy status](docs/SAMPLING_POLICY_STATUS.md).

Historical Spec 15-stage verification: **67 Rust tests passed in each normal/debug-validation build, plus 5 TypeScript tests and the complete ordered compiler/numerical/native gates**. Scope remains a tested cache control plane with opaque payloads, not a model-serving qualification.

## Proposed serving architecture and next delivery

[Serving architecture](docs/SERVING_ARCHITECTURE.md) specifies proposed tuning commands, an iteration scheduler, regular/TurboQuant KV interfaces, pressure and quality gates, and the ordered route to Apple-native model serving. The broader commands remain proposed; `model-run` and `config check|print` now implement a documented subset. The experimental model path below supersedes the earlier first-model milestone. Sampling draft values remain inactive.

## Historical external Qwen model integration — legacy feature only

The retained OptiQ A/B used the bounded Rust iteration scheduler and a private Rapid/MLX worker. This path now requires the explicit `legacy-python` feature and is excluded from default builds; commands below are historical interface examples. Current reproduction profiles target Stewart's selected fused Opus/Cyber checkpoint, so rerunning them would be a new experiment. These historical results do not qualify that fused checkpoint. Custom Metal kernels remain integer-only.

```sh
# src/main.rs — explicit local profile and bounded prompts; regular KV remains default.
./run.sh config check --config fixtures/qwen-serving.json
./run.sh model-run fixtures/qwen-serving.json fixtures/qwen-prompts.json 16
./run.sh model-run fixtures/qwen-serving.json fixtures/qwen-prompts.json 16 --kv-codec rapid-turboquant-k8v4
```

See [implementation, tuning flags and evidence](docs/MODEL_INTEGRATION.md) and [adversarial review](docs/MODEL_ADVERSARIAL_REVIEW.md). TurboQuant is prefix-storage compression with materialization before attention; the backend does not yet tensor-batch sequences. MLX native command-buffer counts are not capped by this adapter. Rust state/workspace reservations and measured MLX physical allocations are reported separately. Production sampling proposals remain inactive.

Next: broader quality/SLO qualification, tensor-batched hybrid decode, completion-safe recurrent/prefix reuse, packed attention and serving transports. Earlier roadmap sections describe stage history; they do not supersede this implementation's explicit limits.

The retained A/B loaded the OptiQ checkpoint without a Fable adapter. It does not qualify the selected fused checkpoint; see [checkpoint correction and next-round gates](docs/MODEL_INTEGRATION.md#checkpoint-correction--fabelcyber-qualification-pending). Stewart subsequently identified the intended workhorse as `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`.

## Native Rust MLX backend — current delivery

`Rust typed graph → bounded admission → native executor thread → C ABI → MLX C++ → Metal → CPU-owned typed receipt`. Default admission is three graphs. A joint completion ledger reserves host staging bytes, logical tensor bytes and graph nodes atomically; cancellation does not return those reservations before native completion. The backend supports dense F32 multiplication, affine Q4 multiplication, RMS normalization and raw I32/U32 elementwise buffers. This tensor interface remains separate from the complete hybrid generation executor described below.

```sh
# build.rs — native headers and libraries must be present; no Python build discovery.
MLX_ROOT=/absolute/path/to/native/mlx/sdk cargo build --release --locked
# src/main.rs — built-in typed tensor checks and macro throughput; no JSON arguments.
./target/release/rust-server native-mlx
./target/release/rust-server native-mlx bench
```

The locally staged SDK is `vendor/mlx` (ignored); `MLX_ROOT` overrides it. It contains native headers, `libmlx.dylib`, `libjaccl.dylib` and `mlx.metallib`. No Python extension is linked. The original SDK was copied from the installed MLX 0.32.4 distribution; this is reuse of the native SDK, not a rewrite of MLX's C++/Metal implementation in Rust.

Current model scope is limited to two targets: **Qwen3.6 Combined MoE is the workhorse**, at `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`; **abliterated Qwen3.8 27B is the only other integration target**. Workhorse model smoke tests use the fused checkpoint recorded in `fixtures/native-model.path`; Qwen3.8 qualification must identify its exact abliteration artifact separately. No additional model integrations are in scope. Current tensor tests and matrix benchmarks are synthetic and make no model-generation claims. [Design, comparison, limitations and next steps](docs/NATIVE_MLX_BACKEND.md).

1.9.0 — 2026-10-22 — Native Rust-owned MLX tensor executor and joint completion ledger; default Python adapter disabled; framework synthesis, 89-test Rust gates and selected fused checkpoint recorded. Qwen forward remains next-stage work.


## Persistent native weights and operator execution

The next native increment retains one immutable checkpoint store per executor across tensor graphs. Rust fingerprints checkpoint artifacts, rejects malformed metadata and cross-owner parameters, and uses exact per-module affine quantization (2/3/4/5/6/8 bits; groups 32/64/128). The selected Combined MoE checkpoint is the workhorse; no model substitution is permitted. Local standard checkpoint metadata is parsed in Rust; runtime tensors cross an in-process C ABI with no Python worker or JSON transport.

```sh
# src/main.rs — selected checkpoint fingerprint and persistent projection smoke only.
cargo run --release --locked -- native-mlx weights
```

Native operators cover RoPE, causal GQA attention, FP32 gated-delta recurrence, top-k routing, gathered quantized experts and weighted reduction. RecurrentBank provides bounded per-sequence CPU snapshots, exclusive row reservations, completion-only atomic commits and cancellation rollback. These operator/tensor building blocks now support the separate full-model executor below. Strict generation qualification remains failed. MTP presence is reported separately from execution. Logical parameter payload accounting is not a physical GPU residency measurement.

See [design and qualification gates](docs/NATIVE_WEIGHTS_EXECUTION.md). Existing custom Metal kernels remain integer-only Q1.31. Proposed production sampling rates remain inactive.

## Git checkpoint

The canonical project lives at `/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine` in the existing [GitHub repository](https://github.com/sbarteau2022/Rust-TS-Tokio-Inference-Engine). The former Desktop `RUST SERVER` path is a compatibility symlink to this checkout. The initial standalone commit is retained locally on `codex/standalone-checkpoint`; delivery continues on the existing repository history. Source, documentation, fixtures and retained verification evidence are tracked. Build outputs, installed dependencies, the local MLX SDK, GPU captures and interpreter caches are excluded. Model weights remain outside this repository. A Git checkpoint does not promote full-model generation or production readiness; the qualification limits above still apply. The existing `origin` remote is retained; model weights and the local MLX SDK are not uploaded.

## Complete native hybrid generation — qualification blocked

Combined now executes all 40 hybrid layers, including convolution/GDN recurrence, full attention with regular KV, sparse/shared MoE, embeddings and output head. Rust tokenization binds the checkpoint template hash; single-turn text with thinking disabled is supported. The bounded scheduler owns unified immutable KV/convolution/recurrent snapshots and drains its native worker on shutdown. There is no Python worker or JSON runtime transport.

```sh
# src/main.rs — selected Combined checkpoint only; native reference defaults to FP32.
cargo run --release --locked -- native-generate "Reply with exactly: READY" 8 --activation f32 --device gpu
# Fixed qualification suite; a failed gate intentionally exits nonzero.
cargo run --release --locked -- native-qualify
```

FP32 passes the preregistered chunk-versus-token continuation bound; BF16 failed it and its evidence is retained. The fixed held-out suite scores **15/16**, with one strict formatting failure (`No.` versus `no`); CPU reproduces that response. Expectations were not relaxed. **Full-model qualification remains blocked.** See [design, evidence and remaining gates](docs/NATIVE_QWEN_GENERATION.md).

Limits: context 512, chunks eight, three resident sequences/admitted native operations; serial row execution, CPU-owned state snapshots, regular KV only. MLX internal command-buffer counts and physical workspace are not hard-capped by Rust's logical reservations. No TurboQuant attention, MTP, vision, ANE, AMX acceleration or production throughput claim is established. Existing custom Metal kernels remain integer-only; this model reference explicitly uses floating activations.

1.11.0 — 2026-10-22 — Complete native Combined hybrid decoder, native tokenizer, unified state and scheduler integration; retain failed strict held-out qualification.

Custom drafting-head candidate: evaluate a checkpoint-bound hidden-state head with exact target verification and atomic KV/convolution/recurrent rollback; native MTP remains disabled. The design and unresolved intermediate-state requirement are in [the generation design](docs/NATIVE_QWEN_GENERATION.md). No custom head is implemented or qualified yet.

## Exact output choices — explicit decoding contract

Use `native-generate "QUESTION" 8 --choices yes,no` to require a canonical choice and EOS. Both answers remain available; target logits choose among legal next tokens. Choices are bounded, checked before admission, isolated per request and fingerprinted in receipts. Unsupported backends reject them. This controls generation itself; it does not lowercase or strip punctuation after generation.

`native-qualify` retains the original unconstrained 16-case gate. `native-qualify --contracts` is a separate profile: the unchanged borrowing case declares both yes/no choices, and four additional balanced yes/no cases check both semantic answers. The original failure is retained; constrained compliance is not evidence of improved unrestricted instruction following. See [design and results](docs/NATIVE_QWEN_GENERATION.md).

## Typed native model IR; tokenization at I/O

The native boundary module owns chat rendering, Rust tokenization, detokenization and CLI qualification. Core model execution receives validated prefill/decode IR and unified state; no inner text/tokenizer round trips occur. Rust compiles an immutable indexed hybrid layer program which the C ABI validates and executes. Combined's trained token IDs and numerical equations are preserved. This is typed execution/control IR, not an MLIR dialect, a byte-based replacement vocabulary or a demonstrated symplectic model.

The pre-IR constrained model smoke passed 20/20 while the raw profile remained 15/16. Those findings and their source/binary bindings are retained under evidence/native-choices. Post-IR regression and model results are recorded separately under evidence/native-ir. See [boundary design, invariants and findings](docs/NATIVE_QWEN_GENERATION.md).

1.13.0 — 2026-10-22 — Explicit per-request output choices; separate raw/constrained qualification; typed native model-step/layer IR and lexical I/O boundary.

Post-IR result: **20/20 constrained smoke**, with continuation and drain passing; reported outputs/metrics match the pre-IR run. Regression passed 113 default / 113 debug / 87 minimal-feature Rust tests, five TS tests and the existing compiler/Metal gates. The original unconstrained profile remains 15/16; broader model quality remains unqualified. [Recorded findings](evidence/native-ir/summary.json).

## Extended qualification gate

`cargo run --release --locked -- native-qualify --extended` runs 24 new authored Rust/security cases plus actual Combined context-512 continuation, overflow rejection and three-fork serial/concurrent isolation. Declared choices and unconstrained cases are reported separately. This is a local engineering gate; independent full-model reference logits and production/domain qualification remain missing. The original 16/20-case suites are preserved. See [preregistered design and measured outcome](docs/NATIVE_QWEN_GENERATION.md).

Historical extended result before the attention correction: **23/24; local model gate FAILED**. Context-512 chunk-versus-token continuation exceeds the fixed drift bounds (logits 0.05924, state 0.01250; limit 0.01). Overflow rejection, three-fork isolation and drain pass. Regression passes 114 default / 114 debug / 87 minimal Rust tests and TS/Metal/compiler gates. [Failure evidence](evidence/native-extended/summary.json). Next: identify the first divergent operator and establish independent reference evidence before device-state/batching optimization.


## Attention divergence diagnosis and correction

`cargo run --release --locked -- native-diagnose` opt-in traces five stages per
layer from the same context-504 snapshot, checks trace/untraced equivalence, and
compares attention with an independent scalar FP64 oracle on actual Q/K/V.
Normal execution allocates no trace and runs no CPU attention validation.

The first divergence was layer 3 attention. Both fused SDPA and explicit batched
FP32 matrix operations differed from the scalar oracle; single-query SDPA also
failed at short context. The accepted reference uses FP32 elementwise products,
stable softmax and reductions with explicit cached causal visibility. This
establishes observed dispatch-dependent numerical error, not its unverified
hardware-internal cause. Custom integer Metal kernels remain unchanged.

Context-512 logit NRMSE improves from **0.05924135 to 0.000000379**; state NRMSE
from **0.01249746 to 0.000000336**. Trace controls are unchanged, all traced
attention calls pass the 1e-5 scalar-oracle bound, and no layer-stage drift exceeds
1e-4. This is a correctness reference path with no throughput promotion.
Full-model independent reference and semantic qualification remain separate gates.
See [diagnosis and final qualification evidence](evidence/native-diagnose/EXPERIMENTS.md).

1.15.0 — 2026-10-22 — Bounded opt-in layer tracing, independent FP64 attention
oracle, adversarial trace validation and corrected FP32 attention reference.

Final unchanged-fixture results after correction: **20/20 constrained smoke**,
**23/24 extended domain cases**, **15/16 raw smoke**. Context continuation,
overflow rejection, three-fork isolation and drain pass. The cancellation-domain
answer remains wrong (`no` versus `yes`); the raw formatting failure remains.
Full-model qualification stays blocked. Regression: 114 default / 114 debug /
87 minimal Rust tests, five TS tests, Clippy, compiler/Metal gates and sanitizer
harnesses pass. Next: independently validate exact-checkpoint projections and
full-model logits, then resolve semantic qualification before optimizing attention,
device-resident state, batching or a drafting head.


## Independent sampled projection reference

`cargo run --release --locked -- native-reference` checks the unchanged r10
cancellation case with an independent scalar FP64 affine-dequantization/dot
product oracle. `--device cpu` repeats the diagnostic as a backend control.
The command exits nonzero while that semantic gate fails; it does not repair an
answer or replace the normal constrained-generation scheduler.

Each backend passes **871 projection probes** over ordinary projection rows,
sampled routed expert outputs at the first/last tail tokens, and both exact
`yes`/`no` LM-head channels. Maximum sampled NRMSE: GPU **7.75e-7**, CPU
**6.89e-7**, against a fixed 1e-4 limit. Checked and unchecked execution match.
Observed checkpoint projections here are 4/8-bit, group 64; a separate 55-case
sanitized harness covers 2/3/4/5/6/8-bit packing and groups 32/64/128.

The failure remains: GPU favors `no` by **0.108324** logits; CPU by **0.133287**.
The two final-head scalar checks have maximum absolute errors below 1.71e-6,
well below those margins. This rules out the sampled projection arithmetic as
the immediate cause, given the supplied hidden states. It does not establish
correctness of unsampled channels, checkpoint parsing, recurrent equations or
the complete model. Full-model qualification remains blocked. Production calls
perform no projection probes; custom integer Metal kernels are unchanged.

Next: independent checkpoint-byte ingestion and scalar recurrent/normalization/
RoPE checks to establish full-logit fidelity before attributing the remaining
failure to model quality or optimizing state, batching and drafting.
See [design and findings](docs/NATIVE_QWEN_GENERATION.md).

1.16.0 — 2026-10-22 — Opt-in sampled FP64 projection oracle, focused output-head
checks, CPU/GPU semantic diagnostics and adversarial interface qualification.

Verification for 1.16.0: **114 default / 114 debug / 87 minimal Rust tests**, five
TS tests, 55 sanitized scalar cases, 12,000 malformed FFI cases, Clippy and the
existing Metal/compiler gates pass. Context-512 parity remains unchanged. The
rerun extended gate remains **23/24**, with continuation, overflow, isolation and
drain passing. [Recorded results](evidence/native-reference/summary.json).


## Independent ingestion and complete operator diagnostics

`native-ingestion` compares every native-loaded tensor byte, shape and dtype
against checkpoint shard payloads. Combined passes **1,757 tensors / 19,508,787,456
bytes**. `native-reference --full` checks all projection channels (including every
active expert and the complete vocabulary head) and all prefill chunks of frozen
r10. Scalar FP64 checks cover embeddings, RMS, RoPE, convolution, recurrence,
attention, router/top-k, activations, gating, mixtures and residual additions.
These checks observe native operands; they are **not an independent model graph**.
They run only in explicit diagnostics, never on the serving hot path.

The expanded check exposed a real embedding defect: BF16 affine scales/biases
were rounded before the requested FP32 result. The FP32 profile now widens
scales/biases before dequantization. The failed real-model gate and a BF16 fixture
negative control are retained. Removing the correction makes that regression
fail. BF16 execution remains unqualified.

Corrected full-prompt diagnostic: **133,880 projection tiles** (maximum NRMSE
2.30e-6; limit 1e-4), **2,768 operator checks** (2.63e-7; limit 1e-5), and **40
attention checks** (2.88e-7; limit 1e-5) pass. Checked/unchecked execution matches;
chunk/single control error is 6.07e-7. The unchanged cancellation case still
chooses `no` over `yes` by **0.107872** logits. Numerical correction does not
establish semantic qualification. See [retained evidence](evidence/native-operators/EXPERIMENTS.md).

1.17.0 — 2026-10-22 — Full ingestion audit, complete composed FP64 operator
checks, bounded all-channel diagnostics and BF16 embedding arithmetic correction.


Final-source regression for 1.17.0: **116 default / 116 debug / 87 minimal Rust
tests**, five TS tests, 93 scalar projection/embedding cases, 43 scalar operator
cases, 14,000 malformed FFI descriptors, Clippy and Metal/compiler gates pass.
Combined emits `READY` then EOS. Unchanged model gates remain **20/20 contracted,
15/16 raw, 23/24 extended**; context-512 continuation, overflow, isolation and
drain pass. These are limited local results, not production qualification.


## Frozen-failure discriminators

The native baseline already has speculation and prefix reuse disabled and regular
FP32 KV. These are recorded as existing settings, not counted as ablations;
KV-only 16-bit native control is unsupported. The separately authorized stock
mlx-lm reference remains outside the Rust server. Its checkpoint-dtype lane has
unquantized BF16 KV; its widened-parameter lane has FP32 KV.

A stock-loader hazard was observed: glob-loading the separate MTP sidecar shifts
already-sanitized main norms and produces garbled text. The accepted reference
uses unchanged stock engine code on a symlink view of exactly the four indexed
main shards and the same boundary artifacts. The failed original-directory run
is retained. All 166 installed mlx-lm Python source files match package RECORD;
no Rapid/DSpark/DFlash modules are imported. Both prompts' template and token IDs
match the native boundary exactly.

Stock reproduces cancellation `No` and borrowing `No.` in both profiles. The
lowercase no-minus-yes cancellation margin is 0.125 at checkpoint dtypes, 0.082024
in stock FP32 and 0.107872 natively. These are thin margins, not evidence of a
wide behavioral preference. The raw capitalization failure remains separate.
Core drop/poll/owned-field probes agree, while an async-cleanup follow-up changes
answer with stock precision. See [experiment design and limitations](docs/NATIVE_DISCRIMINATORS.md).

`cargo run --release --locked --example native_discriminators` records both
original prompts' native logits, stable logprobs, raw greedy output and drain.
`--probes` runs separately labeled interpretation questions. This is an offline
diagnostic with finite/bounds checks and two scalar tests; it changes no server
configuration, fixture expectation, output contract or default decoding policy.


Real-prompt embedding A/B (isolated control build, same native diagnostic):
cancellation no-minus-yes moves **0.108324 → 0.107872**, a **-0.000452** delta;
raw answer stays `No`. Borrowing stays `No.`. This confirms wiring and a small
numerical effect, without establishing a semantic cure. Original fixture gates
remain unchanged. [Measurements and reproducible provenance](evidence/native-discriminators/EXPERIMENTS.md).

1.18.0 — 2026-10-22 — Native margin diagnostic, authorized external stock
reference, indexed-shard loader control, interpretation probes and isolated
real-prompt embedding A/B. Server runtime remains Rust-owned MLX.


## Frozen-suite stock reference

[Design and qualification scope](docs/NATIVE_SUITE_REFERENCE.md) cover all 60
frozen raw/contract/domain executions against unchanged stock mlx-lm, on the
same Combined checkpoint with F32 floating parameters. The comparison is an
offline experiment; the server remains native Rust/MLX, without a Python worker.
The first attempt caught an EOS policy discrepancy: native accepted an extra
terminator. Native now follows checkpoint top-level eos_token_id and explicit
generation overrides, with bounded parsing and malformed-policy rejection.
Original prompts, expectations and output choices remain unchanged.

Reproduce in a new scratch directory with
`cargo run --release --locked --example native_suite_reference -- NEW_SCRATCH`.
Then run `judges/suite_reference.py --scratch NEW_SCRATCH --view INDEX_ONLY_VIEW
--out NEW_EVIDENCE` in the authorized stock MLX environment. Use the exact
index-only symlink view documented in the prior discriminator evidence; the
runner rejects model/view/source drift and incomplete native captures.
Full first-answer vectors stay in scratch; their hashes and comparison aggregates
are retained. Original fixture quality gates and production readiness remain
separate from backend agreement. See evidence/native-suite-reference/.

1.19.0 — 2026-10-22 — Full frozen-suite native/stock capture and comparison;
checkpoint-defined EOS boundary, fail-closed reference provenance and pressure tests.


The completed comparison matches **60/60 exact native/stock trajectories** over
44 unique prompts. Both lanes retain **15/16 raw, 20/20 contracted and 23/24 domain**
quality. The EOS correction leaves all observed outputs/first logits unchanged.
First-answer logit NRMSE still reaches **0.06537** (6.54%); this is not numerical
graph equivalence. Next is layerwise drift localization before wider fidelity or
production claims. [Results and limitations](docs/NATIVE_SUITE_REFERENCE.md).


## Native/stock layer and state localization

The opt-in `native-layer-reference NEW_SCRATCH_DIRECTORY` diagnostic captures the
unchanged s10/r11 prompts, per-layer stages, unified state and logits at each
eight-token prefill boundary. It compares traced and ordinary native execution;
stock observation delegates to unchanged stock module bodies and checks its own
fresh-cache untouched control. This remains an offline experiment with no server
transport/default change. [Design and reproducible scope](docs/NATIVE_LAYER_REFERENCE.md).

1.20.0 — 2026-10-22 — Native/stock layer-state divergence localization and observer controls.

The layer/state experiment localizes the material drift to stock fused SDPA at
the first full-attention layer. On identical operands, the current native FP32
attention is closer to an independent FP64 oracle. An explicitly labeled stock
SDPA-only FP32 control matches all 60 native trajectories and reduces maximum
first-answer logit NRMSE from **0.06537 to 8.76e-7**. Native arithmetic is retained;
this is numerical-profile localization, not unchanged-stock numerical equality
or full-model quality promotion. Both known quality misses remain.

The next [device-state ownership contract](docs/DEVICE_STATE_OWNERSHIP.md) is now
designed, including immutable handles, worker reclamation, resident budgets and
cancellation/branch/teardown gates. The opt-in device-state executor and scheduler
A/B lane are implemented; snapshot serving remains the default. See the
[implementation and measurements](docs/DEVICE_STATE_IMPLEMENTATION.md).


## Completion-owned MLX device state and scheduler smoke

`QwenDeviceExecutor` retains evaluated immutable KV/convolution/recurrent arrays in
an owner/generation-checked native table. Rust forks share one lifetime token;
last-drop reclamation uses a separate nonblocking channel, and only the worker
calls native release. Logical resident slots/bytes are reserved before execution;
failed/cancelled publication retires its version. Shutdown drains native versions
and reports external versions it invalidated. Explicit snapshot export remains
available for offline A/B. This removes full-state staging from the device hot
path, not token/logit transfer or scalar finite checks.

The Combined preservation run covers all 60 frozen executions and 360 steps,
including all logits, layer stages and unified state: observed differences are
zero. Context-512 continuation, branching, pending-receipt cancellation,
slot/byte exhaustion, restart and live-reference teardown are pressure tested.
The original 15/16 raw and 23/24 domain quality misses remain visible.

Run the explicit experimental Rust scheduler smoke with
`cargo run --release --locked -- native-device-smoke NEW_DIRECTORY`.
It compares snapshot/device state through the same immutable weights and scheduler
at concurrency 1 and 3, with monotonic per-token publication timing. Six requests
per window and three authored prompts are a throughput smoke, not autonomous-agent
or network streaming qualification. Native rows remain serial; actual tensor
batching and physical MLX command-buffer limits are unimplemented. No drafting
is active, so speculative acceptance is not applicable. Frozen output equality,
copy counters, native/scheduler drains and original misses remain separate gates.

1.21.0 — 2026-10-22 — Checked native device state, Rust reclamation, explicit
scheduler A/B lane, token timing and full-model pressure/preservation evidence.

The first device implementation regressed (14.43 vs 16.45 aggregate tok/s at one
lane). Consolidating finite checks at the final completion boundary measures
**18.27 vs 16.43 tok/s** at one lane and **18.38 vs 16.68** at three lanes in a
paired six-request smoke. Device TTFT p50/p95 is **306/376 ms** (one lane) and
**978/980 ms** (three). These small fixed-order windows preserve trajectories but
are not a sustained benchmark or network streaming result. Both the regression
and improvement are retained; see the implementation document for full metrics.

The [September–October speculative execution research review](docs/SPECULATIVE_RESEARCH_2026_10.md)
proposes native draft/verify transactions, recurrent-state-safe commit, and separate
exact-prefetch versus approximate sparse-cache experiments. These are design proposals;
no speculation, MTP, page prediction, early exit or new tuning flags are activated.


## Experimental native greedy speculative verifier

The [native verifier implementation](docs/NATIVE_SPECULATIVE_VERIFIER.md) adds
per-position target logits, private Rust cursors and atomic KV/convolution/recurrent
commit using bounded rejected-prefix replay. A request-local suffix drafter exercises
the interface; no trained MTP, production scheduler integration, stochastic sampling,
paging or early exit is activated. Use the explicit `native_speculative_smoke` example.

In the Combined 25-request preservation smoke, every generated trajectory agrees
with its sequential reference. Four authored 64-token code/security probes accept
61/241 proposals (25.31%) and measure 20.05 vs 20.34 decode tok/s: no demonstrated
speedup. These are fixed-order offline probes, not network or autonomous-agent
throughput, completed-task quality, or model qualification. Existing default serving
and original held-out misses remain unchanged.

1.22.0 — 2026-10-22 — Research-grounded opt-in native block verification, Rust
hybrid-state transactions and retained negative-performance retrieval smoke.


## Spec 15 proposer admission and native completion

[Design and measured evidence](docs/SPEC15_DRAFT_ADMISSION.md): retrieval and future MTP/custom plugins share the existing automaton's bounded admission, exact identity, measured cost gate and fixed-point epoch trust decay. Qualification trials reserve capacity before work starts; native commands retain leases independently of cancelled waiters. Failed bonus allocation charges both admitted target calls and publishes zero tokens. Configuration is explicit and opt-in; no trained MTP or serving default is enabled.

On the Combined checkpoint, four cap64 probes preserve all 256 output IDs. Decode-only baseline **20.11 tok/s** versus automaton **20.21 tok/s** is a +0.50% single-window difference, **not an established speedup**. Acceptance is **61/241 (25.31%)**, with **233 target calls / 256 committed tokens (1.0987 tokens/call)**. Cache declared-context coverage remains a separate metric. The empirical gate ended profitable on this short sample; sustained alternating cost/quality windows must precede promotion. TTFT, client inter-token latency, multi-lane streams and completed agent throughput remain unmeasured here.

1.23.0 — 2026-10-22 — Spec 15 proposer arbitration/admission, bounded qualification and probes, epoch trust decay, native completion guards, failure-inclusive target-call accounting, pressure tests and Combined evidence.


Spec 15 phase validation: **136 default / 136 debug-validation / 96 minimal-feature Rust tests**, 10,000 audit cycles, TS 7.1/tsrs and native numerical/pressure gates passed. Default preservation remains bitwise over 60 cases / 360 steps; speculative replay is numerically qualified and does not claim bitwise equivalence. Original quality misses remain unchanged. Source, binaries, configuration and raw results are bound in [the evidence manifest](evidence/spec15-drafts/manifest.sha256).

## Native MTP artifact and tuning plan

The Combined checkpoint contains a 42-tensor mixed-quantization MTP sidecar. Native execution and stateful admission are described in the later sections below. [The tuning plan](docs/NATIVE_TUNING_PLAN.md) records actual CLI controls, proposed unified native configuration, integration order and matched Dspark(2) qualification. The 25.31% result belongs to suffix retrieval, not this MTP head.

Native sidecar ingestion is now available with `cargo run --release --locked -- native-mtp-inspect`. It validates the supported layout and mixed packing, then opens a separate MLX store. This is ingestion only; MTP forward execution and acceptance remain unqualified.

1.24.0 — Native MTP sidecar ingestion: strict supported layout, mixed quantization validation, isolated MLX store and explicit ingestion-only CLI. MTP forward execution remains pending.

MTP ingestion checks passed: **141 default / 141 debug-validation / 96 minimal-feature Rust tests**, strict Clippy/format, TS 7.1/tsrs and the ordered native/kernel judges. [Bound evidence](evidence/mtp-inventory/manifest.sha256) records the real sidecar load; no new throughput result is claimed.

## Device-owned MTP hidden boundary

The opt-in `QwenDeviceExecutor::hidden_block` captures post-normalized activations with the exact target state version. Hidden bytes share the state reservation and completion lifetime; ordinary execution captures none. `hidden_snapshot` is a diagnostic export, not an internal transport. The [native tuning design](docs/NATIVE_TUNING_PLAN.md) records ownership and qualification; the later MTP operator and admission sections describe the current execution path.

Hidden boundary qualification: 1/2/4/8-token matched all-position logits and unified states are bitwise preserved; independent scalar post-norm relative L2 ≤ 7.66e-8. The default frozen-suite gate retains bitwise equality across 60 cases / 360 steps, with existing quality scores unchanged (15/16 raw, 20/20 constrained, 23/24 domain). No MTP draft or speedup is claimed.

1.25.0 — Device-owned opt-in post-norm payload, atomic state/activation lifetime and byte accounting, diagnostic snapshot, scalar normalization and bitwise preservation gates. All 141 default / 141 debug-validation / 96 minimal-feature Rust tests and ordered TS/native/kernel judges passed; context-512 device pressure/recovery passed. [Bound evidence](evidence/native-hidden/manifest.sha256).

## Native mixed-quantization MTP operator

Explicit `QwenDeviceExecutor::open_mtp` now loads the existing sidecar and enables shifted-token prefill and autoregressive draft steps. BF16 fusion/router weights use bounded FP32 matrix-vector dispatch; attention/expert projections retain per-module affine packing. Immutable draft KV/hidden states share target admission budgets but have separate owner IDs and retirement. Scalar CPU probes require `debug-validation`; ordinary startup leaves MTP disabled. This operator is not yet connected to Spec 15 serving or qualified for throughput.

The existing paired Dspark artifact is also inventoried: six draft layers, eight intermediate target taps, learned Markov correction and confidence head. It is a distinct future Spec 15 proposer, not the MTP sidecar or an already wired native backend. [Integration contract and local provenance](docs/NATIVE_TUNING_PLAN.md#existing-dspark-proposer-inventory--2026-10-23) record the required taps and verification gates.

Experimental `mtp_start` / `mtp_decode` now connect the native MTP operator to target greedy verification and authoritative state replay. The real-text gate is `cargo run --release --locked --example native_mtp_decode_smoke -- NEW_DIRECTORY`. This correctness profile excludes known seeds from learned acceptance and counts replay work; it is not yet a scheduler integration or throughput-qualified serving configuration.

1.27.0 — Native mixed MTP projections and immutable draft state, plus a conservative target-verified transactional decoder. The first real-text sequence-completion gate passes widths 1/2/4/8 against sequential greedy output and unified-state checks. It is a narrow correctness smoke, not held-out quality or performance qualification. Full acceptance still pays authoritative replay.

Qualification evidence: [native MTP manifest](evidence/native-mtp/manifest.sha256) binds the operator gates, real-text transactional smoke, default frozen-suite preservation, pressure results and current-source ordered judge. The final judge passed **144 default / 144 debug-validation** Rust tests, strict compiler/Clippy gates, TS 7.1/tsrs, native/kernel checks and GPU capture. Minimal-feature coverage remains 96 tests. Existing held-out model misses remain unresolved.

Accepted-window optimization: experimental MTP decoding now retains a fully accepted target block and synchronizes draft state from its authoritative hidden rows; rejected blocks still replay from the committed parent. Command counts and reused/replayed/forwarded token counts are explicit. The unconditional-replay reference remains available only with `debug-validation`. The focused same-parent replay comparison passes; final hardening evidence is recorded below.

First warmed Rust-boundary pilot: **33.97 tok/s MTP width 4 versus 20.35 target-only (+66.9%)**, with **144/170 learned tokens accepted (84.7%)** and identical output IDs. Four prompts × two reversed-order rounds yielded 256 output tokens per profile. Mean first receipt was slower: **803 ms versus 656 ms**. This is a short, single-lane pilot without HTTP streaming or continuous batching; it is not a matched Dspark result or production promotion. See [measurement scope and reproduction](docs/NATIVE_TUNING_PLAN.md#first-warmed-native-mtp-pilot--observed-2026-10-23).

1.28.0 — Fully accepted target-window reuse, bounded total output receipts, debug replay reference and alternating native-boundary MTP pilot. Rejected unified states still replay from the committed parent.

Accepted-window qualification: same-parent debug replay and sequential-target gates passed widths 1/2/4/8, including output-budget and EOS refusals. The final ordered judge passed **144 default / 144 debug-validation / 96 minimal-feature** Rust tests plus strict Clippy/compiler gates, TS 7.1/tsrs and native/GPU judges. [Bound reuse/pilot evidence](evidence/native-mtp-reuse/manifest.sha256) includes raw timings and the exact experimental scope. Spec 15 MTP admission, streaming latency and sustained multi-lane qualification remain unfinished.

Stateful MTP admission is implemented through `AutomatonMtp` under the existing Spec 15 controller. Seed and learned-token accounting are separate; proposal/verification/synchronization commands retain the admission lease until native completion. Baseline fallback explicitly maintains draft state and charges that cost under a separate profile identity. The real-model cancellation/pressure gate passes; prior pilot timings are not measurements of this adapter.

`cargo run --release --locked --example native_mtp_automaton_smoke -- NEW_DIRECTORY` exercises the real Combined model under `AutomatonMtp`. The focused gate passed exact target-token/state comparison, separate seed accounting, canceled receiver drain, bounded queue fallback and a forced post-verifier draft-sync allocation failure. That failed transaction charged its admitted target command and published zero progress; the parent recovered and shutdown drained to zero bytes. No adapter throughput or production qualification is claimed.

1.29.0 — Stateful Spec 15 MTP admission with known-prefix accounting, opaque prepared branches, command-owned completion guards, failure-inclusive target calls and maintained-state baseline fallback.

Stateful admission qualification: **146 default / 146 debug-validation / 98 minimal-feature** Rust tests passed, together with strict compiler/Clippy gates, TS 7.1/tsrs and native/GPU judges. [Bound admission evidence](evidence/native-mtp-admission/manifest.sha256) records current source, binary, raw controls and exact scope. Actual server-scheduler wiring, streaming, sustained multi-lane qualification and adapter performance measurement remain open.
