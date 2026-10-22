---
title: "Deterministic integer kernel contract"
document_type: "implementation"
version: "1.1.0"
date: "2026-10-22T00:00:00Z"
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
    lines: "1-60"
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
    lines: "1-206"
    role: "test"
  - path: "native/attention_reference.h"
    lines: "1-21"
    role: "primary"
  - path: "native/mlx_bridge.cpp"
    lines: "1-178"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-67"
    role: "primary"
  - path: "native/operator_probe.inc"
    lines: "1-28"
    role: "primary"
  - path: "native/operator_reference.h"
    lines: "1-58"
    role: "primary"
  - path: "native/projection_probe.inc"
    lines: "1-22"
    role: "primary"
  - path: "native/projection_reference.h"
    lines: "1-30"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-160"
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
    lines: "1-447"
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
    lines: "1-99"
    role: "primary"
  - path: "src/serving/model_ir.rs"
    lines: "1-244"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
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
  - path: "src/serving/native_mlx.rs"
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-662"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-123"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-566"
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
related_documents: ["DESIGN.md", "NEXT_STEPS.md"]
tags: [fixed-point, metal, deterministic]
---
# Executive Summary

We replaced the provisional Q16.16 format with the user's confirmed Q1.31 contract and added circular and hyperbolic CORDIC operations. Inputs and outputs stay raw signed 32-bit integers. Metal uses integer instructions and returns repeatable results through the existing runtime.

# Problem Statement

The previous scale did not match the confirmed format. The server also lacked deterministic circular and hyperbolic functions.

# Solution Overview

All six graph families share version-3 traces, checked SSA, cached Metal pipelines, shared buffer ownership, validation and Tokio receipts. CORDIC adds sin, cos, atan2 and tanh. The existing SIMD shuffle dot reduction is retained.

# Q1.31 and CORDIC design (supersedes Q16.16 numerical details)

The user confirmed signed 32-bit storage and 31 fractional bits. Value = raw/2^31, range [-1,1). Positive one saturates to INT32_MAX. New traces use version 3; versions 1 and 2 are rejected. Every multiply casts both operands to int64_t before multiplication, floor-shifts each product by 31, sums in int64_t, adds bias, applies ReLU and saturates at the output. K<=4096 bounds the accumulator by 2^43 plus bias. The 32-lane shuffle reduction remains unchanged.

CORDIC adds four explicit graph operations: sin, cos, atan2 and tanh, using the same trace, SSA, backend, shared pool, cache, runtime and receipt path. Sin and cos use one input; atan2 takes Y then X; tanh uses one input. Each returns one raw i32 value per element. Sin/cos input and atan2 output are normalized radians/pi: raw INT32_MIN means -pi, 2^30 means pi/2; +pi is canonically -pi. Atan2(0,0)=0. Atan2 inputs are ordinary Q1.31 coordinates. Tanh inputs are natural dimensionless Q1.31 values in [-1,1), entirely within the hyperbolic convergence range; this does not support arbitrary-magnitude activations without a future input scale contract.

Internal CORDIC state uses signed int64_t with 60 fractional bits. Committed constants are generated with 90-digit Decimal arithmetic, no platform libm. Circular rotation/vectoring uses 48 iterations, quadrant reduction and exact axis cases; the circular gain is compensated at initialization. Hyperbolic rotation uses shifts 1..47 with repeated iterations 4,13,40. It returns tanh through 31 steps of unsigned-magnitude restoring division, truncating the ratio toward zero. Every GPU shift explicitly sign-extends; no float arithmetic, transcendental intrinsics, or 128-bit GPU arithmetic occurs. Circular output conversion floors to Q1.31 then saturates. CORDIC transcendental values are approximations: CPU/GPU outputs must match exactly; independent libm accuracy gates are separate and do not assert mathematical exactness.

