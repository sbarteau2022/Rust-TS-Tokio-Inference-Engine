---
title: "Threadgroup mapping and Rust boundary macro throughput"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-22T05:32:43Z"
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
    lines: "1-29"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-55"
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
    lines: "1-167"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-60"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-106"
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
    lines: "1-433"
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
    lines: "1-91"
    role: "primary"
  - path: "src/serving/model_ir.rs"
    lines: "1-173"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_extended.rs"
    lines: "1-214"
    role: "primary"
  - path: "src/serving/native_mlx.rs"
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-636"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-123"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-483"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-632"
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
related_documents: ["ASYNC_COMPLETION.md", "FIXED_POINT.md"]
tags: [metal, throughput, benchmark]
---
# Threadgroup and Rust boundary throughput design

Historical validation-inclusive measurement: original results below are preserved in `../evidence-macro-validated-v1/`. The current production policy and rerun are documented in PRODUCTION_PIPELINE.md.

Status: draft; measurement implementation and local validation.

## Contract
Measure 64/128/256/512 threads per threadgroup at in-flight limits 2 and 3. Independent drained windows contain 1000, 1500, or 2000 requests. Three repetitions per configuration. Preserve default dispatch sizes outside this explicit experiment.

GEMM maps each 32-thread SIMD-group to a distinct output column, with a uniform tail guard before reduction. Threadgroup width comes from Metal's threads_per_threadgroup attribute; no cross-SIMD accumulation or threadgroup scratch is introduced. Both synchronous and asynchronous backends use one grid calculation.

The macro timer begins before the first request clone/admission and ends after the last validated Rust receipt. A bounded FIFO of 32 outstanding receipts keeps admission bounded. Input copying, scheduling, GPU completion, exact integer CPU reference validation, output copying, and receipt delivery are included. Compilation and 32 warmup requests are excluded; JSON reporting is excluded. GPU timestamps are reported separately. Requests/s = window count / elapsed seconds; GEMM integer MAC/s = requests/s * M*K*N. These are application measurements, not model token throughput or peak hardware ratings.

## Validation
All six kernel families must match the integer oracle at every width, including nonmultiple output tails. Invalid widths fail before worker creation. Every measured receipt must succeed with zero raw-word error and a warm pipeline. Final acquired/released and submitted/completed counts must balance. Existing adversarial suite remains required. Fixed seed and identical workload per comparison; report all repetitions and avoid promoting a default from a single fastest sample.

## Scope and next steps
Primary macro workload is the existing 128x128x128 GEMM fixture. All-family edge tests qualify dispatch mapping separately. Thermal state, competing host work, and CPU oracle overhead limit extrapolation to production inference. Follow with representative model graphs and sustained thermal tests before choosing a default.

## Problem and solution
A larger threadgroup would previously produce duplicate GEMM writers because every SIMD-group shared its output coordinate. SIMD-group index now selects independent columns, retaining deterministic 64-bit reduction. The explicit experiment option leaves normal defaults intact.

## Implementation and configuration
Rust owns bounded admission, timers, and receipt statistics; Metal performs integer compute; Python orchestrates and archives repeated runs. See FILE_MAP.md for exact source references. `Runtime::with_grid(32, limit, Some(width))` selects the experiment; `Config.threadgroup_threads` controls both backend paths. CLI: `rust-server macro-bench artifacts 128 3 1500`. Existing default widths remain GEMM 32 and other kernels 256.

## Grounding
The implementation uses Metal's `simdgroup_index_in_threadgroup` and `threads_per_threadgroup` kernel attributes, with compiler validation on the local Metal SDK. Native command completion and resource lifetime follow ASYNC_COMPLETION.md. Rate arithmetic uses measured monotonic elapsed time; integer MAC counts are algorithmic operations, not floating point operations.

## Adversarial review
Reviewed duplicate writers, uniform SIMD tail branches, illegal widths, warmup contamination, unbounded admission, lost completions, and conflation of GPU timestamps with full receipt throughput. Both grid calculations were consolidated. A Metal compilation failure initially exposed inconsistent uint2/uint3 attribute vector sizes; both are now uint3. Each timed request checks exact output parity, cache warmth, and successful completion. Remaining limits: one GEMM macro shape, FIFO benchmark collection, three repeats without thermal isolation, and unchanged per-lane cache duplication.

## Reproduction and rollout
Run `cargo build --release --locked`, `python3 judges/macro.py`, and `python3 judges/run.py`. The macro runner writes evidence after each window and only marks complete after all 72 windows. No deployment or default promotion is performed. This local implementation remains draft pending independent review.

## Necessity and next stage
The grid change is necessary to measure larger GEMM threadgroups without changing numerical semantics. Next qualify model-scale workloads, isolate CPU oracle cost in a separately labeled performance mode, and run sustained thermal windows before default selection.

## Observed results

All 72 windows passed, totaling 108,000 timed and 2304 warmup requests. Median requests/s across three repetitions:

| Threads | In flight | 1000 req | 1500 req | 2000 req |
| --- | --- | --- | --- | --- |
| 64 | 2 | 808.1 | 792.7 | 800.3 |
| 64 | 3 | 1137.9 | 1157.0 | 1168.7 |
| 128 | 2 | 785.6 | 784.7 | 800.6 |
| 128 | 3 | 1184.0 | 1110.2 | 1132.7 |
| 256 | 2 | 778.2 | 775.2 | 778.1 |
| 256 | 3 | 1105.7 | 1179.7 | 1104.2 |
| 512 | 2 | 781.3 | 787.8 | 771.9 |
| 512 | 3 | 1183.0 | 1106.1 | 1150.6 |

GPU median per window ranged 0.0389–0.1130 ms; CPU reference validation medians ranged 1.799–1.916 ms. Validation dominates the measured boundary. No consistent grid-width winner is promoted. Cap 3 provides more CPU validation concurrency as well as permitting more GPU submissions; the throughput difference cannot be attributed solely to GPU overlap. Exact byte parity, warm pipelines, zero completion errors, and balanced final native counters passed in every window. Latency percentiles, run ranges, actual native peaks, and binary SHA256 are archived in macro-throughput.json and macro-summary.json.

## Final verification

The complete ordered judge passed: 45 Rust tests, 5 TypeScript tests, strict Clippy, formatting, TypeScript 7.1 gates, integer and analytic numerical checks, native completion/signpost checks, and Metal capture. The two new grid tests exercise all requested widths across seven fixtures / six families and reject invalid widths before worker creation. Benchmark executable SHA256 matches the final validated release binary.

## Retrospective

In this workload, increasing the runtime lane cap changes CPU oracle concurrency, while group width has small, inconsistent effects on macro rate. GPU-only timing is insufficient to predict validated request throughput. The old single-SIMD coordinate mapping had to change before larger-group measurements were valid.

## Changelog

1.0.0 — Local design, guarded multi-SIMD dispatch, bounded Rust boundary harness, repeated measurements, and adversarial review.
