---
title: "Native Rust MLX tensor backend"
document_type: "architecture"
version: "1.0.0"
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
    lines: "1-45"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-62"
    role: "primary"
  - path: "examples/native_device_pressure.rs"
    lines: "1-184"
    role: "primary"
  - path: "examples/native_device_reference.rs"
    lines: "1-162"
    role: "primary"
  - path: "examples/native_discriminators.rs"
    lines: "1-145"
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
  - path: "judges/device_abi.cpp"
    lines: "1-39"
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
    lines: "1-218"
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
    lines: "1-85"
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
  - path: "native/qwen_device.inc"
    lines: "1-45"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-171"
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
    lines: "1-451"
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
    lines: "1-104"
    role: "primary"
  - path: "src/serving/model_ir.rs"
    lines: "1-244"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-546"
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
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-674"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
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
related_documents: ["MODEL_INTEGRATION.md", "NEXT_STEPS.md", "../README.md"]
tags: [rust, mlx, ffi, native, design]
---
# Executive Summary

The next backend runs in the Rust server process. Rust builds typed tensor graphs and owns bounded admission and completion; a small compiled C++ bridge invokes Apple's native MLX library. No Python interpreter, child process, JSON transport or JSON configuration is used by this backend.

# Problem Statement

The earlier Rapid worker was an external numerical baseline. It did not implement the intended native backend, and its checkpoint comparison did not qualify the Fabel/Cyber model. Native tensor execution must be established before model integration and performance promotion.

# Solution Overview — design before implementation

Use a dedicated Rust-owned native executor thread. Submit owned typed graphs through a bounded Tokio channel. A semaphore of three bounds admitted graphs; a permit travels with the command until native evaluation and CPU-owned output copying finish. Cancellation of a waiting caller cannot recycle a permit or input buffers early. Shutdown closes admission and joins after queued work drains. Native exceptions are converted to bounded error strings; a native execution failure latches the executor closed.

The C ABI holds no Rust objects. A per-command native graph owns MLX arrays; Rust node IDs are validated against prior nodes, dtype and inferred shapes before any native dispatch. Operations initially cover F32/I32/U32 input, equal-shape elementwise arithmetic, rank-2 matrix multiplication, affine Q4 matrix multiplication and RMS normalization. Evaluate only terminal output, then copy into Rust-owned memory. GPU is explicitly selected, with CPU as an explicit test option. Never reinterpret the MLX floating-point path as deterministic Q1.31 arithmetic; custom integer Metal kernels retain their existing contract.

Graph validation bounds nodes, rank, dimensions, copied input bytes and total logical tensor bytes. These are conservative logical limits, not a hard cap on MLX's physical allocator, driver scratch, weights or OS memory. Do not advertise this host submission limit as MLX's internal command-buffer cap.

# Implementation Details

Rust controls graph validation, ownership, submission and shutdown. C++20 is used only for MLX API calls and exception containment. The native SDK is selected by MLX_ROOT, or vendor/mlx; missing headers/libraries fail the build. A local SDK may be copied from an installed MLX distribution without using or linking its Python extension. Library dependency inspection must show no libpython. Default builds exclude the old Rapid worker adapter; it remains an explicitly named legacy feature for historical evidence reproduction.

# Testing Evidence — planned gates

Independent CPU arithmetic oracles cover dense and packed Q4 multiplication, RMS normalization and multi-operation graphs. Adversarial cases include bad IDs, incompatible shapes/dtypes, zero/oversized dimensions, NaN/Inf inputs, memory and node caps, cancellation and bounded overload. Pressure runs compare thousands of native GPU completions and drained accounting. Run existing Rust, integer Metal and TypeScript gates, strict Clippy and formatting. Record actual results after execution; this design does not claim those gates have already passed.

# Deployment / Rollout

Local prototype only. No model server promotion, existing service replacement or production sampling change. No Qwen forward implementation, tokenizer, recurrent state, KV codec or model qualification is claimed by the tensor backend. Legacy checkpoint benchmark artifacts remain historical.

# Next Stage of Implementation

First establish native tensor execution and its lifecycle. Then add persistent safetensors parameter loading and immutable model identity, implement Qwen attention/recurrent/MoE operators and KV ownership in Rust, integrate the existing iteration scheduler, and qualify single versus tensor-batched generation before repeating codec A/B on the intended fine-tune. Unsupported model features must fail explicitly; never route back to Python.

# Push further and grounding

Use Apple's installed MLX 0.32.4 headers as the API contract. This engineering design is not grounded in a new published research claim. Packed KV attention, AMX/ANE execution, global physical memory brokering and Apple-wide gold-standard performance remain unqualified roadmap work.

# What We Learned

A working external model smoke test is not a native backend. Model identity and execution architecture need separate evidence gates.

# Changelog

1.0.0 — 2026-10-22 — Native-first design recorded before implementation.

# Cross-framework inference — completion-owned joint admission

