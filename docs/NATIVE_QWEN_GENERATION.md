---
title: "Native Qwen hybrid generation and qualification"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T20:00:00Z"
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
    lines: "1-59"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-104"
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
    lines: "1-432"
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
    lines: "1-85"
    role: "primary"
  - path: "src/serving/native_mlx.rs"
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-730"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-107"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-377"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-570"
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
related_documents: ["NATIVE_WEIGHTS_EXECUTION.md", "NEXT_STEPS.md", "../README.md"]
tags: [rust, mlx, qwen, generation, qualification]
---
# Executive Summary

Compose the selected Combined workhorse's complete text decoder with native tokenization and bounded Rust scheduling. A model step returns logits and all next-state components together; cancelled or failed work cannot change the caller's previous snapshot. Generation capability and held-out quality are distinct gates.

# Problem Statement

Operator/projection checks do not execute the complete hybrid model, tokenizer or stateful decode. The current recurrent-only bank cannot establish attention KV or convolution continuation equivalence.

# Solution Overview

Use the existing persistent native parameter store behind a separate dedicated full-model executor. Rust supplies a validated typed architecture and affine module overrides over the C ABI, never JSON transport or a Python worker. Embed selected token rows, run every recurrent/full-attention block and sparse/shared expert, then final normalization/output projection. Native attention/GDN operators retain the reference semantics grounded in the pinned installed MLX-LM sources. Do not load or execute MTP or vision components.

# Design before implementation

State is immutable CPU-owned FP32 snapshots ordered by layer: convolution history plus recurrent matrices for linear layers, rotated K plus V for full-attention layers. BF16 history converts losslessly to FP32 storage then casts back on read; recurrent matrices stay FP32. The snapshot binds executor/checkpoint identity, token offset and shape. Native execution copies input state and returns all new state plus last-token logits only after evaluation/synchronization. The Rust scheduler adapter validates an entire batch before replacing its sequence map. Dropped futures discard candidates; accepted native commands retain their permit and memory charge until completion. This reference has bounded CPU transfers; no zero-copy or device-state performance claim is made.

One dedicated executor and a channel/semaphore cap of three bound native submissions. Maximum context initially 512 tokens and prefill chunks at most eight; state/workspace bounds are explicit and conservative. The existing fair scheduler owns request admission, prefill/decode iteration, cancellation and per-sequence release. Its batch adapter executes rows independently and commits the batch together; this is state-safe iteration batching, not fused GPU tensor batching. Regular native KV is supported first. Reject TurboQuant selection until an actual native attention codec path and quality gate exist.

Read architecture only from the selected workhorse and verify expected dimensions/layer types. Select exact module quantization rather than infer bits from names. Derive norm sanitization from main-store MTP keys or unsanitized convolution layout, matching reference loader behavior; a separate MTP sidecar alone does not trigger shifting. Reject unrecognized layouts and unsupported variants.

Use Hugging Face's Rust tokenizers crate with local tokenizer.json and no remote fetching. Initial chat support is explicit single-turn text user input, optional system message and thinking disabled; bind the inspected template's hash and reject unsupported tools, vision, roles or control-token injection. Raw text tokenization remains available for tests. EOS identities come from actual tokenizer tokens; do not trust conflicting nested configuration values blindly.

# Implementation Details

Implemented in native/qwen_model.inc, src/serving/native_qwen.rs and src/serving/native_tokenizer.rs. All 40 layers execute against the persistent exact mixed-quantization store. src/serving/qwen_tests.rs exercises a synthetic four-layer hybrid fixture; it is not an alternate real checkpoint integration. ModelBackend shutdown now joins the native worker. Runtime configuration crosses a typed C ABI; standard local checkpoint/tokenizer metadata is parsed in Rust.

Only single-turn text chat is supported. The template SHA-256 is 55d4931433fe502b794226ee7f4d206a6bdd436ac9f80eb7d8ebb4c639f9ea0c; EOS is resolved from tokenizer tokens rather than conflicting nested configuration. Qwen3.8 is not integrated in this increment. Returned state is immutable and executor-bound; logical memory reservations are estimates, not measured MLX physical allocation caps. Layer evaluation/synchronization bounds graph construction, with a CPU snapshot copy per layer. This is a correctness reference, not a throughput promotion.

# Testing Evidence

Required: independent operator oracles retained; synthetic full hybrid CPU/GPU and chunk/full continuation; malformed descriptors, foreign/stale state, bounded admission, cancellation, duplicate/permuted sequence IDs, shutdown drain; actual Combined generation, prefix/decode equivalence and held-out output quality. Bind source, SDK, weights, tokenizer/template and numerical mode. Check finite logits/state before any commit. Distinguish actual measurements, self-review and untested claims.

# Was It Proven Necessary?

The previous increment proved projection/operator execution only. Completing hybrid state and tokenizer semantics is necessary before model-token throughput or text quality means anything.

# Deployment / Rollout

Local CLI first. Do not replace existing services or enable proposed production sampling. Keep regular KV explicit and TurboQuant unavailable until qualified.

