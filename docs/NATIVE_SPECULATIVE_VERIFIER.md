---
title: "Native greedy speculative verifier and hybrid-state transactions"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-23T02:48:48Z"
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
    lines: "1-53"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-62"
    role: "primary"
  - path: "examples/native_automaton_smoke.rs"
    lines: "1-394"
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
    lines: "1-221"
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
    lines: "1-87"
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
    lines: "1-47"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-172"
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
    lines: "1-592"
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
  - path: "src/serving/native_automaton.rs"
    lines: "1-324"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-687"
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
    lines: "1-688"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-380"
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
    lines: "1-380"
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
related_documents: ["SPECULATIVE_RESEARCH_2026_10.md", "DEVICE_STATE_OWNERSHIP.md", "NEXT_STEPS.md", "ADVERSARIAL_REVIEW.md"]
tags: [speculation, verifier, native-mlx, recurrent-state, experimental]
---

# Executive summary

Built an opt-in native verifier that scores every candidate position and a Rust transaction that commits only an accepted prefix and target correction or bonus. The transaction preserves the model's KV, convolution and recurrent state together. A bounded request-local suffix drafter exercises the interface without a trained head. Initial measurements do not justify enabling it by default.

# Problem statement

The previous ABI returned only final-position logits, which cannot verify a speculative block. Keeping its final recurrent state after rejecting a suffix would silently commit rejected tokens; truncating KV alone cannot undo recurrence or convolution. Draft acceptance must be separated from actual time saved and from model quality.

# Solution overview

Native block evaluation returns position-major logits and an immutable provisional state. A private Rust cursor binds a state version to its next-token logits; callers cannot construct a cursor from unrelated logits. Verification compares the first proposal with cursor logits and each subsequent proposal with the preceding block row. No candidate is externally committed until the final state is available.

# Implementation details

## Files modified and line references

The generated metadata and FILE_MAP.md enumerate exact current source ranges. Primary changes are native/qwen_model.inc (optional all-position projection), native/qwen_device.inc and mlx_bridge.h (bounded block ABI), src/serving/native_device.rs (worker command and owned receipt), src/serving/native_speculative.rs (cursor/acceptance/replay/drafter), examples/native_speculative_smoke.rs, and judges/speculative_report.py plus its malformed-evidence tests. The four authored probe prompts are in fixtures/native-speculative-prompts.tsv. Original quality fixtures are unchanged.

## Languages and literature

Rust owns immutable cursors, async admission, transactions and token history. C++ calls the existing MLX C++ tensor API through a Rust-owned worker; Metal execution remains MLX-backed. Python appears only in offline evidence judges, never as a runtime worker or JSON transport. TS 7.1 and tsrs remain unchanged and must pass the existing compiler gates.

The explicit research review is SPECULATIVE_RESEARCH_2026_10.md, grounded in AgSpec (2610.01108), ASPIRE (2609.17943), H-Spec (2609.24197), and TreeWY (2608.20961), among others. This first mechanism implements linear draft/verify and bounded replay, not those papers' complete algorithms or reported performance. The existing full target graph and explicit FP32 attention remain authoritative.

## Acceptance and unified-state contract

Draft length is 0–8 and cannot exceed remaining context. Greedy selection uses the existing total-order floating comparison and lowest-ID tie rule, including signed zero. Every returned row must be finite and match the vocabulary, even if its candidate suffix will be rejected.

- Empty proposal: consume one greedy target token.
- First mismatch: emit accepted prefix plus the target correction; discard provisional state and replay those tokens from the original cursor.
- All accepted: append one target bonus using a native step from the block state, unless EOS or context leaves no room.
- EOS inside acceptance: emit through EOS and replay that prefix if the block included a rejected suffix. EOS is consistently consumed into the unified state. A terminal cursor rejects further generation.

The input cursor remains unchanged on failure or cancellation. Block and replay work retain Arc state ownership through worker completion. Provisional state drops through the separate reclamation channel. Budget failure after successful verification returns an error rather than publishing a partial cursor. Original input and final output versions coexist until completion; callers must provision resident slots for that overlap.

The block head uses a multi-row projection and the hybrid stack processes a chunk, so its floating reductions need not be bitwise equal to one-token execution. Measured preservation at a 1e-5 relative-L2 gate and exact observed token decisions is finite evidence, not a proof for unseen near ties. Stochastic distribution preservation is not claimed.

## Draft provider and configuration

SuffixDraft retains 16–65,536 committed token IDs in a request-local buffer. It matches suffixes of 2–16 tokens and proposes at most eight IDs. It does not train, install, or silently select an MTP head; no workspace/global corpus is indexed. Rejected proposals are not observed as committed history. Public callers must preserve that rule and isolate the history per request.

No defaults or production flags changed. The executor's existing three-work semaphore applies to individual native commands. A speculative transaction spans one or two commands; neither this semaphore nor three concurrent transaction probes establishes a physical command-buffer cap. The scheduler adapter and HTTP/client streaming do not use this decoder yet. Existing sampling-policy proposals remain inactive.

# Testing evidence

Initial Combined smoke: 21 free-greedy frozen executions plus four authored code/security requests, all with identical sequential and speculative generated IDs and final terminal status. Constrained-choice cases are explicitly excluded from speculative mode; the separate 60-execution snapshot/device preservation gate covers the unchanged existing path. Four authored cases stop at the configured 64-token cap; they are output-preservation probes, not completed autonomous-agent tasks or quality qualification.