Source inspection on 2026-10-22 identifies a common pattern: each throughput optimization moves work across time and changes the state that must remain live. Prefix reuse trades prefill for residency; compression trades storage for restore scratch/work; batching changes recurrent state and deferred graph growth; overlap delays the moment resources may be reused. Independent token and memory caps can therefore admit individually valid work whose combined retained footprint is unsafe.

Evidence: installed Rapid-MLX 0.16.3 scheduler defines recurrent materialization limits using active sequences times deferred interval, prefill transient memory estimates and adaptive chunk costs. Installed oMLX 0.7.0 scheduler has explicit non-sliceable hybrid boundary snapshots and preflight memory guards. SGLang local checkout df13d5dce31abfeff0e701a62e8c39a136758e6c has PrefillAdder compute/state admission budgets, shared Mamba gap reservation and decode-memory retraction. The SGLang checkout also has local MLX modifications; those are not treated as upstream behavior. No source was copied into the implementation and no upstream speed claim is imported.

Primary source routes: [Rapid scheduler](https://github.com/raullenchai/Rapid-MLX/blob/main/rapid_mlx/scheduler.py), [oMLX scheduler](https://github.com/jundot/omlx/blob/main/omlx/scheduler.py), [SGLang schedule policy](https://github.com/sgl-project/sglang/blob/df13d5dce31abfeff0e701a62e8c39a136758e6c/python/sglang/srt/managers/schedule_policy.py). Local installed sources and their hashes are retained with the engineering evidence; moving web branches are context, not the evidence pin.

Prediction: a native Apple server needs a common completion ledger before adding more cache and batching optimizations. The ledger atomically reserves three separately capped resources: CPU staging bytes, logical native tensor bytes and retained graph nodes. Its lease travels with accepted work, not its caller's receipt. Failed admission rolls back all dimensions together; native completion releases all dimensions together. A command-buffer or graph limit alone cannot express all three resources. The implementation integrates this ledger directly into native MLX admission. It is a correctness and bounded-resource mechanism, not a proven speedup or a measurement of physical MLX allocation.

The second deduced requirement is a typed cache-restoration contract: token-sliceable attention, exact-boundary recurrent snapshots and packed KV restoration have distinct validity and temporary-memory rules. This remains the next model-layer design; the tensor foundation must not invent a cache hit or borrow an incompatible recurrent snapshot. A future scheduling quote will combine these contracts with observed prefill/decode cost, but no adaptive model scheduler is claimed in this increment.

# Source comparison and transfer plan

| Framework inspected | Engineering problem addressed | Transfer into this server | Boundary |
| --- | --- | --- | --- |
| Rapid-MLX 0.16.3 | Deferred recurrent graphs, decode stalls from prefill, cache representation costs | Bound retained graph work; later calibrate chunk size against decode latency | Its constants and throughput are not portable proofs |
| oMLX 0.7.0 | Exact hybrid boundary snapshots, native cache layout, process-aware preflight memory | Distinguish recurrent snapshots from sliceable attention; budget restore scratch | A cache hit must prove state compatibility, not merely matching tokens |
| SGLang pinned local checkout | Separate compute/KV admission, state-slot reservation, decode retraction and overlap | Atomic multi-resource admission; later fair tensor batches and recovery | CUDA policies and local MLX changes require Apple-specific qualification |

The integration unit is a validated state-and-work obligation, not a concatenation of three servers. The implemented ledger is the first common mechanism. Persistent parameters, typed state restoration and token scheduling still need to join it. This is a grounded engineering inference and implementation new to this server, not a claim that multidimensional admission has never existed elsewhere.

# Actual testing evidence

The complete judge passed on Apple M5 Pro with MLX 0.32.4: 89 Rust tests in each normal and debug-validation build, five TypeScript tests, TypeScript 7.1/tsrs, strict all-feature Clippy, formatting, existing integer Metal/signpost/capture gates and CLI rejection of the default-disabled Python adapter. The no-default-feature Rust suite also passed. Eight new native tests include independent dense/Q4/RMS scalar oracles, exact I32/U32 preservation, malformed graph rejection, cancellation, drain and 4,500 GPU completions across 1,000/1,500/2,000 windows. Eight threads attempted 80,000 resource reservations while checking bounds and complete drain.

A separately compiled C++ private-ABI judge ran 1,000 cycles under AddressSanitizer and UndefinedBehaviorSanitizer, including null pointers, invalid dtype/dimensions/IDs/operand counts/opcodes/output sizes and successful work after rejected calls. Both sanitizers passed for the instrumented bridge/judge. Apple's prebuilt MLX library itself was not instrumented; leak detection was disabled because this run does not establish whole-process leak freedom. See `evidence/native-mlx/bridge-sanitizer-judge.cpp`, build/run logs and `evidence/native-mlx/native-tests.log`.

The final release binary and native SDK hashes are retained beside `evidence/native-mlx/macro-throughput.txt`. A single diagnostic matrix run measured 64/128/256/512-square F32 matmul graphs over 1,000/1,500/2,000 completion windows. Rates ranged from 2,735 to 6,252 graphs/s. All 18,000 timed graphs plus 12 warmups completed; each profile drained, with at most three admitted graphs. Timings include graph/input construction, Rust admission, native execution, completion and CPU-owned result transfer; exclude backend creation and one warmup per profile. These are synthetic tensor-graph rates, not token throughput, a scheduler ablation, confidence intervals, sustained thermal qualification or a comparison against the three frameworks.

Dependency inspection shows no libpython/Python.framework in the server or native MLX dependency lists. The default Tokio feature tree excludes process/io-util support; those are enabled only by the historical `legacy-python` feature. The existing comprehensive judge is a development Python script; the native server execution path and its tensor benchmark do not invoke it or an interpreter.

# Reproduction

```sh
# build.rs and native/mlx_bridge.cpp — SDK headers/lib must match.
MLX_ROOT=/absolute/path/to/native/sdk cargo build --release --locked
cargo test --locked
cargo test --locked --features debug-validation
cargo clippy --all-targets --all-features --locked -- -D warnings
./target/release/rust-server native-mlx
./target/release/rust-server native-mlx bench
# Standalone private-ABI sanitizer judge, using the locally staged SDK.
c++ -std=c++20 -fsanitize=address,undefined -fno-omit-frame-pointer -g \
  -Ivendor/mlx/include -Inative native/mlx_bridge.cpp \
  evidence/native-mlx/bridge-sanitizer-judge.cpp -Lvendor/mlx/lib -lmlx \
  -Wl,-rpath,"$PWD/vendor/mlx/lib" -o /tmp/rust-server-mlx-asan
ASAN_OPTIONS=detect_leaks=0:halt_on_error=1 UBSAN_OPTIONS=halt_on_error=1 /tmp/rust-server-mlx-asan
```

The local SDK was staged with ordinary file copies of `include/` and `lib/` from the installed MLX 0.32.4 distribution into `vendor/mlx/`. `MLX_ROOT` may instead point to a separately built native SDK. The SDK is ignored, not redistributed or rebuilt by this change. This application uses Apple's native implementation through FFI; it does not reimplement MLX internals in Rust.

# Adversarial review — author self-review

| Concern | Evidence / outcome |
| --- | --- |
| Caller cancellation frees live native inputs | Command owns graph, permit and ledger lease; cancellation test still completes and drains |
| One budget succeeds while another fails | Atomic mutex-protected three-axis reservation; overflow and per-axis rejection leave usage unchanged |
| Shutdown closes admission and accidentally discards accepted work | Admission closure is separate from native failure poison; shutdown joins and accepted receipt succeeds |
| MLX exception crosses Rust ABI | Every mutating bridge call catches exceptions; sanitizer judge exercises failures followed by valid execution |
| Raw integer bits pass through F32 | Terminal typed copy preserves MIN/MAX I32 and U32 high bits on CPU and GPU |
| Malformed packed-Q4 descriptors reach kernels | Shape/dtype/group validation rejects them before native admission; scalar Q4 oracle covers all supported groups |
| Native or nonfinite-output failure silently continues | Executor latches closed and rejects subsequent submissions; failure/drain test passes |
| Three admitted graphs mistaken for three native command buffers | One dedicated worker executes serially; MLX's internal command-buffer count is not controlled or claimed |
| Logical tensor/node budget mistaken for physical UMA/Metal handle count | Reported resources count declared graph nodes and logical bytes only; MLX internal temporaries, driver allocations and returned caller-owned results are outside this ledger |

No independent reviewer approved this work. Process/global memory brokering, catastrophic device-loss recovery, callbacks without per-graph stream synchronization, persistent parameter storage and full model execution remain open engineering work. Dropping a backend detaches a worker that drains accepted commands; explicit async shutdown joins and provides final accounting. A failed join reports unfinished work rather than claiming a successful drain.

# Confirmed model target

Stewart selected `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined` as the Opus/Cyber fine-tuned model. `fixtures/native-model.path` records it without a JSON runtime profile. All twelve files are fingerprinted in `evidence/native-mlx/target-checkpoint-sha256.txt`. Inspection finds four base weight shards, a separate MTP shard and mixed per-module quantization. The local MODEL.md contains historical upstream descriptions; it is not independent proof of the current fusion procedure. No full model smoke was run in this increment. The earlier OptiQ A/B does not qualify this checkpoint.

# Model scope ruling — 2026-10-22

Stewart limited model implementation and integration to Qwen3.6 Combined MoE (the workhorse) and abliterated Qwen3.8 27B (the only other target). The workhorse path remains `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`. The exact Qwen3.8 abliteration checkpoint has not been bound to this server; do not substitute an upstream/base Qwen3.8 artifact or infer compatibility from its name. Workhorse implementation remains first. Shared native components should serve these two concrete architectures; broad model-family support is outside current scope. Qualify each artifact and its state/quantization contracts independently.