# Next Stage of Implementation

After continuation and quality gates, optimize CPU snapshots into native state handles, fuse actual tensor batches, implement validated paged/quantized KV, then benchmark TTFT/token latency/aggregate throughput and memory under concurrent load.

# What We Learned

The workhorse's convolution layout is already sanitized; MTP sidecar presence and main-store MTP presence are different inputs to norm sanitization. Source references are engineering guidance, not a qualified teacher run.

# Changelog

1.0.0 — 2026-10-22 — Design recorded before full-model implementation.

# Held-out gate preregistration

Before running the suite, fixtures/native-heldout.tsv fixes 16 prompts and strict trimmed expected responses across arithmetic, Rust, defensive security and instruction following. Expected answers never enter model inputs. Require 16/16, clean EOS and clean scheduler drain for this limited smoke gate; retain failures without changing expectations. This small authored suite is not a domain benchmark or a production-quality certificate. Before it, compare actual workhorse prefill in an eight-token chunk versus single-token continuation: equal top-1, logit normalized RMSE ≤0.01, state normalized RMSE ≤0.01. Record max absolute drift as well. No independent workhorse teacher-logit baseline exists yet; synthetic CPU/GPU equality plus internal continuation is a narrower numerical gate.

# Precision gate finding and explicit reference policy

The first actual BF16 run failed the preregistered continuation bound despite equal top-1; its report is retained unchanged. Add an explicit FP32 activation reference, preserving affine weight storage and FP32 recurrence. Default the local reference to FP32 and expose --activation f32|bf16 on generation; identity includes the numerical mode. This is a new explicit numerical profile, not a retroactive relabeling of the BF16 result. Keep the same held-out prompts and thresholds. Do not tune expected answers or trim punctuation to hide an instruction-following failure.

# Measured outcome and adversarial review

The selected Combined identity is 509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b, using native MLX 0.32.4. BF16 continuation failed: logit NRMSE 0.02087158 and state NRMSE 0.02217916. Explicit FP32 passed: logit NRMSE 0.00048852 and state NRMSE 0.00019037; both modes retained equal top-1. The fixed held-out suite remained 15/16 in both modes, failing strict `no` with `No.`. CPU FP32 reproduces `No.` using the same model equations; this is not an independent implementation oracle. No post-hoc punctuation normalization was introduced. The strict model gate remains FAILED.

Synthetic hybrid tests cover CPU/GPU equivalence, cached/full continuation, context 512 and overflow rejection, foreign snapshots, invalid tokens, duplicate/stale/permuted sequence rows, atomic batch publication after a later-row error, dropped futures, three-sequence residency pressure, cancellation and clean scheduler shutdown. Existing operator oracles and full repository regression gates remain necessary. Actual workhorse context-512 pressure, independent teacher logits, domain evaluation and fused tensor batching remain unproven. This is an engineering self-review, not independent reviewer approval.

Reports and source/binary bindings reside in evidence/native-qwen. Historical BF16 results retain their original binary binding; final FP32 evidence is separately labeled. A passing repository regression summary does not override the failed model gate. Proposed production sampling values remain inactive.

# Final regression evidence

Final-source regression passed 106 default Rust tests, 106 debug-validation tests and 81 no-default-feature tests; five TypeScript tests, TypeScript 7.1/tsrs, all-target/all-feature Clippy, formatting, integer Metal/resource checks, signposts and GPU capture passed. The separately rebuilt existing private-ABI bridge harness exited zero under address/undefined sanitizers. It does not execute the new full hybrid model; upstream MLX is uninstrumented and leak detection is disabled. The final-source FP32 model qualification rerun exits 1 as intended: continuation passes, held-out remains 15/16, scheduler drains. See evidence/native-qwen/qualification-summary.json and f32-final-qualification.log.

# Candidate custom drafting head — proposed, inactive

Stewart raised replacing native MTP with a custom drafting head. Keep native MTP disabled. First expose a bounded, versioned last-hidden-state tap at the exact post-norm/output-head boundary, without copying full intermediate activations into receipts. Bind any head weights/calibration to the exact Combined checkpoint and tokenizer identity. No head or training artifact exists in this increment.

A draft may propose tokens; the target model verifies them. For the initial greedy contract, accept only the longest prefix identical to target greedy tokens, emit the target correction on mismatch, and discard speculative state beyond the accepted boundary. KV, convolution and recurrence must fork/commit/rollback as one unit. The current executor returns only terminal state, so correct accepted-prefix recovery needs bounded intermediate snapshots or target replay; terminal-state truncation is invalid for recurrent layers. Stochastic sampling would require a separately designed exact acceptance/correction algorithm, not greedy prefix acceptance.

Judge the head against target-only execution on output congruence, accepted tokens per second, verification latency, state-copy/replay cost and peak memory. Acceptance rate alone cannot establish a speedup. Require mismatch-at-first/middle/last-token, all-rejected, EOS, cancellation and three-sequence pressure tests before enabling speculation. Qualify target semantics first; speculation must not obscure the existing failed strict formatting gate.
