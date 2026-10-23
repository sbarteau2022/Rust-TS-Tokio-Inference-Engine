---
title: "Preserved supplied source specification"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T03:50:40Z"
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
    lines: "1-81"
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
    lines: "1-262"
    role: "primary"
  - path: "examples/native_mtp_bench.rs"
    lines: "1-130"
    role: "primary"
  - path: "examples/native_mtp_decode_smoke.rs"
    lines: "1-160"
    role: "primary"
  - path: "examples/native_mtp_scheduler_bench.rs"
    lines: "1-264"
    role: "primary"
  - path: "examples/native_mtp_scheduler_smoke.rs"
    lines: "1-155"
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
    lines: "1-232"
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
    lines: "1-458"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/choices.rs"
    lines: "1-133"
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
    lines: "1-110"
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
    lines: "1-206"
    role: "primary"
  - path: "src/serving/native_mtp_cli.rs"
    lines: "1-220"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-436"
    role: "primary"
  - path: "src/serving/native_mtp_scheduler.rs"
    lines: "1-726"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-694"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-386"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-213"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-568"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-762"
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
    lines: "1-577"
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
related_documents: ["DESIGN.md", "../README.md"]
tags: [rust, metal, prototype]
---
#####    Creating the MLX-Rust/Tokio/TS/C++ Runtime   #####

UPDATED 2026-10-21 — audit corrections applied and scope cut to a BUILDABLE
PROTOTYPE an agent can start today. Corrections are recorded in §7, not
silently rewritten. This prototype does NOT replace spec 13
(Meta/13-rust-mlx-hybrid-server.md): spec 13's server still rides MLX via
mlx-rs. This project is the kernel-compiler spike beside it — its job is to
prove the TS-trace → Rust → Metal pipeline end to end on ONE kernel, judged
against MLX itself.

## Sequencing decision (Stewart, 2026-10-21)

Build on the CONVENTIONAL architecture first. No hyperbolic-map
integration anywhere except one seam: model retrieval through the
one-mind map stays a PLUGIN (context in, context out; no tendrils into
the KV core or the scheduler). Phase 1's judge is a bench harness run
against the incumbents on this machine — oMLX, rapid-mlx, SGLang, and
the DSpark server (dspark and dflash modes) — same model artifact, same
prompts, same sampler settings, speed always reported with its
waste/acceptance context. The hyperbolic cache (spec 13 §12.3) is
PHASE 2: a policy/substrate port behind the cache-policy seam, judged
by the SAME harness as a one-variable A/B. So phase 1 is not
throwaway: blocks stay standardized and exact with parentage recorded
(policy v1 may ignore the metadata), and the cache-policy interface
stays replaceable.

## 0. Prototype scope

IN SCOPE (the whole proto):
  1. A TypeScript front-end whose functions RECORD a symbolic trace of
     block-matrix ops (tl.load / tl.dot / tl.add / tl.relu / tl.store).
  2. FlatBuffers as the trace's wire/storage format between TS and Rust.
  3. A Rust core (Tokio) that ingests the trace, lowers it through a small
     SSA tile IR, and emits Metal Shading Language (MSL).
  4. A Unified Memory Allocation (UMA) layer over Metal shared-storage
     buffers: weights written from Rust, read by the kernel at the same
     address. No copies, no PCIe (there is no PCIe in this machine).
  5. ONE kernel, the Minimum Viable Kernel: GEMM + bias + ReLU, fused.
  6. Telemetry from day one: os_signpost spans on every job; an optional
     MTLCaptureManager hook for GPU captures.

OUT OF SCOPE for the proto (see §6, Parked): the ANE, CPU AMX, any kernel
beyond the MVK, any optimization pass beyond basic tile-residency choice,
and any serving feature (batching, KV, HTTP). If it is not needed to run
the MVK through the full pipeline, it is not in the proto.

## 1. Verified foundations (corrected — see §7)

- **tsrs** is REAL and is what the original note said: an experimental Rust
  port of the TypeScript compiler AND language server, by Cloudflare +
  VoidZero + Max Schwenk (Owner.com), in close collaboration with the
  TypeScript team, incubated in Oxc. Cloudflare Blog, 2026-10-20: "You can
  try an experimental release" — `npm install tsrs` / `npx tsrs`. Up to 8x
  faster and ~70% less memory than TS 7.0 in early benchmarks; TS 7.1 beta
  is itself up to 2.5x faster than 7.0, partly from tsrs optimizations
  upstreamed. Captured copy: Meta/Cloudflare/cloudflare-tsrs-rust-port-2026-10-20.md.
  ROLE IN THIS PROTO: tsrs typechecks the TS front-end (it is already
  installed and proven on this machine — Model Home, `npx tsrs -p ui`,
  exit 0). tsrs does NOT record traces and does NOT execute DSL math; the
  trace-recording layer is OURS to build (§2, stage 1). The doc must never
  imply otherwise.
