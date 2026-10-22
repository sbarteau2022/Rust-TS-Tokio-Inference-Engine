---
title: "Native Qwen hybrid generation and qualification"
document_type: "architecture"
version: "1.6.0"
date: "2026-10-22T22:53:08Z"
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
    lines: "1-37"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-60"
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
    lines: "1-211"
    role: "test"
  - path: "judges/suite_reference.py"
    lines: "1-248"
    role: "test"
  - path: "judges/test_attention_profile.py"
    lines: "1-51"
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
    lines: "1-449"
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
    lines: "1-102"
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
    lines: "1-662"
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

# Output choice contract — design before implementation

The raw strict gate failed on formatting (`No.` rather than `no`), reproduced on CPU and GPU. Preserve the original prompt/expected fixture and unconstrained result. Add explicit per-request finite token-path choices, supplied by callers rather than inferred from expected answers. Both `yes` and `no` remain legal for the borrowing question. Select the maximum finite logit among legal next tokens at every decode step, then require EOS at a terminal path; never rewrite completed text.

The scheduler validates bounded choices before admission, rejects unsupported backends, carries a legal-token mask into model steps and independently checks returned tokens before publishing receipts. Require sufficient generation budget for the longest choice plus EOS. Bind a versioned contract fingerprint into receipts. Bounds: 16 choices, 64 tokens per choice, no embedded EOS, duplicates rejected. Canonical tokenizer paths define the supported language; alternate tokenizations are not included. Default unconstrained behavior remains unchanged.

Expose `native-generate ... --choices yes,no` and separate `native-qualify --contracts` evidence. The constrained qualification applies the explicit yes/no contract only where declared; the raw gate remains a separate failing profile. Tests must include shared prefixes, terminal prefixes, ties, invalid tokens/EOS/duplicates/budgets, backend contract violations, sequence isolation/cancellation, and both semantic answers. No production or unrestricted model-quality promotion follows from constrained format compliance.

# Choice-profile findings recorded before typed IR

The pre-IR Combined smoke passed 20/20 strict answers with EOS, continuation and scheduler drain; exit zero. The original 16 prompt/expected rows are unchanged. Only rust-borrow declares both yes/no token paths; four additional balanced semantic cases also pass. See evidence/native-choices/findings.txt and contract-qualification.log, bound to the pre-IR source/binary. This is a constrained-profile pass; unrestricted instruction following and production quality remain unqualified.

# Typed internal IR and strict lexical boundary — design before implementation

Move CLI/chat/tokenization/detokenization and qualification orchestration into native_boundary.rs. The model executor and scheduler adapter must not import NativeTokenizer or operate on text. Preserve trained model token identities; no byte/carrier substitution and no learned symplectic/MLIR claim.

Introduce validated native_ir.rs types for token spans, token position, prefill/decode operations and the immutable hybrid layer program. Rust compiles every layer into an indexed typed mixer instruction (gated-delta or full attention); the C ABI validates and executes that program instead of independently inferring its branch from layer number. Existing MoE/residual/norm equations remain unchanged. Enqueue model-step IR with its unified state and carry the immutable program through the native worker. Decode operations contain exactly one token; bounds and IR/state positions must agree before admission. The FFI alone lowers validated spans to raw uint32 buffers and layer descriptors.

The IR is an execution/control representation, not a replacement model vocabulary or an MLIR compiler dialect. Require unchanged actual Combined continuation/output under the same contract profile, full hybrid CPU/GPU tests, malformed/stale/mismatched IR rejection, and a structural guard against tokenizer dependencies in core runtime modules. Keep raw and constrained quality evidence separate, retain pre-IR evidence, and benchmark overhead before claiming speed gains.

# Typed-IR implementation, validation and remaining limits

Implemented model_ir.rs with private validated token spans, explicit prefill/decode kinds and positions, and an immutable indexed hybrid mixer program. The native worker now queues model-step IR plus its unified snapshot; only forward lowers it to raw ABI descriptors. The C++ executor validates layer count, order and tags and executes the typed mixer program. native_boundary.rs exclusively owns the native model path's chat/tokenization/detokenization and qualification orchestration. Core runtime dependency guards reject lexical imports. Existing separate legacy adapters are not newly qualified or converted by this change.

