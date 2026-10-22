---
title: "Model integration foundation: iteration scheduling and typed KV"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-22T00:00:00Z"
implementation_date: "2026-10-22T15:39:43Z"
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
    lines: "1-209"
    role: "test"
  - path: "judges/suite_reference.py"
    lines: "1-243"
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
related_documents: ["SERVING_ARCHITECTURE.md", "SPEC15_IMPLEMENTATION.md"]
tags: [scheduler, kv, model-integration]
---
# Executive Summary

Qwen3.6-35B-A3B-OptiQ-4bit now executes through a Rust iteration scheduler and a private Rapid/MLX worker. Regular KV and Rapid TurboQuant V4/K8V4 prefix storage are explicitly selectable. The worker retains each request's actual attention and recurrent state, consumes it on decode, and acknowledges release before Rust delivers completion. This is an experimental CLI inference path, not a production server or an integer-only Qwen implementation.

Stewart authorized this separately labeled MLX A/B baseline. Existing custom Metal kernels remain raw integer Q1.31. Regular KV remains the default; no production sampling policy or draft rates were activated.

# Problem Statement

The trace runtime had native buffer ownership but no real model token lifecycle. A fixed-size opaque cache payload cannot silently stand in for the hybrid Qwen model's actual attention and recurrent state. The selected checkpoint has 40 layers: 10 full-attention and 30 recurrent layers. Quantizing only attention KV cannot eliminate recurrent-state memory.

# Solution Overview

The design was recorded in this document before implementation and expanded after the user selected Qwen and authorized the Rapid/MLX numerical baseline.

`Engine` bounds admission and active sequences, chunks prompts under a token budget, rebuilds work each iteration, and rotates sequences fairly. Each request receives a private sequence identity. Backend results must match the entire selected batch's IDs, shapes, token timing, vocabulary and state limits before any host state commits. Duplicate, missing, malformed or foreign results latch model admission closed. Native work must finish before a backend step returns; cancellation waits for that boundary before state cleanup.

The model adapter is a private bounded JSONL subprocess protocol, not an HTTP proxy to another server. Rust owns admission, iteration composition, generation limits, receipts, cancellation and drain. The installed Rapid environment supplies MLX-LM model execution and Rapid's actual TurboQuant prefix-storage codec. The worker evaluates MLX results and state before responding; recurrent arrays remain separate and are never fed to a KV codec. Token sampling is greedy and EOS-aware.

Rust can also run host-owned typed Q1.31 KV through the same scheduler interface. That reference path stores exact regular keys/values or regular keys plus an experimental TurboQuant-MSE value representation. It is tested with a clearly labeled synthetic backend; the real Qwen path uses backend-managed native MLX state and never relabels floating-point values as Q1.31.

# Implementation Details

| Files | Purpose |
| --- | --- |
| `src/serving/scheduler.rs` | Backend contract, host/backend-managed state ownership, bounded fair iterations, cancellation, batch validation and receipts |
| `src/serving/mod.rs` | Shared bounded byte reservations and RAII release |
| `src/serving/kv.rs` | Exact Q1.31 regular frames and integer experimental MSE vector codec |
| `src/serving/config.rs` | Strict required scheduler fields, limits and configuration fingerprint |
| `src/serving/rapid.rs` | Explicit model configuration, bounded/timeout-latched subprocess RPC, readiness and release acknowledgements |
| `backend/rapid_worker.py` | Immutable artifact inventory, tokenization, real hybrid-model cache lifecycle and selectable Rapid prefix compression |
| `src/main.rs` | Configuration checking and runnable model CLI with strict overrides |
| `judges/model_ab.py` | Rotated-order real-model A/B, resumed-decode logits and declared diagnostic acceptance gates |
| `tests/serving.rs`, `tests/rapid_rpc.rs` | Numerical, ownership, pressure, cancellation, fault and untrusted-protocol cases |

## Languages chosen

Rust owns the scheduling and safety boundaries. Python uses the explicitly approved existing Rapid/MLX implementation as a numerical baseline; it is not a rewrite of the native fixed-point backend. TypeScript stays pinned to 7.1 and its existing compiler gates remain required. No new floating-point MSL kernel was added.

## Actual configuration and commands