- **TS as DSL host**: TypeScript code in this design does NOT compute math.
  It calls builder functions that append op records to a trace. This is a
  library we write (plain functions + an op list), executed under Node,
  typechecked by tsrs. The idea is standard embedded-DSL tracing; no
  framework provides it off the shelf.
- **Metal matrix path**: the GPU's matrix units are reached from MSL via
  `simdgroup_matrix` primitives. AMX is a different piece of silicon — the
  CPU's matrix coprocessor, reached via Accelerate — and is NOT part of
  this proto. (The original note mapped them onto each other; corrected.)
- **Buffers**: `MTLResourceStorageModeShared` is the UMA requirement —
  CPU and GPU see the same LPDDR5 pool. Tensor buffers MUST be 64-byte
  aligned at minimum; tile layouts follow simdgroup_matrix fragment
  requirements. The Rust memory manager enforces layout at tl.load /
  tl.store lowering time.
- **FlatBuffers**: zero-copy reads on the Rust side, no parse/unpack step.
  Its job is carrying the trace and job envelopes. (Compilation is the slow
  rare path; do not gold-plate IPC latency — the original note's
  "microsecond latencies" framing is retired.)
- **Tokio**: the async runtime that schedules, submits, and tracks
  hardware jobs without stalling the user-facing thread. ("Erlang/Tokio
  Style Allocator/Controller" from the original note is made concrete
  here: the UMA allocator and the job controller are Tokio tasks with
  supervision-style failure handling — a failed job is reported on its
  receipt, never panics the runtime.)

## 2. Architecture — five stages

  Stage 1 — TS trace front-end (TypeScript, typechecked by tsrs)
    Builder API: load(name, shape), dot(a, b), add(a, b), relu(a),
    store(a, name). Each call appends one op record to a trace.
    Output: a FlatBuffers trace file (.fbs schema below).

  Stage 2 — FlatBuffers schema (trace.fbs)
    table Trace { ops: [Op]; }
    Op = Load { name, rows, cols } | Dot { a, b, out } |
         Add { a, b, out } | Relu { a, out } | Store { a, name }.
    The schema models block-matrix ops ONLY (the tl.load / tl.dot bridge
    from the original note). Generate TS + Rust bindings with flatc.

  Stage 3 — Rust tile IR (SSA)
    A SMALL SSA intermediate representation written in Rust: values are
    versioned tiles; ops mirror the trace ops plus layout annotations.
    This is deliberately NOT the MLIR framework (MLIR is a C++ framework;
    binding it is a later decision, not a proto dependency). The IR's one
    optimization decision: for each tile, whether it resides in
    threadgroup memory across its uses or is flushed to unified memory —
    decided from use counts in SSA form. That is the proto's entire
    optimizer; anything smarter is a later step with its own judge.

  Stage 4 — MSL emission + UMA execution (Rust, metal-rs)
    Lower IR → MSL source → compile at job time → dispatch.
    Weights and inputs live in shared-storage MTLBuffers allocated by the
    UMA layer (64-byte alignment enforced). The kernel receives pointers;
    nothing is copied between Rust and the GPU.
    MVK kernel shape: load M*K and K*N tiles into threadgroup memory;
    compute micro-GEMMs with simdgroup_matrix; fuse bias-add and ReLU
    (clamp at zero) in-register; store the M*N result tile back to UMA.

  Stage 5 — Tokio job runtime + telemetry
    Jobs arrive as (trace file, input buffer names) envelopes; the runtime
    compiles (stages 3–4), submits, and returns a receipt: job id, kernel
    name, shapes, wall time, os_signpost span ids. A failing job produces
    a failure receipt with the stage and error — fail-closed, no silent
    fallbacks.