Final regression passed 113 default Rust tests, 113 debug-validation tests, 87 no-default-feature tests, five TypeScript tests, TypeScript 7.1/tsrs, all-target/all-feature Clippy, formatting and the existing integer Metal/resource/signpost/GPU-capture judges. The typed-IR bridge sanitizer harness rejected 5,000 malformed descriptors before store reads or output writes. Address/undefined instrumentation covers the bridge/harness, not upstream MLX; leak detection was disabled. Full valid hybrid execution has separate CPU/GPU/continuation tests rather than a full-model sanitizer claim.

The post-IR Combined constrained smoke again passed 20/20 with EOS, continuation and scheduler drain. Its full reported continuation metrics and all answers match the pre-IR report exactly. Evidence/source/binary bindings are under evidence/native-ir. The original unconstrained pre-IR rerun remains 15/16 and is retained under evidence/native-choices. The choice profile does not fix unrestricted punctuation behavior. Neither typed IR nor reversible text transport proves universal floating-point determinism or a throughput gain.

Adversarial review covered contract budget/EOS/vocabulary bounds, shared and terminal prefixes, stable ties, unsupported/lying backends, per-sequence masks and cancellation, malformed/foreign/stale state, decode width, IR position/vocabulary mismatch, invalid native layer counts/order/tags and successful work after rejected descriptors. Review is an engineering self-review; independent reviewers are still absent.

Next: independent exact-checkpoint reference logits and broader held-out domain/long-context qualification, persistent device state and genuine tensor batching, then measured native regular/TurboQuant attention and an independently gated drafting head. The typed IR creates an explicit integration point; it does not provide symplectic MLIR semantics or train a new inner representation.

1.1.0 — 2026-10-22 — Explicit output choices; separately qualified constrained profile and preserved raw failure.
1.2.0 — 2026-10-22 — Typed internal model-step/layer IR, lexical boundary isolation and unchanged post-IR constrained model smoke.

The final CLI smoke `native-generate "Does Rust allow two simultaneous mutable references to the same value in safe code? Reply with only yes or no." 8 --choices yes,no` emits exactly `no` then EOS, with contract fingerprint 38530787a615166fb03bb840a5aeb22916f9cbe2a58347ce45f45b65c619f0d6. Both yes/no choices were legal. This single smoke is not a throughput benchmark.

# Extended qualification — design and preregistration

Before tuning device residency, add a separately labeled `native-qualify --extended` local gate. Preserve the original raw/constrained fixtures. Freeze 24 new authored Rust/defensive-security cases before the run, explicitly declaring each case's legal choices independently of its expected answer; require all 24 exact trimmed outputs and EOS. A choice set must contain at least two alternatives, and unconstrained cases remain unconstrained. This is authored engineering qualification, not a standardized Cyber benchmark.

Exercise the actual Combined checkpoint to context 512 with a fixed raw-text-derived token cycle. From position 504 compare an eight-token chunk with eight single-token steps: same top-1 and logit/state NRMSE <=0.01. Reject token 513 without changing the immutable snapshot. Fork position 511 into three admitted one-token operations with distinct tokens; compare their results to serial replay (same top-1, NRMSE <=1e-7), then require the executor ledger to drain. This tests real hybrid KV/convolution/recurrent state and bounded native queue isolation; it does not establish fused tensor batching.

Use the existing FP32 profile, exact Combined fingerprint and native typed IR. Keep tokenization in I/O. Record per-case results, state size, timings, logical scheduler memory and drain separately. CPU/GPU or split-continuation agreement is same-implementation evidence, not an independent full-model reference. Print independent_reference=false and full_model_qualified=false even if these local gates pass. An external, exact-checkpoint independent reference remains required and must not be fabricated from this executor's own outputs.

# Extended measured findings — gate failed, preserved