`fixtures/qwen-serving.json` is an explicit local experimental profile, including absolute model/interpreter paths. It reserves 128 MiB per active sequence and 128 MiB transient workspace from a 512 MiB Rust state/workspace budget. Separately, the worker sets a 36 GiB MLX allocator limit including weights. These are experiment settings, not production recommendations or an OS-wide unified-memory guarantee. `fixtures/qwen-serving-long.json` permits longer contexts and 256-token prefill chunks.

The current named overrides are `--kv-codec`, `--max-in-flight`, `--prefill-chunk-tokens`, `--max-batch-tokens`, and `--probe-directory`; duplicate or unknown flags are rejected before loading. Configuration JSON rejects unknown fields and has no hidden model or policy defaults. Resolved settings and their SHA-256 appear in the output. The broader `serve/tune/doctor` command architecture remains proposed.

```sh
# src/main.rs — check the supplied local profile, then run real model inference.
./run.sh config check --config fixtures/qwen-serving.json
./run.sh model-run fixtures/qwen-serving.json fixtures/qwen-prompts.json 16
./run.sh model-run fixtures/qwen-serving.json fixtures/qwen-prompts.json 16 --kv-codec rapid-turboquant-k8v4
```

**In-flight distinction:** existing Rust/Metal commands retain the native 2/3-buffer semaphore cap. For the external MLX baseline, the scheduler has one awaited backend step in progress; MLX's internal native command-buffer count is not exposed or enforced by this adapter. Readiness reports `mlx_native_command_buffer_cap_enforced: false`. The configuration's in-flight field must not be interpreted as evidence that MLX internally obeys that native cap.

## KV format and ownership

Host frames charge their encoded payload bytes before allocation and retain that charge through the last Arc reader. Regular encoding is exact little-endian i32. The experimental host MSE codec stores regular keys, an integer L2 norm, and packed scalar indices for rotated values. Rotation coefficients use signed Q2.30, centers use Q1.31, arithmetic uses bounded widened integers, rounding is ties-to-even, and reconstruction reports saturation. Profiles require shape, coefficient-range, ordered-center and quantized orthogonality checks. These checks do not establish a Haar-random rotation, an optimal Lloyd-Max codebook, or the paper's distortion guarantee. The unbiased residual-QJL construction is not implemented.

For actual Qwen, state stays in MLX. Rapid compresses full-attention cache layers once at completed prefill and materializes native KV before resumed attention. This is **prefix-storage compression, not attention running directly against packed KV**. Both V4 and K8V4 are opt-in, with explicit codec fingerprints. Recurrent `ArraysCache` state stays native and private per request. No cross-request prefix reuse or Spec 15 recurrent-state fork is claimed.

The Rust byte budget is logical encoded-payload/workspace accounting, not total physical process memory. Backend-managed state reserves an explicit upper bound per sequence, verifies reported state bytes, and retains the reservation until release acknowledgement. MLX allocator measurements, weights and peak allocations are reported separately. Temporary Python/driver metadata and OS allocations are not represented by Rust reservation counters.

## Provenance

Worker readiness hashes local JSON/tokenizer/model/safetensors artifacts, checks size/mtime stability across inventory/load, records installed MLX/MLX-LM/Rapid versions, and hashes this worker plus Rapid's codec source. It does not offer an atomic filesystem snapshot against a privileged writer changing and restoring metadata; freeze artifacts operationally before production. The measured environment was MLX 0.32.4, MLX-LM 0.31.3, Rapid-MLX 0.16.3. Each A/B report binds its exact binary, worker, config, prompts, and model identity. Existing servers and model files were not modified.

## Push further

The current backend loops over selected sequences inside a logical iteration; readiness explicitly reports `vectorized_batching: false`. This is continuous request/token scheduling, not a claim of tensor-batched Qwen execution. The next throughput milestone is hybrid-compatible batch merging and attention/recurrent state routing, then native packed attention and a shared global memory broker. HTTP/gRPC serving, tenant isolation, deadline lanes, model hot swap and qualification across chips are still future work.

# Grounding in Verifiable Literature

Zandieh et al., *TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate*, arXiv:2504.19874, https://arxiv.org/html/2504.19874v1 motivates normalized rotation and scalar quantization. The fixed-point host reference and installed Rapid variant are distinct implementations; published accuracy and GPU results do not transfer automatically. Spec 13 §7 supplies iteration/chunking goals; Spec 15 governs the existing cache controller, which remains a separate lifetime/control component.