All CORDIC lanes are independent (256-thread groups with output guards); the SIMD reduction applies only to dot products. Atan2 rotations have gain below 1.647; full-range two-coordinate Q60 magnitude stays below 2^62. Hyperbolic internal magnitudes for |z|<=1 stay below 2^61. Restoring division doubles a remainder smaller than the positive denominator and remains within int64. Nonzero tiny atan2 vectors are first normalized into Q60 rather than shifted in raw i32 precision.

## Verification requirements

Exact CPU/GPU parity, repeated bytes, full-range boundaries, quadrant/axis conventions, tiny atan2 vectors, saturation, negative floor shifts, tails, malformed/unsupported op and wire versions, cache separation, runtime pressure/recovery, deterministic table regeneration, dense/random libm error gates and monotonic tanh. Existing end-to-end compiler, receipt, capture and memory gates remain required.

## Alternatives and limits

64-bit storage was rejected by the user's clarification. LUT float evaluation and shader floating arithmetic were rejected by the numerical requirement. CORDIC is a correctness baseline; no speedup or complete model inference coverage is claimed. Input scale calibration, full activation ranges, async GPU completion and workload-specific grids follow qualification.


# Implementation Details

Exact file lines are indexed in FILE_MAP.md and related_files above. src/cordic.rs contains integer CPU and emitted MSL implementations; src/cordic_tables.rs holds generated constants. src/ir.rs and src/wire.rs lower and validate CORDIC edges. The schema and generated bindings add a Cordic union operation with function codes 0=sin, 1=cos, 2=atan2, 3=tanh. Unary operations require b=a; atan2 requires two equal-shaped inputs, Y then X. SSA IDs and names must not alias outputs. Unsupported functions and mixed graph shapes fail closed.

| Language | Component | Reason |
| --- | --- | --- |
| Rust | Reference, lowering and backend | Existing ownership and runtime |
| MSL | Integer arithmetic and CORDIC | Native GPU execution |
| TypeScript 7.1 | Typed recording | Same wire producer and compiler gates |
| Python | Decimal tables and analytic judges | Reproducible constants and independent accuracy checks |

## Literature grounding

The implementation applies the established shift/add CORDIC family, with these foundational references (IEEE citation format). No published throughput result is transferred to this implementation.