The exact Combined FP32 run scored 23/24. Case r10 expected `yes` for cancellation of a future by dropping it; the model emitted `no` despite both alternatives being legal. Keep the prompt/expectation unchanged. [Tokio's select tutorial](https://tokio.rs/tokio/tutorial/select) documents cancellation by dropping a future; [JoinHandle documentation](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html) distinguishes dropping a join handle, which detaches its task. The authored question tests the former distinction; no answer rewrite or constraint narrowing was applied.

At context 512 the unified snapshot contains 86,835,200 bytes. From the same position-504 snapshot, chunk-eight versus eight single-token steps produced logit NRMSE 0.05924135 and state NRMSE 0.01249746, both exceeding the preregistered 0.01 bound. This is observed chunk-dependent numerical drift; its operator-level cause has not been established. The earlier eight-token/20-case smoke remains valid within its narrower scope and cannot qualify context 512.

Overflow rejection, three-fork versus serial replay isolation, native lease drain and scheduler shutdown passed. Scheduler logical memory peaked at 797,376,512 bytes; this is a logical reservation, not measured MLX physical residency. The model gate exits 1. Evidence/native-extended retains source/binary/SDK/fixture identities, the failed report and structured summary.

Regression passed 114 default Rust tests, 114 debug-validation tests, 87 minimal-feature tests, five TS tests, TS 7.1/tsrs, Clippy, formatting and existing integer Metal/resource/signpost/GPU capture checks. Harness tests passing do not override the model gate. Actual scheduler cancellation/restart under workhorse load and an independent full-model oracle remain unestablished.

The next necessary engineering step is bounded layer/operator tracing at the first divergence from the same position-504 snapshot: isolate quantized projection dispatch, attention, recurrent updates and MoE route selection, then compare the suspect operator against an independently coded numerical oracle. No cause is presumed. Do this before persistent device state, tensor batching or drafting-head optimization. Preserve both failures and rerun the same gates after any fix.

1.3.0 — 2026-10-22 — Preregistered extended qualification; retained 23/24 domain result and failed context-512 continuation, with passing overflow/isolation/drain controls.

# Divergence diagnosis — design before implementation

Add opt-in bounded typed layer tracing to the native model IR. Capture each layer's normalized input, mixer result, residual, MoE result and output for every token; return CPU-owned FP32 trace data after completion. The production/default path allocates no trace. Charge trace storage to the completion ledger and validate ABI lengths before evaluation. Diagnostic traces may affect graph materialization: compare traced and untraced runs from identical state and require logit/state NRMSE <=1e-6 before treating a trace as evidence about the original failure.

Reproduce the frozen context fixture from position 504. Compare chunk-eight layer/token trace with eight single-token traces and report the first stage with NRMSE >1e-4, while retaining full trace differences. Do not assume which kernel caused it. Add an independent scalar numerical oracle only after locating the suspect stage. Keep original failures and acceptance limits unchanged; no throughput or full-model qualification claim follows from a trace alone.

Attention diagnostic oracle: traced calls additionally compare SDPA with a scalar
FP64 stable-softmax implementation on CPU-owned copies of the actual post-RoPE
Q, cached K and V. Grouped heads map by integer head ratio, and row i admits
keys 0 through offset+i inclusive. This tests attention arithmetic and cached
causality independently of MLX attention; it does not independently qualify
projections, weights, the complete model or semantic answers. Trace/untraced
controls remain mandatory because materialization can affect dispatch.

Correction under test: replace shape-dependent fused SDPA with explicit FP32
GQA matrix products and stable softmax. Q is reshaped into KV-head/group axes;
K/V broadcast across query groups. An explicit bottom-right additive mask admits
key <= offset+query row. This is a correctness reference path; no throughput
promotion is implied. The prior fused path remains preserved in failed evidence.

The explicit batched FP32 matmul experiment reproduced the same ~0.00089
attention error against FP64; expressing the operation differently did not fix
it. Next correction under test: evaluate each query row through the validated
single-query SDPA dispatch, with K/V sliced through offset+row inclusive and no
implicit mask. Concatenation restores original head/token order. This preserves
GPU execution but intentionally trades prefill dispatch efficiency for a stable
reference. Hardware-internal precision is not established by these experiments.

The single-query dispatch is also inaccurate for short keys (synthetic six-token
oracle NRMSE 0.000777), so it is not accepted as a general fix. The final reference
candidate uses FP32 elementwise products and reductions for both QK and PV,
row by row, with grouped-head broadcasting and physically truncated K/V. It
avoids shape-dependent matrix dispatch entirely; scalar-oracle agreement is a
hard diagnostic gate at 1e-5 NRMSE. Peak per-row products are bounded by the
existing 256 MiB logical temporary allowance for this supported architecture.

### Observed attention correction

The initial trace controls were exactly unchanged. Layer 0–2 mixer errors were
below 5e-7 NRMSE; layer 3 attention rose to 0.000877932, with later amplification.
For actual layer-3 QKV, batched attention differed from the independent scalar
oracle by 0.000893746; the long-context single-query path was ~1.18e-7. Explicit
batched FP32 matmul reproduced the error. Single-query dispatch fixed that window
but failed the six-token synthetic oracle at 0.000776917, so was rejected.

The final elementwise/reduction path passes the scalar oracle (first actual
batched attention 1.14e-7), short-context synthetic checks and trace controls.
At 512, logit NRMSE is 3.79e-7 and state NRMSE 3.36e-7; no traced stage exceeds
1e-4. These observations implicate shape-dependent matrix/attention arithmetic;
the hardware-internal precision mechanism is unverified. Cached causal visibility
is explicitly checked by the independent oracle and analytic test cases.

The final path performs per-row FP32 products/reductions on GPU, not CPU inference
or custom floating-point Metal kernels. CPU FP64 checks run only with explicit
tracing. No comparative throughput claim is made; test runs may overlap other
verification jobs. A future fast attention kernel must pass the same short/long
context and oracle gates before replacing this reference.

### Final unchanged-fixture gates and adversarial review

Final reference results: constrained 20/20; extended domain 23/24 (r10 remains
`no` against expected `yes`); raw 15/16 (original punctuation failure retained).
Context continuation, overflow, three-fork replay isolation and drain pass.
Independent attention arithmetic is established only on supplied projected QKV;
projection dequantization, tokenization and complete logits still lack a separate
exact-checkpoint implementation. No full-model promotion follows.

Reviewed trace capacities before store access, finite snapshots, unchanged
untraced allocation behavior, grouped-head indexing, cached causal bounds,
ledger charging and drain on errors. Analytic scalar tests and 7,000 invalid FFI
descriptors run under ASan/UBSan; synthetic native CPU/GPU/chunk tests include
trace/untraced equivalence. Full regression passes 114 default / 114 debug / 87
minimal Rust tests, five TS tests, Clippy and compiler/Metal gates. This is an
implementation adversarial review, not independent external sign-off. Prebuilt
MLX is uninstrumented and its internal command buffers/physical workspace remain
outside Rust's hard caps. Failed intermediate experiments remain visible.

### Projection reference and remaining semantic failure — design first

Add a separately opted-in projection-check flag to validated model-step IR and
its private C ABI. Ordinary execution and existing trace mode keep it disabled.
The diagnostic checks eight evenly spaced output channels across all token rows
of each ordinary affine projection; for gathered experts it checks two routed
slots of the first token, selecting the matching input, expert and output slice.
Packed rows and scale/bias values are copied to CPU-owned FP32 snapshots. An
independent scalar implementation extracts little-endian packed bits across byte
boundaries, applies scale*q+bias in FP64 and accumulates FP64 dot products. It
must support 2/3/4/5/6/8-bit affine packing and groups 32/64/128, reject malformed
or nonfinite inputs, and pass independent bit-by-bit packing and analytic tests.
It does not call MLX dequantization, matmul or gather kernels. Diagnostic sample
NRMSE limit is 1e-4; samples do not establish unsampled-channel correctness.

`native-reference` reuses the unchanged r10 fixture. It compares ordinary,
projection-checked and single-token execution from the same pre-tail state,
reports exact yes/no token logits and the target's legal choice, and preserves a
semantic failure. Optional `--device cpu` repeats the same equations on CPU as a
backend control, explicitly not an independent full-model oracle. Expected
answers never enter the forward pass. Checks materialize extra graph nodes, so
checked/untraced state and logits must agree within 1e-6 NRMSE before interpreting
them. CPU copies are bounded by selected channels; no full dense expert matrix
is materialized. Existing logical temporary reservation covers these snapshots;
physical workspace remains outside that guarantee. Full-model qualification
still requires independent nonprojection operators and checkpoint tokenization.

Focused final-head extension: the text-boundary diagnostic supplies the exact
candidate token IDs as a bounded, unique typed list (1–8 IDs, each < vocabulary).
The final LM-head oracle checks those channels instead of the evenly spaced
sample. The C ABI rejects invalid lists before reading the weight store. This
checks the two decision logits against independent FP64 dot products on the
actual hidden state, while avoiding token/text logic in core execution. It still
does not validate how that hidden state was produced by nonprojection operators.

Expert coverage refinement before final qualification: probe both the first and
last token of a multi-token tail, at routed slots 0 and last. A one-token tail
uses that token once. This directly includes the failing decision's last-token
expert operands while remaining a sample (two of eight routes, eight output
channels). Ordinary projections already include every tail token row.


### Measured projection and semantic findings

The final diagnostic checks 871 sampled projections per backend, including
ordinary projection rows, first/last tail-token expert slots 0 and 7, and the
exact decision-head channels (yes=9405, no=2083). On this checkpoint run only
4/8-bit modules with group 64 were observed; other widths/groups have synthetic
packing evidence only. GPU maximum sampled NRMSE is 7.75145078e-7; CPU maximum
is 6.89049815e-7, both below the preregistered 1e-4 limit. Checked/untraced state
and logits are identical. Tail chunk/single maximum NRMSE is 5.73e-7 GPU and
6.10e-7 CPU. This compares seven tail tokens from each backend's own position-24
state, not the full prompt executed entirely token by token.

GPU yes/no logits are 19.005685806 / 19.114009857 (no margin 0.108324051).
CPU gives 19.075370789 / 19.208658218 (no margin 0.133287430). Both choose the
unchanged wrong answer. The focused final-head absolute errors versus scalar
FP64 are at most 5.36432584e-7 GPU and 1.70389918e-6 CPU. Thus sampled affine
projection arithmetic does not explain the observed decision, given the actual
hidden states. The two backends' logits differ; identical choices do not imply
bitwise determinism or an independent full-model reference.

The diagnostic deliberately exits 1 for the unresolved semantic gate. Neither
expected answers nor final-head probe IDs bias forward logits. Production calls
leave projection checks disabled. Independent checkpoint parsing, unsampled
expert/channel arithmetic and nonprojection operators remain outside this proof.
Next, independently check checkpoint bytes and recurrent/GDN, normalization and
RoPE equations before concluding whether the remaining error belongs to the
implementation or to the fine-tuned model. No held-out prompt/answer alteration,
answer forcing, training on the fixture or production throughput promotion occurs.

### Projection adversarial review and qualification

The default production path keeps checks and trace disabled. The private ABI
validates diagnostic mode, trace capacity and candidate channel bounds/uniqueness
before accessing the weight store. Rust owns the immutable IR and CPU snapshots;
existing completion leases remain held through native completion. The focused
candidate list contains both answers and does not mask or alter model logits.

The full judge now builds/runs 55 scalar projection cases under ASan/UBSan on
independently packed test values. They cover every supported width/group pair,
word-crossing extraction, affine biases, negative values and malformed/nonfinite
refusals. A separate sanitizer harness checks 12,000 invalid IR/trace/projection
wire descriptors against a poison weight-store pointer and unchanged output
sentinels. Native synthetic checks exercise sampled ordinary/expert projections
and trace equivalence. Four malformed CLI forms reject before loading a model.
Prebuilt MLX itself is uninstrumented; no leak-detection claim is made.

Regression passes 114 default / 114 debug-validation / 87 minimal Rust tests,
five TS tests, TS 7.1/tsrs, Clippy, formatting and existing integer Metal/compiler
judges. The context-512 diagnostic is unchanged at logit NRMSE 3.79e-7 / state
3.36e-7. The extended model gate remains 23/24 and exits 1; continuation, overflow,
three-fork isolation and drain pass. Diagnostic CPU and GPU commands deliberately
exit 1 for r10. Regression success does not override these model failures.
Evidence is bound under `evidence/native-reference/summary.json` and its manifest.
This is an implementation adversarial review, not independent external approval.

### Readiness continuation — independent bytes and nonprojection operators

Before another readiness claim, add a diagnostic that compares every native
parameter byte, shape and dtype against Rust-read safetensors payloads using
bounded 4 MiB chunks and cryptographic digests. This verifies MLX ingestion
against separate Rust file parsing; it is not a second model implementation.
Diagnostic exports stay private, validated and off the serving hot path.

Extend checked model-step IR with a separate operator-check bit (bit 0 remains
sampled projections; bit 1 requests independent nonprojection checks). Checks
require bounded tracing; default flags remain zero. Implement scalar FP64 RMS
normalization, SiLU, partial nontraditional RoPE, depthwise causal convolution
and the complete gated-delta recurrence/state update from CPU-owned actual
projected operands and incoming state. Each observed array is compared against
an independently computed scalar result at a fixed 1e-5 NRMSE limit. No CPU
reference result substitutes for a model output. Diagnostic materialization must
still satisfy checked/untraced controls; errors release native leases normally.

The remaining semantic fixture and its expected answer remain unchanged.
Independent operator checks narrow possible implementation defects; their
composition on supplied operands does not by itself establish independent
full-model logits, nor does readiness follow from passing tensor checks.

The full diagnostic will additionally offer explicit `--full` coverage: every
ordinary projection output, every token's active routed experts, all final-head
channels and every prefill chunk of the frozen prompt. This is intentionally an
expensive CPU reference workload, never a serving configuration. Private flag
bit 2 requires both projection and operator checking; invalid combinations reject.
Projection work is tiled into at most 256 output channels; scalar FP64 tiles may
use up to eight joined CPU threads. The same bit-extraction arithmetic remains
independent of MLX matmul/dequantization. Diagnostics remain observational; native
outputs are never replaced. Composed operator checks are distinguished from a
separately implemented full model graph and do not erase semantic failures.

Close the remaining composed-check gaps before interpreting the full run: compare
all selected-token embedding dequantization values, complete routing softmax and
selected weight normalization/top-k membership, routed mixture reduction, shared
expert gating, attention output gating, recurrent output gating and decoder
residual additions with independent scalar arithmetic. Native results remain
observations; scalar values never substitute for outputs. Exact ties may select
any top-k member at the cutoff, but duplicates or a selected value below an
unselected value reject. Embedding decoding uses the same independently tested
packed-bit interpretation as the scalar projection oracle.

Embedding gate failure (preserved before correction): the first checked lookup
has NRMSE 0.00132344712 and max absolute error 0.000244140625 versus exact affine
FP64 decoding. Supplying only a float32 output dtype did not establish FP32
scale/bias arithmetic. Test a correction that explicitly casts embedding scales
and biases to FP32 before dequantization in the F32 activation profile. BF16
remains separately labeled and unqualified. Preserve the failed embedding gate,
then rerun all-channel checks and unchanged semantic suites; do not infer a
semantic improvement merely from improved arithmetic.


### Observed result — ingestion and composed scalar checks

The independent raw-byte audit passes all 1,757 parameters and 19,508,787,456
payload bytes; aggregate name/tensor SHA256 is
`aa4f8c241b218a3c674a407d900808cef556d2abe035fd349f8adcba854975d4`.
This validates byte ingestion, shape and dtype, not model configuration semantics.

Explicit FP32 scale/bias casts correct the embedding discrepancy. A BF16-affine
synthetic hybrid fixture now exercises this path; removing the casts makes the
embedding oracle reject, while restoring them passes. Original real-model
embedding failure: NRMSE 0.00132344712, max absolute error 0.000244140625.

The corrected full run passes 133,880 projection tiles, 2,768 nonprojection checks
and 40 attention checks across all 31 prompt tokens. Maximum respective NRMSE:
2.29420587e-6 / 2.62358289e-7 / 2.87192754e-7. Projection limit is 1e-4;
operator and attention limits are 1e-5. Checked/untraced error is zero;
chunk/single error is 6.07e-7. Frozen r10 still fails: yes 19.005435944,
no 19.113307953. No expected answer enters native arithmetic.

Coverage is complete for the instrumented operations and active projections of
this prompt, not all possible tokens, expert routes or contexts. Scalar checks
consume actual native operands. A separately composed graph with independent
state/config/tokenization and independently generated reference logits remains
an open gate. Do not attribute the semantic failure conclusively to checkpoint
quality, or claim production readiness, from these observations.


Final-source unchanged suites retain 20/20 contracted, 15/16 raw and 23/24
extended; the original format and cancellation failures remain. READY smoke
returns [44061, 248046] with EOS through the native scheduler. Context-512
continuation, overflow rejection, three-request isolation and drains pass.
Regression: 116 default, 116 debug-validation, 87 minimal Rust tests; five TS,
93 sanitized scalar projection/embedding and 43 operator cases; malformed FFI
14,000; Clippy/format/compiler/Metal gates pass. Source, dynamic MLX dependencies,
fixtures and final executable are fingerprinted with phase evidence.