## 3. Build order — every step has a judge; a step is done only when its judge passes

  Step 0 — Skeleton. Cargo workspace + TS package; flatc codegen for both
    languages; Tokio app that starts and idles.
    JUDGE: `cargo build` clean; `npx tsrs` typechecks the TS package with
    zero errors (same standard the Model Home UI met this morning).
  Step 1 — Trace round-trip. TS builds the MVK trace (load A, load B,
    dot, add bias, relu, store); Rust reads it back.
    JUDGE: the op list Rust prints equals the op list TS emitted,
    op for op, in order. Mismatch = step fails, no proceeding.
  Step 2 — UMA buffer layer. Rust allocates shared buffers, writes a
    known pattern; a trivial Metal kernel (identity copy) reads it.
    JUDGE: bytes the kernel writes back equal the pattern byte for byte;
    alignment assertion fires on a deliberately misaligned request.
  Step 3 — IR + MSL for the MVK. Lower the step-1 trace; emit, compile,
    and run GEMM+bias+ReLU on stated shapes (start 128x128x128, then one
    non-square case).
    JUDGE: output matches a reference GEMM (Accelerate or MLX via a
    one-off comparison harness) within stated fp32 tolerance on every
    element; any NaN or out-of-tolerance element fails the step.
  Step 4 — Runtime + telemetry. Submit the MVK job through Tokio N times;
    receipts carry timings and signpost ids.
    JUDGE: N/N jobs complete with correct results; submission path never
    blocks the caller (submit returns before the job finishes); signposts
    present for every job in the log.
  Step 5 — The honest bench. Same GEMM shapes through MLX (mlx-rs or the
    Python MLX one-off harness) vs through this pipeline.
    JUDGE: the report states both numbers and the gap, whatever it is.
    The proto SUCCEEDS if the pipeline works end to end and the gap is
    measured. Beating MLX is NOT the proto's success condition.

## 4. Environment (this machine — give the agent no reason to guess)

  - Apple M5 Pro, 48GB unified memory, macOS. Metal via the `metal` crate
    (metal-rs) is the default GPU route from Rust.
  - cargo: ~/.cargo/bin. node/npm: /opt/homebrew/bin (not on the
    non-interactive PATH — prefix it).
  - tsrs is installed at ~/Desktop/Model Home (npm root) — the proto gets
    its OWN package; do not reach into Model Home's node_modules.
  - Project home: ~/Desktop/Special Ops/Meta/Local/mlx-rust-proto/
    (Local = the local-fleet lane of the Special Ops worktree). Create it;
    keep TS, Rust, schema, and judges in that one tree.
  - FlatBuffers compiler (flatc): install via Homebrew if absent; record
    the version in the project README when it lands.

## 5. Rules for the building agent

  - Build in the order of §3. A step whose judge has not passed is not
    done; do not build step N+1 on an unjudged step N.
  - Every claim in commit messages and the README carries its evidence:
    the judge output, the bench numbers, or "UNVERIFIED".
  - Speed is always reported with its context (shapes, tolerance, the
    MLX comparison number). A speed with no comparison is not a result.
  - If a foundation in §1 turns out wrong on this machine (a crate
    missing, an API different from its docs), STOP and write the finding
    into Meta/Local/Failures.md before working around it.

## 6. Parked (out of the proto, with reasons — do not build these)

  - **ANE backend.** The Neural Engine is reached ahead-of-time through
    CoreML conversion (MIL is CoreML's graph format); the OS compiles the
    model; op coverage is limited and shapes are effectively static. It
    is not a JIT target for a Triton-like pipeline, and there is no
    evidence it can serve this project's kernels. Revisit only with a
    working CoreML conversion of the MVK in hand.
  - **Full MLIR / custom dialect stack** (the original note's
    apple_tile / metal_simd / MIL hierarchy). MLIR is a multi-year
    framework commitment; the proto's small Rust SSA tile IR (§2 stage 3)
    carries the same concepts at proto scale. The dialect names are
    retired until a second kernel family exists to justify them.
  - **CPU AMX path** (Accelerate). A different backend decision; the
    proto is GPU-only.
  - **Cloudflare Workers pairing.** The tsrs post's Workers/Wasm story is
    about dev toolchains, not this kernel pipeline. If a Workers tie-in
    ever makes sense, it will be its own design in the Cloudflare lane.

## 7. Audit corrections record (2026-10-21)

  Applied at Stewart's direction after the audit of the original note:
  1. tsrs: the original header was a faithful summary of the Cloudflare
     announcement and stands. What the audit corrected is the ROLE: tsrs
     is the toolchain port (compiler + language server), experimental;
     the trace-recording DSL layer is new work (now §2 stage 1), never
     something tsrs supplies.
  2. AMX vs GPU: the original note lowered dialects "to metal_simd blocks
     for the Apple GPU AMX." AMX is the CPU coprocessor; the GPU path is
     simdgroup_matrix. Corrected in §1; AMX parked in §6.
  3. ANE as a lowering target of the JIT: withdrawn. ANE = AOT via
     CoreML. Parked in §6 with the reason.
  4. "Microsecond-latency IPC for compilation requests": priority
     retired (§1, FlatBuffers). Compilation is the slow rare path.
  5. MLIR as the proto's IR: replaced by a small Rust SSA tile IR
     (§2 stage 3); MLIR parked in §6.
  6. Scope: the original note described the full runtime with no build
     order and no judges. §0 and §3 now bound it to the MVK pipeline
     with a judged step ladder.