[1] J. E. Volder, “The CORDIC Trigonometric Computing Technique,” IRE Transactions on Electronic Computers, EC-8(3), pp. 330–334, 1959. DOI: [10.1109/TEC.1959.5222693](https://doi.org/10.1109/TEC.1959.5222693). [Open paper copy](https://www.ece.ucdavis.edu/~vojin/CLASSES/EPFL/Papers/3-Volder_CORDIC.pdf).

[2] J. S. Walther, “A unified algorithm for elementary functions,” AFIPS Spring Joint Computer Conference, pp. 379–385, 1971. DOI: [10.1145/1478786.1478840](https://doi.org/10.1145/1478786.1478840). Publisher access may require subscription. The exact numerical choices here are defined by this design and local judges, not a claim of verbatim implementation of that paper.

Apple API references: [64-bit signed Metal type](https://developer.apple.com/documentation/metal/mtldatatype/long) and [SIMD instructions](https://developer.apple.com/videos/play/tech-talks/10876/). Hardware support is gated by live compilation and dispatch.

## Configuration changes

Dtype q1_31_i32, layout row_major_q1_31, fractional bits 31, wire version 3, six registered kernel families. Versions 1 and 2 fail closed. JSON remains raw i32 arrays; no implicit float conversion. Constants have 60 fractional bits and are regenerated with `python3 judges/generate_cordic.py --check`. No new dependency or port is introduced.

# Testing Evidence

The numerical milestone passed 34 Rust tests and 5 TypeScript tests; the subsequent async extension brings the current full suite to 43 Rust tests and 5 TypeScript tests, with strict Clippy, formatting, tsrs and TypeScript 7.1, deterministic bindings/constants, independent Python exact GEMM reference, analytic CORDIC accuracy, native signposts/capture, CLI and source hashes. Final evidence: evidence/judge-summary.json and evidence/source-sha256.json. Each saved CORDIC workload has 6,643 outputs; independent Python libm maximum observed error is below one raw unit for every function. The edge/random Rust/Metal workload measured at most one raw unit. GPU and integer CPU outputs matched exactly. Dynamic shapes cover 1,31,32,33,255,256,257 elements and a 7x13 grid; atan2 tiny-vector accuracy and 8,193-point tanh monotonicity also passed. The accuracy gate is 2.01 raw units, distinct from the zero-error bit-parity gate.

| File | Type | Coverage |
| --- | --- | --- |
| tests/cordic.rs | Rust/Metal | Five tests: full-range/random GPU parity, quadrants/axes/tiny vectors, tanh monotonicity, graph corruption, runtime recovery, tails and pool reuse |
| tests/fixed_point.rs | Rust/Metal | Four tests: widening, negative floor shifts, per-product scaling, signed shuffle reduction, saturation, K boundaries and repeatability |
| tests/adversarial.rs | Rust/runtime | 13 tests, including 10,000 malformed traces, pressure, cancellation and rejection |
| tests/resources.rs + src/backend.rs | Rust/Metal | 12 tests for allocation/cache budgets and recovery |
| ts/trace.test.ts | TypeScript | Five tests including four function encodings and invalid functions/handles |
| judges/cordic.py | Python | Independent analytic error on 6,643 saved outputs per function |

## Reproduce

Run `python3 judges/run.py` from the project root. The runner fails on any gate and saves stdout/stderr. evidence/cordic-accuracy.json records libm errors; evidence/adversarial-tests.log records bit-exact GPU comparisons and boundary tests. evidence/source-sha256.json binds source to the completed run.

## Was it proven necessary?

The scale migration and integer CORDIC are necessary to meet the user's requested contract. Tests establish deterministic local execution and sampled accuracy, not a throughput advantage, full input-space proof, cross-device proof or model-quality equivalence.

# Adversarial Review

Single-author review checks casts before multiplication, explicit sign extension, full-width shuffles, no saturation before dot reduction, safe i64 bounds, all lanes participating, normalized tiny vectors, quadrant wrap, zero vector convention, exact cardinal values, hyperbolic repeated iterations and positive denominator in restoring division. Every input i32 is inside the tanh convergence contract. Input shapes, IDs, names, function codes and wire versions are validated before GPU submission. Two independent checks distinguish bit parity from approximation quality.

No 128-bit GPU multiplication, float fallback or transcendental MSL intrinsic is present. Source assertions and live compilation establish the programmed path; no instruction disassembly or guaranteed ALU throughput is claimed. Constants are generated once outside the runtime and checked for drift.

# Deployment / Rollout

Local prototype only. No commit, merge, service replacement or deployment. Documents remain draft pending independent review. Rollback requires source and producer/fixtures to move together to the previous wire contract. Native signposts, typed receipts and validation remain enabled. Async completion and hard GPU-hang isolation remain later work.

# Next Stage of Implementation

1. Async command completion is now implemented; see ASYNC_COMPLETION.md.
2. Compare kernel grids and reduction variants under identical integer contracts.
3. Define calibrated per-model scales and full activation ranges; validate output quality before model serving.
4. Qualify CPU AMX and ANE capability lanes separately.

This implementation sets up deterministic Apple GPU operators and dynamic shapes for the broader server goal; it does not complete model serving.

# What We Learned

Q1.31 cannot represent +1. Tiny vectors require normalization before CORDIC shifts. Internal guard bits and repeated hyperbolic iterations matter for accuracy. Exact CPU/GPU agreement alone can preserve a shared algorithmic error, so analytic judges are required. Fixed-point determinism does not imply higher throughput.

# Changelog

| Version | Date | Author | Change |
| --- | --- | --- | --- |
| 1.1.0 | 2026-10-21 | Codex | Confirm Q1.31, wire four CORDIC functions, define units and qualification |
| 1.0.0 | 2026-10-21 | Codex | Provisional Q16.16 and shuffle reduction, superseded numerically |