# Testing Evidence

`python3 judges/run.py` covers the entire existing integer kernel, native completion, CORDIC, resource, cache, adversarial and compiler suites plus the new serving tests, Python syntax and model configuration/CLI rejection checks. Real-model A/B runs are separate because they depend on the local 24+ GiB checkpoint and external Rapid environment. They are not silently replaced by synthetic results.

The serving tests exercise exact regular round trips, zero/extreme integer vectors, independently calculated inverse-rotation values, malformed profiles and packed payloads, 10,000-vector deterministic pressure, concurrent budget contention, at least 10,000 synthetic decode steps, real typed-history consumption, both host codecs, queue saturation, cancellation while a step is blocked, EOS/context bounds, batch atomicity, reversed backend responses, failed release, backend errors/panics, oversized/invalid RPC, timeout and Unicode hash rejection.

## Reproduction

```sh
# judges/run.py — full application gates, including real Metal capture and signposts.
python3 judges/run.py
# judges/model_ab.py — separate model-dependent A/B; the output directory must be new.
/Users/stewartbarteau/.local/share/uv/tools/rapid-mlx/bin/python judges/model_ab.py --output artifacts/qwen-ab-review --repetitions 3
```

For longer prompts add `--config fixtures/qwen-serving-long.json --prompts fixtures/qwen-prompts-long.json`. Diagnostic logits IO is included in Rust boundary time and excluded from the worker's per-step compute timer. Loading is excluded from the Rust admission-to-final-receipt window. Neither timing is an isolated native kernel benchmark.

## Real-model observations

Final source passed **83 Rust tests in each of the normal and debug-validation postures**, five TypeScript tests, strict Clippy, formatting, TypeScript 7.1/tsrs, native GPU capture/signposts, existing runtime/cache/resource judges, and new CLI rejection gates. See `evidence/model-integration-summary.json` and final source manifest.

| Workload | Codec | Median Rust seconds | Speed vs regular | Total-state compression | Minimum generated-token position match |
| --- | --- | ---: | ---: | ---: | ---: |
| short | regular | 2.0662 | 1.0000x | 1.0000x | 100.00% |
| short | rapid-turboquant-v4 | 2.0776 | 0.9945x | 1.0029x | 59.46% |
| short | rapid-turboquant-k8v4 | 2.1529 | 0.9597x | 1.0051x | 59.46% |
| long | regular | 3.0027 | 1.0000x | 1.0000x | 100.00% |
| long | rapid-turboquant-v4 | 2.9864 | 1.0054x | 1.0903x | 100.00% |
| long | rapid-turboquant-k8v4 | 3.0486 | 0.9849x | 1.1724x | 100.00% |

Each repeated matrix used three rotated-order runs per mode; load is excluded and diagnostic probe IO is included. Short: four prompts, at most 16 generated tokens. Long: two retrieval prompts, approximately 1,130 consumed tokens each, 256-token chunks. Both longer answers were preserved in all modes: `MARBLE-714` and `37`. State compression includes recurrent state and actual cache payload arrays, rather than quoting only packed attention indices. It does not include model weights or every allocator overhead.

Short regular outputs were stable across repetitions. Both lossy codecs had a minimum 59.46% generated-token position match despite small first-resumed-step KL (maximum about 0.00293). This is output divergence, not proof that every divergent answer is incorrect. Long K8V4 passed the declared diagnostic gates, with 1.1724x total-state compression (about 14.7% fewer state bytes) and 0.9849x wall speed (about 1.5% slower). These small samples have no sustained-load or confidence-interval qualification. Regular therefore remains default.

Repeated A/B evidence is [short](../artifacts/qwen-model-ab-v2/summary.json) and [long](../artifacts/qwen-model-ab-long-v2/summary.json). Their exact measured binary is preserved at `artifacts/qwen-ab-measured-rust-server`. A final self-review then corrected panic admission telemetry and added its regression test; the model worker, numerical path and scheduling decisions did not change. The final source reran the full gates and all three model modes: [final-build smoke](../artifacts/qwen-model-ab-final-smoke/summary.json). Its generated tokens matched the repeated-build outputs exactly for every mode. Do not relabel the earlier timing samples as measurements of the final binary; the binary/source distinctions are explicit in `evidence/model-integration-summary.json` and artifact hashes.
 The four short prompts test basic output congruence, not a representative task-quality corpus. Longer prompts test two known-answer retrieval cases around 1,130 consumed tokens. Even repeated diagnostic passes cannot establish general quality neutrality or production SLO performance.