Mechanical pressure covers widths 1/2/4/8 and all 19 rejection/full-acceptance boundaries, each against sequential unified state. It also covers EOS, rejected admission, cancellation while verification is pending and after verification before bonus completion, full resident capacity during partial transactions, recovery, three concurrent branches, and context 504→512. The context-edge full-acceptance case suppresses the bonus; the next decode rejects overflow. Native and Rust invalid-geometry/finite/accounting tests and sanitizer pressure are separate gates.

Observed block-vs-sequential maximum logit relative L2 is 2.1476678e-6 (max absolute 3.7908554e-5); maximum block state relative L2 is 7.5258901e-7 (max absolute 5.4836273e-5) in the mechanical row/state comparison. These are below the preregistered 1e-5 relative threshold; they are not bitwise parity. Diagnostic state exports are offline-only and excluded from the decode timers.

| Measured decode-only group | Requests | Tokens | Accepted / proposed | Target calls | Sequential tok/s | Retrieval tok/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Frozen raw | 16 | 47 | 1 / 8 | 47 | 20.44 | 20.11 |
| Frozen unconstrained domain | 5 | 13 | 0 / 0 (N/A) | 13 | 20.51 | 20.63 |
| Authored code/security | 4 | 256 | 61 / 241 (25.31%) | 233 | 20.34 | 20.05 |

All 25 requests combined: 62/249 proposed tokens accepted (24.90%), 316 committed tokens, 293 target calls. Reducing target-call count did not establish a speedup: four authored cases are roughly 1.4% slower in this fixed-order smoke, with no confidence interval or repeated sustained windows. Do not promote retrieval drafting on these results. TTFT, network latency, multi-lane sustained throughput, continuous speculative tensor batching and model-quality acceptance were not measured here.

Raw runs and derived accounting live in evidence/speculative-verifier/. The first short exploratory run is retained separately from the final 25-request run. Historical device-state evidence remains unchanged. Full regression, preservation and native-pressure outcome is recorded in that directory's test-summary.json; no separate independent review is claimed.

## Reproduction

```sh
cargo build --release --locked --example native_speculative_smoke --example native_device_reference --example native_device_pressure
target/release/examples/native_speculative_smoke NEW_SPEC_DIRECTORY
python3 judges/speculative_report.py NEW_SPEC_DIRECTORY
target/release/examples/native_device_reference NEW_PRESERVATION_DIRECTORY
target/release/examples/native_device_pressure NEW_PRESSURE_DIRECTORY
python3 judges/run.py
cargo test --no-default-features --locked
```

Use only the user-designated Combined checkpoint in fixtures/native-model.path. Directories must not exist. Run model measurements without concurrent GPU tests. Load and prompt prefill are excluded from the per-request decode timers; native EOS consumption is included. Fixed baseline-then-variant order and four 64-token probes limit interpretation. No training or adapter source is provided by this implementation.

# Was it proven necessary?

Per-position logits and correct rejected-state handling are necessary for this implementation's speculative contract. The test run establishes bounded mechanics and observed preservation, not that speculation is profitable or that a custom MTP head will improve this model. Retrieval's weak acceptance and extra replay cost motivate better proposals and a measured cost controller before promotion.

# Deployment and rollback

Explicit library API and offline example only; existing serving defaults are retained. Do not route production traffic into this lane. Rollback removes the new block API/example and keeps the old final-logit ABI unchanged. Source and binary digests bind the retained final run; model identity and profiling limits are recorded separately. No release, independent approval, or broad readiness is asserted.

# Next stage

| Work | Dependency and gate |
| --- | --- |
| Per-request profitability controller | Measure verifier/replay cost and committed progress; compare a no-speculation action, include tail latency and MoE expert union cost. |
| Native custom MTP or hybrid draft head | Audit available tensors; freeze train/validation split and training provenance; native execution plus matched-logit/acceptance tests. Do not silently activate checkpoint MTP tensors. |
| Reduce rejected-state replay | Derive factorized Gated DeltaNet updates and convolution rollback; compare every accepted state against this bounded-replay reference. |
| State-safe tensor batching | Per-sequence ownership/epochs, mixed draft/verify scheduling, cancellation and tail-latency pressure. |
| Paged KV and Monte Shuffle | Exact regular paging first; distinguish prefix reuse and query relevance. Prefetch changes placement, not which target contributions count. Sparse/early-exit modes require separate quality qualification. |
| Prefill/decode separation | Logical queues and bounded chunks first; measure contention over one shared weight store before cross-device handoff. |

# What we learned

| Area | Finding |
| --- | --- |
| Correctness | Candidate rollback must include recurrence and convolution, not just KV. |
| Numerics | Chunk/all-row evaluation agrees within the tested gate but is not bitwise sequential. |
| Performance | Fewer target calls and a nonzero acceptance rate do not imply higher throughput. |
| Scope | Trained MTP, scheduler integration and approximate dynamic forwards remain separate, unqualified work. |

# Changelog

1.0.0 — 2026-10-22 — Native block verifier, Rust greedy transactions, request-local suffix proposals and retained Combined A/B/pressure evidence.

## Final validation outcome

All gates completed: 126 default Rust tests, 126 debug-validation tests and 87
no-default-feature tests; all registered example tests; TS tests, TS 7.1/tsrs,
Clippy and formatting; existing Metal/CORDIC/resource/signpost/capture judges;
70,030 native ABI and 96,024 state-table sanitizer checks. Upstream MLX itself is
not sanitizer-instrumented. The complete 60-execution / 360-step default
snapshot/device preservation run remains bitwise equal in logits, layer traces
and state, with unchanged historical prompt/token/text/finish/quality results.
Quality remains 15/16 raw, 20/20 contract and 23/24 domain. The existing hardware
pressure suite also passes context, cancellation, queues, branch isolation,
restart and zero native residents at shutdown. No model-quality promotion follows.