# Was It Proven Necessary?

A real model path was necessary to evaluate state handling and meaningful inference. The A/B distinguishes host kernel correctness, generated-output congruence, and end-to-end state compression. None of these alone establishes the gold-standard serving goal.

# Deployment / Rollout

Experimental CLI only; regular KV remains default. No existing runtime, model weights or server ports were replaced. No production random-validation claim is made. Source and evidence manifests identify the tested build. Draft documentation has not received independent approval; no merge or production promotion is claimed.

# Next Stage of Implementation

1. Broader held-out quality and long-context tests, with workload-specific thresholds set before testing; repeat thermal/power-controlled SLO runs.
2. Tensor-batched hybrid Qwen execution, per-lane deadlines/fairness and streaming/cancellation endpoints.
3. Completion-safe Spec 15 prefix reuse with exact recurrent checkpoints and codec/layout/version identities; no global GPU drain for each audit.
4. Fused packed KV attention and measured codec/residency policies under one physical-memory broker.
5. Complete integer Qwen operator coverage, activation calibration and model-level comparison before claiming Q1.31 parity. The existing isolated integer kernels do not provide a full integer model baseline.
6. Qualified AMX/ANE backend capability gates, versioned model repository, metrics/tracing, failure recovery and independent release review.

# What We Learned

| Category | Finding |
| --- | --- |
| Technical | Hybrid recurrent memory limits the total benefit from attention-only KV compression. |
| Numerical | A small first-resumed-step logit divergence can still lead to later greedy-token divergence. |
| Ownership | Backend-native state needs its own managed contract; it cannot masquerade as host Q1.31 frames. |
| Measurement | Prefix compression, native packed attention, logical iteration scheduling and tensor batching are separate achievements. |

# Changelog

1.0.0 — 2026-10-22 — Codex — Design first, then bounded scheduler, typed codecs, actual Rapid/MLX Qwen lifecycle and diagnostic A/B evidence.


## Checkpoint correction — Fabel/Cyber qualification pending

The retained A/B used `/Users/stewartbarteau/models/Qwen3.6-35B-A3B-OptiQ-4bit` through `mlx_lm.load` without an adapter. It establishes only the recorded checkpoint/cache-codec comparison; it does not qualify Stewart's intended Fabel/Cyber fine-tune. The checkpoint may itself have upstream training/distillation provenance; “without an adapter” does not mean an untouched upstream Qwen release.

On 2026-10-22, inspection found `qwen-moe-fable-train` and Fable `adapter-smoke` / `adapter-nativeprobe` directories. The two adapter trees contained configuration files but no `.safetensors` weights. Their names and configuration are insufficient to establish the intended fine-tune. No training data was inspected and no candidate was substituted.

The next round first requires the exact fine-tuned checkpoint directory, or its base plus complete adapter weights. Keep the retained A/B artifacts unchanged as historical evidence. Before performance comparisons, fingerprint the intended weights, tokenizer, adapter configuration and adapter weights; reject missing/mismatched artifacts before admission. A separately loaded adapter must be part of model identity and cache identity, must be verified as actually applied, and must never silently fall back to its base.

After identity is established, run regular KV against V4/K8V4 on that same model with identical prompts and generation settings. Add held-out cyber/domain quality checks and output-congruence gates before throughput promotion. Then implement tensor-batched hybrid decode with per-sequence attention/recurrent state, cancellation isolation, bounded memory and single-versus-batched output equivalence. Approval to continue is recorded; fine-tune qualification remains pending identification of the actual artifact.

## Native-first execution decision — current

Stewart directed native Rust MLX execution with no Python/JSON backend, then selected `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined` for every model smoke. The previous worker is now excluded from default builds behind `legacy-python`. Its old A/B remains historical. The new [native tensor backend](NATIVE_MLX_BACKEND.md) has no full Qwen forward pass yet. Current legacy reproduction profiles point at the selected fused checkpoint; their current hashes therefore differ from the retained A/B's recorded profiles. This does not alter or relabel those benchmark artifacts.
