---
title: "Native MTP integration and tuning control plan"
document_type: "architecture"
version: "1.5.0"
date: "2026-10-23T21:27:14.642Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: ".gitignore"
    lines: "1-9"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-1148"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-106"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-64"
    role: "primary"
  - path: "examples/native_automaton_smoke.rs"
    lines: "1-394"
    role: "primary"
  - path: "examples/native_device_pressure.rs"
    lines: "1-196"
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
  - path: "examples/native_physical_batch.rs"
    lines: "1-232"
    role: "primary"
  - path: "examples/native_seed_qualification.rs"
    lines: "1-271"
    role: "primary"
  - path: "examples/native_speculative_smoke.rs"
    lines: "1-370"
    role: "primary"
  - path: "examples/native_suite_reference.rs"
    lines: "1-198"
    role: "primary"
  - path: "examples/native_text_stream_smoke.rs"
    lines: "1-57"
    role: "primary"
  - path: "examples/stream_wire_fixture.rs"
    lines: "1-34"
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
  - path: "fixtures/stream-v1.amst"
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
    lines: "1-64"
    role: "test"
  - path: "judges/device_smoke_report.py"
    lines: "1-38"
    role: "test"
  - path: "judges/generate_cordic.py"
    lines: "1-44"
    role: "test"
  - path: "judges/http_controls_verify.py"
    lines: "1-140"
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
  - path: "judges/native_batch_http.py"
    lines: "1-81"
    role: "test"
  - path: "judges/native_http_smoke.py"
    lines: "1-81"
    role: "test"
  - path: "judges/native_physical_http.py"
    lines: "1-93"
    role: "test"
  - path: "judges/native_seed_http.py"
    lines: "1-84"
    role: "test"
  - path: "judges/native_stream_disconnect.py"
    lines: "1-36"
    role: "test"
  - path: "judges/operator_reference.cpp"
    lines: "1-29"
    role: "test"
  - path: "judges/physical_throughput_verify.py"
    lines: "1-90"
    role: "test"
  - path: "judges/projection_reference.cpp"
    lines: "1-53"
    role: "test"
  - path: "judges/resources.py"
    lines: "1-35"
    role: "test"
  - path: "judges/run.py"
    lines: "1-241"
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
    lines: "1-113"
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
  - path: "native/qwen_batch.inc"
    lines: "1-41"
    role: "primary"
  - path: "native/qwen_device.inc"
    lines: "1-63"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-185"
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
  - path: "schema/stream.fbs"
    lines: "1-19"
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
    lines: "1-32"
    role: "primary"
  - path: "src/macro_bench.rs"
    lines: "1-81"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-472"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/batch_transaction.rs"
    lines: "1-591"
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
  - path: "src/serving/http.rs"
    lines: "1-685"
    role: "primary"
  - path: "src/serving/kv.rs"
    lines: "1-280"
    role: "primary"
  - path: "src/serving/mod.rs"
    lines: "1-119"
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
    lines: "1-1046"
    role: "primary"
  - path: "src/serving/native_device_scheduler.rs"
    lines: "1-205"
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
  - path: "src/serving/native_http.rs"
    lines: "1-460"
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
    lines: "1-207"
    role: "primary"
  - path: "src/serving/native_mtp_cli.rs"
    lines: "1-344"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-275"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-438"
    role: "primary"
  - path: "src/serving/native_mtp_scheduler.rs"
    lines: "1-1057"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-698"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-391"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_text_stream.rs"
    lines: "1-186"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-220"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-570"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-762"
    role: "primary"
  - path: "src/serving/stream_wire.rs"
    lines: "1-568"
    role: "primary"
  - path: "src/serving/weights.rs"
    lines: "1-395"
    role: "primary"
  - path: "src/stream_generated.rs"
    lines: "1-618"
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
  - path: "tests/http.rs"
    lines: "1-345"
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
  - path: "ts/generated/stream-wire/frame.ts"
    lines: "1-180"
    role: "primary"
  - path: "ts/generated/stream-wire/kind.ts"
    lines: "1-10"
    role: "primary"
  - path: "ts/generated/stream-wire/stop.ts"
    lines: "1-9"
    role: "primary"
  - path: "ts/generated/stream-wire.ts"
    lines: "1-7"
    role: "primary"
  - path: "ts/generated/stream.ts"
    lines: "1-5"
    role: "primary"
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
  - path: "ts/http-bench-config.test.ts"
    lines: "1-31"
    role: "test"
  - path: "ts/http-bench-config.ts"
    lines: "1-30"
    role: "primary"
  - path: "ts/http-bench.ts"
    lines: "1-55"
    role: "primary"
  - path: "ts/http-client.test.ts"
    lines: "1-18"
    role: "test"
  - path: "ts/http-client.ts"
    lines: "1-12"
    role: "primary"
  - path: "ts/http-metrics.test.ts"
    lines: "1-8"
    role: "test"
  - path: "ts/http-metrics.ts"
    lines: "1-15"
    role: "primary"
  - path: "ts/smoke.ts"
    lines: "1-1"
    role: "primary"
  - path: "ts/stream-inspect.ts"
    lines: "1-8"
    role: "primary"
  - path: "ts/stream.test.ts"
    lines: "1-28"
    role: "test"
  - path: "ts/stream.ts"
    lines: "1-47"
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
related_documents: ["SPEC15_DRAFT_ADMISSION.md", "NATIVE_QWEN_GENERATION.md"]
tags: ["mtp", "configuration", "qualification"]
---

## Executive summary

The designated Combined checkpoint contains an existing MTP sidecar. Integrate and qualify it as a Spec 15 proposer before commissioning a replacement head. This increment adds strict native sidecar ingestion and the native-mtp-inspect command. It does not execute or activate MTP drafting.

## Problem statement

The suffix-retrieval experiment accepted 25.31% of proposed tokens. This says nothing about this checkpoint's MTP acceptance. Its approximately 0.5% throughput difference was one short fixed-order measurement, not an established speedup. The user's Qwen over Dspark(2) setup is the intended comparison target; its live process, settings and artifact identity must be captured before comparison.

## Solution overview

Use one target verifier, one transactional state discipline and Spec 15 admission for all proposers. Optimize measured end-to-end cost per committed token while preserving quality. Reuse the supplied head where its architecture and quantization are verified; tensor presence alone establishes neither correctness nor profitability.

## Implementation details

Header inspection found 42 tensors in model-mtp.safetensors, 491,319,646 bytes. The evidence inventory records tensor names, shapes, dtypes, exact header digest and configuration overrides. It is a header inspection, not a complete tensor-data integrity or execution qualification.

The sidecar includes a BF16 2048-by-4096 fusion projection, two input normalization vectors, one attention/MoE layer and an output norm. Quantization overrides specify Q/K at 5 bits, V at 6 bits and selected shared projections at 8 bits; default affine quantization is 4 bits. Never apply the default uniformly. No embedding or vocabulary output projection appears in this sidecar; verify intended sharing and forward semantics against the owning architecture before implementation.

The current target graph projects the final normalized hidden activation directly into vocabulary logits. A device-owned hidden-state boundary must expose the representation required by MTP, including the correct pre/post-normalization convention and token alignment. Do not infer that convention from a name. Preserve target normalization sanitization: loading a separate MTP sidecar must not change the main-model norm-shift heuristic.

### Native sidecar ingestion

Run `cargo run --release --locked -- native-mtp-inspect`. This hashes the selected complete checkpoint, validates the exact 42-tensor supported layout and affine geometry, then opens and destroys a separate native MLX store. It reports ingestion only, not head execution or GPU numerical qualification. No Python worker or JSON transport is involved. Checkpoint metadata remains standard on-disk JSON. The new validator rejects duplicate metadata, missing/extra tensors, wrong dtype/shape, bad mixed packing, gaps, overlaps, trailing payload, truncation and symlinks.

Installed Rapid source (`rapid_mlx/spec_decode/mtp/qwen3_5_inject.py` lines 65 onward and the vendored `qwen3_5_mtp.py` prefill method) specifies post-normalized target hidden inputs and shifted tokens plus the bonus token. This is local implementation evidence, not independent numerical proof. Preserve those semantics when adding the native forward operation; raw pre-norm activations are not interchangeable. Separate sidecar loading currently does not set quantization overrides in a Qwen executable or perform projections; the forward implementation must do so explicitly.

### Current controls, actually wired

| Surface | Controls | Scope |
| --- | --- | --- |
| native-generate PROMPT MAX_NEW | --activation f32 or bf16; --device gpu or cpu; --choices yes,no | Native generation; FP32 is the qualified numerical profile |
| RUST_SERVER_IN_FLIGHT | 2 or 3 | Kernel runtime; not proof of a physical MLX command-buffer cap |
| DraftPolicy Rust object | max_proposers, work_limit, initial_mass, admission_mass, reward_mass, qualification_trials, probe_every_epochs | Programmatic automaton integration; no production policy defaults |
| Legacy config/model-run | JSON configuration and scheduler overrides | Explicit legacy-python feature only; not the native server configuration |

No native settings UI exists. fixtures/qwen-serving.json selects the historical Python adapter and must not be offered as a native launch profile. Native smoke example policies are benchmark inputs, not production defaults.

### Proposed unified control architecture — not implemented commands

Use one versioned native configuration profile, with typed CLI overrides. Suggested control names: --config, --print-effective-config, --check-config, --draft-proposer, --draft-max-tokens, --draft-work-limit, --draft-qualification-trials, --draft-probe-epochs, --max-sequences, --prefill-chunk-tokens, --max-batch-tokens and --state-budget-bytes. These names are proposals, not executable flags today. Prefer a human-readable non-JSON profile; select its format before adding a parser dependency.

Resolution order: explicit profile, then explicitly supplied CLI overrides. Reject unknown keys, duplicates, unsupported capabilities and incompatible numerical profiles. Print the resolved profile, provenance, artifact identity and fingerprint before measurement. No hidden adaptive defaults. Keep admission/trust controls distinct from the inactive proposed production receipt-sampling policy.

### Integration sequence

1. Validate complete sidecar layout, per-projection affine metadata and byte bounds; bind it into artifact identity.
2. Add device-owned, version-bound hidden-state handles and a native MTP forward operation with independent draft attention state. Determine token alignment and normalization from source, then test explicitly.
3. Implement MTP as DraftProposer under existing admission/completion guards. Include draft-state rollback, cancellation and cleanup; target KV, convolution and recurrent state remain one transaction.
4. Expose only operational controls through the validated native profile and CLI. Fail closed for unsupported MTP, paging, codec or scheduling combinations.
5. Integrate scheduler admission with memory budgets and continuous request turnover. Audit stale completion, fairness, starvation and cancellation before advertising continuous batching.

## Testing and adversarial review

Inventory checks: payload intervals are contiguous and nonoverlapping, tensor byte lengths match dtype and shape, every quantized projection has scale/bias companions and configuration overrides match packed geometry. These checks do not establish packing semantics or numerical equivalence. Integration gates must cover malformed sidecars, wrong normalization, token misalignment, stale hidden-state versions, quantization overrides, rejected drafts, exhaustion, shutdown and cancellation. Compare greedy outputs and unified state with no-draft target execution. Retain held-out quality failures as visible failures.

## Reproduction and qualification

Evidence: evidence/mtp-inventory/header-inventory.json. Inspect the selected checkpoint only; no Python serving worker is introduced. For performance, capture live Dspark(2) configuration and use the same checkpoint, tokenizer, prompts, generation limits and output contract. Alternate warmed runs and report uncertainty, not just a best run. Measure TTFT, inter-token p50/p95/p99, decode and aggregate throughput, committed tokens per target call, acceptance, draft overhead, memory and cancellation recovery. Run single and multiple lanes with isolated and mixed prefill/decode workloads. Acceptance is diagnostic; promotion requires quality and net throughput gains.

## Proven necessity and rollout

Mixed packing and missing hidden-state interfaces are concrete integration requirements observed in this artifact and source. Performance gains remain hypotheses. Leave current runtime behavior unchanged until head equivalence, rollback and held-out gates pass; retain no-draft fallback. Sidecar ingestion and the opt-in hidden-state boundary are now implemented. Next deliverable is native mixed-quantization MTP forward execution, not a promised acceptance percentage.

## Changelog

1.0.0 — Record actual MTP artifact inventory, current controls, proposed configuration architecture and promotion gates. Strict sidecar ingestion implemented; no target forward behavior changed.

Native ingestion result: the real sidecar opened successfully through the Rust-owned C++ MLX ABI. Validated payload: 491,314,304 bytes; header SHA-256: `231867bc50804aa05e7eb08648ec34e7162398be4aee0400c0d3afd55285cec5`. The full checkpoint identity remains `509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b`. Five focused tests cover all 42 missing-parameter cases, wrong default/invalid quantization, altered dtype/shape/offsets, extras, truncation, trailing bytes and symlinks. Sparse synthetic payloads are used for metadata tests; no numerical claims follow from them.

### Hidden-state ownership contract for the next increment

The current native device table publishes exactly 80 state arrays and accounts only KV/convolution/recurrent bytes. Do not append hidden tensors to that vector silently: snapshot inventory, reservations and reference comparisons assume its current layout. Introduce an explicit versioned payload carrying unified state plus optional post-norm hidden output, reserve its bytes before encoding, and publish both atomically. A handle must bind executor owner, model identity, position and numerical profile; stale or foreign handles fail before work admission. Speculative rejection must release candidate hidden and draft states while preserving the committed parent. Request cancellation retains all buffers through native completion. Keep hidden export optional on the default no-draft path so its allocation and numerical behavior remain unchanged.

## Device hidden boundary implementation — 2026-10-23

Implemented an explicit native DevicePayload containing the unchanged 80 unified-state arrays and optional post-norm hidden array. The existing StateId binds owner, generation, end position and full charged bytes. `hidden_block` charges all activation rows before encoding and atomically publishes them with the target state; rejected or canceled outputs retire the complete payload. No additional payload is retained by ordinary step/block calls. Snapshot of unified state excludes hidden activations, while `hidden_snapshot` explicitly exports only captured rows for diagnostics. Native production consumers will use the same worker-confined payload without CPU export. The API does not yet execute MTP.

The native_hidden_smoke example compares opt-in versus ordinary all-position logits and unified state bitwise at chunk sizes 1/2/4/8, checks post-norm activations against an independent f64 scalar RMS computation using actual BF16 norm weights, compares continuation, saturates four state slots and checks parent preservation and zero residency after release. Single-row versus batched vocabulary projection has a separate relative-L2 gate of 1e-5, consistent with existing verifier qualification; it is not claimed bitwise. An initial exploratory assertion incorrectly required those two different projection geometries to be bitwise equal; the final gate distinguishes them explicitly and retains bitwise requirements for matched geometry.

Local engineering distinction: the inspected Rapid explicit-sidecar injection path documents and enforces uniform packing inferred from fusion weights. This artifact has BF16 fusion/router weights plus mixed packed projections. Native integration must preserve each validated module override and dense BF16 projection rather than copy that uniform assumption. The sidecar normalization values and direct-load source indicate sanitized weights; qualify that explicitly rather than invoking the Qwen main-store MTP-presence shift heuristic.

Observed focused results: post-norm scalar maximum relative L2 7.650997966225019e-8; single-row/batched logit maximum relative L2 2.460363544993645e-7. Matched all-position logits and unified state were bitwise equal. Frozen default preservation: 60 cases / 360 steps, all six drift maxima zero, no residual residency; quality remains [15, 20, 23] in suites of [16, 20, 24]. No throughput is measured by this boundary qualification.

1.1.0 — Implement and qualify opt-in post-norm payload ownership, scalar normalization, bitwise default preservation and context-512 device pressure/recovery. MTP forward and performance remain unqualified.

## Native MTP forward design — next implementation

Open MTP explicitly; ordinary executor startup must not load or execute the sidecar. Keep target and MTP weight stores separate. Apply every sidecar module's affine override and support BF16 fusion/router projections explicitly. Use the sidecar's sanitized norm weights without the main-store MTP-presence offset heuristic. Fuse normalized token embeddings before normalized target hidden activations, then run the attention/MoE layer, sidecar output norm and shared target vocabulary head.

Draft KV and hidden output belong to an independent immutable state table, with distinct owner IDs and the same combined slot/byte budget as target state. Prefill consumes shifted tokens paired with exact rows from a captured target version; its cache position must equal that hidden window's start plus the selected row. Autoregressive extension consumes only the preceding draft version's final hidden row. No pooled GPU memory or hidden activations cross into CPU control messages. Rejected branches retire entire draft versions. Completion guards and semaphore permits persist through native completion; shutdown drains both tables. Qualify projected values, chunk/serial behavior, parent isolation, cancellation and exhaustion before presenting this operator as a usable speculative decoder.

## MTP operator implementation and precision finding — 2026-10-23

The native operator now implements sanitized sidecar norms, embedding-before-hidden fusion, gated attention, routed/shared experts and the shared target output projection. Native draft state has a distinct owner namespace, immutable KV and post-norm output, shared total slot/byte limits and independent retirement. Rust source handles retain exact input versions through completion. Open via `QwenDeviceExecutor::open_mtp`; ordinary `open` does not load the sidecar. `mtp_prefill` takes an explicitly shifted token sequence and selected target-hidden rows; `mtp_next` extends only its own parent. Diagnostic snapshots include draft KV and hidden values, and are not internal transport. These are Rust APIs, not new native-generate flags.

The first dense fusion scalar check failed at relative L2 4.01944918e-4 with ordinary batched FP32 matmul. A separate same-operand GPU microprobe measured 0.00356262 relative L2 for plain matmul and 1.48387e-6 for row-wise matrix-vector dispatch; disabling MLX_ENABLE_TF32 in a separate process reduced plain error to 1.97098e-6 while leaving row-wise error unchanged. This supports a reduced-precision dispatch explanation on this hardware. MLX documents potentially reduced float32 matrix-family precision in its [numerical precision guide](https://ml-explore.github.io/mlx/build/html/usage/precision.html). The chosen bounded row-wise dispatch avoids changing process-wide target settings. It passes the original scalar gate; no tolerance was widened to accept the failed fusion result. This is a numerical correction, not a measured throughput improvement.

Reproduce the full operator gate with `cargo run --release --locked --features debug-validation --example native_mtp_smoke -- NEW_DIRECTORY`. Debug probes sample eight channels per dense/affine projection and bounded routed expert rows/slots, validate normalization/routing/gates and use scalar attention checks. They do not compare every weight or constitute an independently implemented full-model MTP reference. The example tests 1/2/4/8-token prefill against serial steps, immutable branch reproducibility, alignment/vocabulary refusals, combined pressure, cancellation, parent preservation, recovery and the 512-position draft-context boundary. Debug CPU snapshots are deliberately outside hot-path ABI transfer counters; do not interpret those counters as proving no diagnostic CPU work.

The verifier integration must account for any first token already known from target logits separately from newly predicted MTP tokens; counting that seed as a learned draft success would inflate acceptance. Target verification/replay must also retain the hidden windows required for exact draft-state reconstruction after rejection. This operator does not yet provide those serving semantics, a scheduler profile, acceptance rates or throughput claims.

## Existing Dspark proposer inventory — 2026-10-23

User authorized using the custom Dspark already paired with the model. Local inspection found the cached `Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK` snapshot `815f706fe138402c9e446b2e2787da23ff7ba9e7`, separate from Combined's MTP sidecar. [Local inventory](../evidence/dspark-local-inventory.json) binds configuration, tensor-header metadata and installed reference source hashes. The 3,060,342,130-byte file contains 75 tensors. Header/configuration inspection does not verify every weight byte or establish compatibility with the fine-tuned Combined target. No second custom artifact has been identified.

This is a six-layer, hidden-2048 draft backbone with vocabulary 248,320, rank-256 learned Markov correction and a confidence projection over 2,304 features. Its configuration requests target layers `[1,6,11,16,22,27,32,37]`. The installed implementation applies token-transition logits sequentially across the predicted block and can estimate proposal confidence. These differ materially from the one-layer native MTP operator. The existing post-final-norm payload alone cannot satisfy this drafter: implement version-bound intermediate layer taps, their normalization/concatenation projection, distinct positional/attention geometry and draft cache semantics before advertising native Dspark support. Keep intermediate activations device-owned and charge their residency to the shared budget.

Integration order: finish MTP accepted/rejected-state synchronization under the existing exact greedy verifier; add the Dspark native weight/layout contract and eight target taps; port draft backbone, Markov correction and confidence math through Rust-owned MLX; qualify scalar operators and frozen full-graph proposals; then register a separate Spec 15 proposer identity. Confidence may trim draft work, never bypass target verification or commit state. Keep per-proposer cost, learned-token acceptance, rollback and epoch-decayed admission ledgers separate. Run MTP-only, Dspark-only and controller-arbitrated trials against identical greedy baseline outputs before selecting a default.

The local `~/opus-dspark-up.sh` launcher selects the older named checkpoint directory, `--mode dspark --max-draft 4 --kv-bits 8 --max-batch 1`; process inspection matched those arguments on port 8001. This is an observed comparison lane, not the native server profile or proof it serves Combined. Do not silently switch target weights or invoke its Python worker in the Rust path. User's Dspark(2) naming does not establish the live cap: the observed process uses 4. Recheck settings and complete weight/config identity before a matched benchmark. No service was restarted.

## Transactional MTP decode correctness profile

`mtp_start` captures prompt hidden windows, pairing each with the next prompt token or the target-greedy seed at the prompt boundary. `mtp_decode` proposes up to eight tokens including that known seed, verifies all positions using the existing exact greedy decision rule, discards provisional target/draft branches and replays the authoritative committed tokens from the immutable parent. Each replay window updates draft KV from target hidden activations and shifted committed tokens; the last shifted token is the next target-greedy seed. Published cursors bind synchronized target and draft positions. Cancellation cannot mutate the input cursor. EOS stops proposal extension; a terminal receipt cannot decode again. Context remains bounded to 512.

This initial correctness profile always replays, including full acceptance. It is intentionally not the profitable serving profile: measure target verification plus replay calls, autoregressive proposal calls and synchronization calls separately. `learned_proposed` and `learned_accepted` exclude the known seed. No throughput claim follows from the real-text gate. The next optimization should reuse accepted target hidden windows and draft prefixes without changing these transactional semantics. Spec 15 plugin completion guards and scheduler integration remain required before enabling production draft admission.

Reproduce with `cargo run --release --locked --example native_mtp_decode_smoke -- NEW_DIRECTORY`. The gate compares each emitted token against independent sequential target decode, checks committed unified state at the existing numerical tolerance and checks parent state bitwise, across widths 1/2/4/8. It records actual learned acceptance rather than synthetic oracle acceptance.

Observed initial real-text gate: all four widths passed sequential target token equality, committed unified-state tolerance and bitwise parent preservation. The simple sequence prompt produced learned acceptance counts 0/0 (width 1), 4/4 (width 2), 12/12 (width 4), and 15/21 (width 8) over four transactions per width, with EOS reached in the last case. These are prompt-specific smoke counts, not a representative acceptance estimate. Width 8 exercised rejected proposals and multi-window replay; target command counts include that replay. No elapsed throughput was measured.

## Accepted target-window reuse design

Capture hidden rows during target block verification. When every proposed token is accepted, the verified target state is already the authoritative state for that entire block: retain it and synchronize draft KV from its hidden rows instead of forwarding those same target tokens again. An additional bonus token still gets its own target/hidden evaluation and draft synchronization. If any candidate is rejected, discard the entire provisional unified state and replay all emitted tokens from the immutable committed parent; recurrent/convolution state is never truncated. A terminal accepted block needs no bonus evaluation. Count reused target tokens, replayed target tokens and total forwarded target tokens, in addition to commands. Draft branches remain provisional and are reconstructed from authoritative hidden rows, because recursively predicted draft hidden values are not target hidden values. Keep the previous unconditional-replay path behind debug-validation for same-parent numerical and accounting comparison. No default speculative serving activation follows.

`mtp_decode_bounded` caps the entire committed receipt, including a bonus/correction token; a budget of one must publish exactly one token without another target command. The correctness gate also refuses zero output budget and invalid EOS IDs.

Pilot measurement command: `cargo run --release --locked --example native_mtp_bench -- NEW_DIRECTORY`. It runs the four frozen speculative prompts, 32 generated tokens maximum, twice with reversed baseline/MTP order after warming both profiles. Both use the same loaded Combined target and greedy output settings. It asserts token-ID equality across profiles and rounds. Timings exclude checkpoint loading and tokenization, include actual prompt processing for first receipt, and report decode-only tokens/s plus receipt intervals. MTP publishes grouped tokens at the Rust API, so first-receipt time and receipt intervals are not HTTP streaming TTFT/ITL. The pilot is single-lane, has no continuous batching, does not prove system isolation and cannot establish sustained agent throughput.

## First warmed native MTP pilot — observed 2026-10-23

On Apple M5 Pro / 48 GiB, the four frozen prompts × two reversed-order rounds produced 256 output tokens per profile with identical IDs. Aggregate decode throughput (total tokens / total measured decode time) was 20.3506468748 tok/s target-only and 33.9698351940 tok/s MTP width 4, a 66.9226% pilot gain. Learned acceptance was 144/170 (84.7059%); the known target seed is excluded. Target commands fell from 256 to 112, while MTP also incurred 204 draft calls. Total target-forwarded tokens were 256 baseline versus 336 speculative: batching/amortization can improve wall time even when the target processes more tokens. Do not equate command reduction with proportionate arithmetic savings.

Mean first Rust receipt regressed from 656.484 ms to 802.862 ms because speculative preparation and grouped publication add work. Decode time amortized per emitted token was 49.1385 ms versus 29.4379 ms. Median of each run's receipt p50 was 48.7529 ms baseline versus 132.676 ms MTP; each MTP receipt can contain multiple tokens. These are not client TTFT, pooled percentile estimates or streaming inter-token timings. The pilot used a single lane, no continuous batching and no isolated-system guarantee. It is not a matched Dspark comparison, a held-out quality result, a sustained 1k/1500/2k window, or a production promotion.

Next serving work should overlap/decouple target-first-token publication from MTP initialization, add completion-guarded stateful proposer admission under Spec 15, and measure longer alternating windows plus real streaming latency and mixed request lengths. Keep acceptance and profitability separate: first-receipt regression matters even when decode throughput improves.

### Stateful Spec 15 adapter requirements discovered during integration audit

The existing `DraftPermit::finish` requires bound candidate count = proposed count and committed ≤ accepted + 1. That geometry fits retrieval, but a trained MTP block includes one already-known target seed plus learned proposals and may add one target bonus. Do not inflate learned acceptance to fit the old contract. Add an explicit bounded known-prefix count (0 for existing proposers, at most 1 for this MTP profile), require bound count = known-prefix + learned-proposed, and cap committed ≤ known-prefix + learned-accepted + 1. Keep learned rewards based only on learned acceptance. Bind the full candidate sequence to the parent/version key. The optimized path remains at most two target commands; the debug unconditional-replay reference is not an admitted production path.

Split MTP preparation from commit: an opaque prepared proposal retains the exact immutable parent, candidates and native completion guard; the controller binds candidates before verification. Commit must charge all admitted target commands even if later synchronization fails and must retain the draft permit through every target and MTP native completion. Caller cancellation reports zero published progress while the detached completion/accounting job drains. State publication remains atomic across target and draft. A baseline fallback must either synchronize draft state with explicit charged cost or invalidate it and charge rebuild cost on the next admission; stale draft reuse is forbidden. Namespace-scoped decode profitability must not hide MTP initialization or first-receipt overhead from end-to-end service reporting.

Counter definition: the current experimental `replayed_target_tokens` field counts committed target tokens evaluated after the verifier, including an additional bonus. `target_forward_tokens` counts verifier tokens plus those evaluations. Use the explicit definition when interpreting raw smoke data; future serving telemetry should split correction/replay from fresh bonus tokens.

## Stateful MTP admission implementation

`AutomatonMtp` registers the existing mixed MTP path as a Spec 15 workload with its own maintenance-cost profile identity. `mtp_prepare` retains the exact cursor and completion guard, generates candidates, and exposes them only to the adapter for branch-key binding. `mtp_commit` verifies and synchronizes atomically. Every target command receives failure-inclusive accounting; every target and MTP command retains the admission guard through completion. The detached job publishes its result and finishes the existing DraftPermit ledger, or records zero published progress if the receiver was canceled. Runtime abort leaves unfinished accounting visible through the permit lifetime.

`DraftOutcome.known_prefix` is bounded to one; binding covers known-prefix + learned-proposed tokens. Ledger known-prefix, proposed and accepted counters are separate. Committed output can include known-prefix + accepted learned tokens + one target bonus. Learned trust rewards ignore known-prefix. Existing proposers explicitly use zero known-prefix. Benched or queued work falls back to one target token and charged draft-state maintenance; queued tickets are canceled rather than silently retained. This maintained-state baseline is isolated from the retrieval adapter's cheaper no-maintenance baseline through profile identity. Prompt/MTP initialization is outside decode admission accounting and must remain included in service first-receipt reporting. The adapter does not add streaming, continuous batching or cross-profile proposer arbitration.

Observed stateful admission smoke: three admitted transactions bound three known seeds and 11 learned proposals, accepted 11 learned tokens and published 16 total tokens across five admitted target commands. Canceling the next result receiver increased observation/cost and attempted seed/proposal counts while published committed tokens remained 16 and unfinished stayed zero. A separately injected post-verifier allocation failure recorded one admitted target command, one attempted known seed, one learned proposal, one failure and zero accepted/committed progress. Shutdown logical bytes were zero. These are constructed controls on one prompt, not an acceptance or performance estimate.

1.3.0 — Record accepted-window pilot and implement stateful MTP admission, separate known-prefix ledger geometry and native completion-owned preparation/commit.

## Bounded scheduler streaming contract

The current shared Engine has only a final oneshot receipt; its backend contract returns one token per row. Add optional bounded token streaming there before connecting speculative grouped work to it. `submit_stream` explicitly selects a capacity in 1..128. Each token event binds sequence, monotonically increasing index, token ID and elapsed time from admission. A sole scheduler producer publishes only after whole-batch validation and records the same timestamp in the final generation. Never silently discard an event. A full stream pauses only that sequence before further native work; other lanes continue. Consumption/cancellation/admission/shutdown wake the scheduler through a shared Notify, avoiding a polling hot loop when every stream is blocked. Dropping the request cancels it. Calling final `result` drains any remaining token events so bounded backpressure cannot deadlock that convenience method.

Orderly shutdown drains readable requests but returns an explicit error for a stream that blocks further work without being consumed, releases its state, and preserves its already-enqueued prefix. It must not wait forever on an undrained bounded stream. This is Rust API token streaming, not an HTTP protocol. Existing submit/choices and backends retain their final-receipt behavior. MTP serving still needs a backend bridge that distinguishes its internally verified token bundle from per-token scheduler publication and honors the full remaining output budget.

### Shared Engine MTP bridge

`ScheduledMtp` retains request-local prompt staging, a private automaton cursor, verified pending output IDs and a virtual consumed position. Prompt chunks accumulate until complete; initialization then runs the native prompt and consumes/publishes one maintained-state baseline token. Decode validates the scheduler's last published input ID and virtual position. If pending IDs exist, it publishes the next already-verified ID without another model call; otherwise it requests a bounded automaton transaction and buffers its verified receipt. `StepInput.remaining_output_tokens` bounds the full speculative receipt, so hidden GPU state never advances beyond requested output. Stop/cancellation releases unpublished buffered IDs and the entire private cursor. All rows validate before work and new rows publish atomically after successful backend completion.

The bridge advertises context 511 because it consumes the last returned output into a native context of 512, unlike the older next-logit-only backend. It supports at most three request-local lanes, explicit width 1..8 and regular FP32 state. It rejects choice masks and external history, and enforces `prompt.len() + tokens.len() + remaining_output_tokens <= 512` on every prefill chunk. Storage reservations include a conservative per-lane budget plus workspace; model weights remain outside that logical state budget as in the existing native adapters. Iteration scheduling is shared, but GPU kernels still execute per row: do not call it physical tensor batching. Controller committed counts refer to its internal receipt, not streamed end-user delivery.

1.4.0 — Implement `ScheduledMtp` (`src/serving/native_mtp_scheduler.rs`) and bounded scheduler token streaming (`Engine::submit_stream`), fuse hot-path logit/argmax scans, eliminate per-step 993 KB final-logit vector clones, and wire concurrent batch + streaming verification into `examples/native_mtp_automaton_smoke.rs`.

## Native MTP CLI and audit driver

`native-mtp-generate PROMPT MAX_NEW` requires explicit `--draft-width`, `--stream-capacity`, `--qualification-trials`, `--probe-epochs`, `--initial-mass`, `--admission-mass` and `--reward-mass`. These set actual backend/controller fields; there is no implicit production policy. The local command fixes the existing bounded draft-metadata scaffold to capacity 8 × 64 bytes, 64 branches, 4,096 declared IDs, 16 leases, queue depth 4, wait 3 epochs, anchor 1,073,741,824, dead threshold 2, rate masses [268435456,1073741824,2147483647] and pressure multipliers [1,2,4] in Q31. It does not place model KV payloads in that metadata pool or claim cache coverage. Draft registration is bounded to one proposer and work limit three. Native state budget is 1 GiB with up to 64 version slots; Engine reserves 128 MiB/lane plus 512 MiB workspace for at most three lanes. Weights are outside this logical state budget.

The CLI owns its controller and advances one audit epoch after scheduler drain at explicit Normal pressure, with native logical state bytes supplied as the outside-cache estimate. Auditing per consumed event failed because the worker may already own another native lease. Live decay/probing requires a scheduler-owned quiescent audit hook and remains unfinished. Other embeddings of ScheduledMtp must provide an audit driver appropriate to their shared controller; blindly auditing an unrelated shared payload cache with an empty live set is not supported. Engine configuration fingerprints cover serving settings, not controller policy. The CLI prints the complete policy, model identity and width; preserve that header and full command with measurements. A unified configuration fingerprint remains a promotion requirement.

Observed shared-scheduler smoke: three lanes completed exact output limits 1/8/16 with IDs equal to the sequential target; the final scheduler had zero failed/canceled requests and zero logical used bytes. A second run held a capacity-one stream while another request finished eight tokens, then canceled the held stream; one completion and one cancellation were accounted and native shutdown bytes were zero. Token timestamps include grouped verification gaps followed by rapid buffered publication. They must not be interpreted as uniform per-token model execution or an HTTP benchmark.

## Preregistered live audit boundary

Add a default no-op asynchronous backend boundary hook to Engine. Invoke it exactly once after successful whole-batch validation and before publishing any token. Failed steps and malformed batches never audit. Hook failure closes admission and fails the batch before publication. ScheduledMtp may opt into one Normal-pressure epoch per validated batch only for its exclusively owned draft-metadata controller; refuse controllers with payload branches/blocks/reservations. Native completion guards must be gone at this point. Buffered publication batches also advance epochs: this is an explicit batch clock, not a target-call or token clock. Cancellation retains existing cleanup semantics. No production sampling policy changes.

Controls: generic Engine tests must prove hook ordering, error refusal and no audit on malformed output; native smoke must preserve exact three-lane outputs, cancellation and zero resident drain with live audits enabled. Re-run default/debug/minimal suites, strict Clippy and native/GPU judges. CLI uses this explicit experimental batch clock; throughput must be remeasured rather than transferred from the earlier pilot. Shared payload-cache audit planning and adaptive pressure remain outside this metadata-only hook.

Implementation of the preregistered boundary is now present: `ModelBackend::validated_boundary` is awaited by Engine between validation and publication, and `ScheduledMtp::with_exclusive_metadata_audits` enables Normal-pressure batch epochs. The CLI and native three-lane smoke opt in. This supersedes the earlier drain-only CLI limitation above; a controller shared with other runtimes or payload-cache branches still requires a global live-set planner and is not supported by this adapter. `AUDIT epochs=...` reports the actual controller clock after CLI shutdown. No new environment variables or production sampling modes are introduced.

## Preregistered scheduler throughput matrix

Measure the designated Combined model through Engine token streams with identical greedy prompts under target-only ScheduledDevice, ScheduledMtp without live audits, and ScheduledMtp with Normal-pressure validated-batch audits. Use regular FP32 state, draft width four, stream capacity one, one/three concurrent request lanes, 64-token request budgets and aggregate checkpoints at 1,000/1,500/2,000 delivered tokens per cell. Tokenization, model load and per-profile warmup are excluded. A fixed four-prompt sequence repeats; all measured outputs must equal a separately computed 64-token target reference prefix. EOS is honored and short completions count actual tokens; a no-progress request fails rather than fabricating throughput. Two rounds reverse profile ordering to expose drift; report raw request token times, receiver timing, complete policy, audit epochs and learned-only acceptance ledger. Ledger target-call counts cover admitted speculative transactions only and must not be mislabeled as total native calls.

Macro windows include request turnover and host orchestration. TTFT is scheduler-admission to first published token; publication ITL and receiver-observed ITL are distinct and include near-zero gaps when a verified bundle drains. Aggregate post-first-token speed excludes each request's first token and is not total wall throughput. No HTTP, physical tensor batching, long-context, serving-isolation or quality-generalization claim follows. Save per-cell evidence before proceeding, preserve failures and include warmups/target references separately. Compilation, bounded metric-unit tests, parity and zero-resident cleanup are gates before publishing numbers.

Matrix workload interpretation: request lanes are submitted in synchronized groups and replenished after the group completes. This measures bounded concurrent stream groups; it is not a Poisson/open-loop arrival test or sustained continuous-batching qualification. The target-only adapter predicts the final output without consuming it into state, while MTP maintains target/draft state through committed output; that extra work is included rather than subtracted. An existing external Dspark service is resident on the shared host; process inventory is retained and no service is restarted for the experiment.

Observed audit-clock consequence: single-lane cells advanced 2,216 measured audit epochs; three-lane cells advanced 834/840. Output parity held, but controller decay exposure depends on row packing and consumer scheduling. Keep the no-live-audit profile as a control. Clock cadence and lane-normalized alternatives require explicit tuning/qualification rather than treating these per-batch rates as universal production settings.

## Preregistered text streaming boundary

Before HTTP transport, implement append-only incremental text at the existing native tokenizer I/O boundary using the installed tokenizers 0.23.2 DecodeStream primitive. Never decode each token independently: a UTF-8 code point may cross token boundaries. Bound each stream to 512 token IDs, reject unknown IDs, poison after a decoding error, and forbid push/finish after finalization. Hold incomplete UTF-8 until more tokens arrive. At finish, compare against canonical full-sequence decoding and emit only its remaining suffix; a truncated terminal byte sequence therefore follows the canonical decoder's replacement policy instead of silently dropping data. Do not emit speculative unverified tokens. Stamp checks occur on stream creation/finalization, not every token.

Tests must cover split two/four-byte Unicode, legitimate replacement characters, special-token skipping, newline/whitespace, incomplete terminal bytes, unknown IDs, token cap, repeated finish and append-only prefix failure. The Combined tokenizer smoke compares concatenated deltas with full decode for multilingual/adversarial text and checks each published prefix against final output. Wire the same decoder into native CLI token consumption, retaining raw token event timing and final receipt equality. HTTP remains a separate unqualified transport until cancellation/deadline and wire framing are implemented and tested; no JSON worker/transport is introduced here.

## Proposed native HTTP contract (not implemented)

Use a maintained bounded HTTP library rather than a handwritten parser. Local experimental listener binds an explicitly supplied loopback socket, with no default port or restart of another service. `POST /v1/generate` accepts a bounded raw UTF-8 text body (64 KiB maximum) and explicit output-budget/deadline headers; no Python/JSON request worker is allowed. The configured native profile/policy and full model identity must be discoverable independently of request content. Fixed output caps/context/admission must be checked before encoding. Unknown control headers/parameters, malformed budgets, unsupported media types and oversize bodies fail before model work. Bound connection/body-read time separately from model execution deadline.

Response contract should use versioned length-delimited FlatBuffers with generated Rust/TS bindings: verified token ID/index/sequence, publication elapsed time, optional decoded text delta, terminal finish/usage, and explicit partial-result error. Delta text stays at I/O only; inner scheduling uses typed IDs/state. No complete/finish frame may precede successful final receipt and canonical text check. The pending Unicode suffix must be represented in the terminal frame if final decoding needs it. One bounded network-output slot propagates backpressure to the existing request stream; socket close/deadline cancels the Request while the scheduler/native completion owner drains already-encoded work. Half-written frames never count as delivered tokens.

Required gates before HTTP claims: binary framing limits/version rejection and TS/Rust round-trip; slow reader/body attack; disconnect before admission/during prefill/verification/Unicode suffix; deadline races; no duplicate finish; malformed text/IDs; simultaneous streams; orderly shutdown with a stalled connection; real Combined parity and full transport throughput. This design preserves the existing no-JSON transport request and makes no OpenAI API compatibility claim.

## Preregistered AMST v1 framing

Implement the HTTP prerequisite as size-prefixed FlatBuffers, identifier AMST, version one. One metadata frame (full model identity and explicit profile), zero to 128 verified token frames, then exactly one Finish or Error. Tokens bind sequence, zero-based index, ID and monotonic publication microseconds; decoded text may be empty while UTF-8 is incomplete. Finish carries count/reason and final decoder suffix, only after canonical text equality. Error carries committed-publication count/code and partial-stream status; missing terminal is an explicit accounting gap. Maximum encoded frame payload 72 KiB, text delta 64 KiB, profile 4 KiB, total decoded text 1 MiB, 130 frames. No hidden JSON envelope.

Rust verifies FlatBuffers structurally before semantic use. TypeScript must independently preflight vtable/object/scalar/string bounds and fatal UTF-8 before generated accessors, because generated JS accessors are not verifiers. Both incremental parsers bound their pending buffer before allocation, handle arbitrary fragmentation/coalescing, reject unknown enums/versions, sequence/index/timestamp regressions, duplicate/trailing terminals, missing metadata, excess tokens and unfinished EOF; errors poison parser reuse. Bindings regenerate under existing flatc and TS 7.1/tsrs gates. Cross-language binary fixture/CLI round trips and deterministic malformed-input mutation tests precede any transport claim.

Wire the same frames into a separate `native-mtp-stream` command, retaining existing human-readable `native-mtp-generate`. Binary stdout must contain frames only; configuration/audit diagnostics go to stderr. A completed native receipt and canonical text equality precede Finish. Error/pipe failure must cancel the Request and drain scheduler/native state before returning when possible. HTTP sockets, deadlines and disconnect-specific behavior remain a separate unqualified step, not implied by a working file/pipe protocol.

## Preregistered HTTP/1 implementation

Use installed Hyper HTTP/1 with Tokio I/O/timers, one request per connection, a 16 KiB header buffer/32-header cap and explicit loopback socket. Configure 1..64 concurrent connections and a 1..30,000 ms header/body-read timeout; no default CLI socket or port. `POST /v1/generate` requires singleton `x-rust-max-new-tokens` (1..128), `x-rust-deadline-ms` (1..300,000) and `Content-Type: text/plain[; charset=utf-8]`. Deadline starts at handler entry and includes body/preparation; bounded synchronous tokenization is checked again before admission. Body cap 64 KiB. Reject query/absolute targets, nonnumeric/mismatched Host, Origin, content encoding, unknown x-rust controls and duplicate controls before native admission. `GET /health` and `/v1/info` expose readiness and AMST metadata.

Response uses existing AMST bytes, with one pending output slot plus HTTP/TCP buffers. Reserve network-channel capacity before receiving another scheduler token. Body drop wakes the producer and drops/cancels Request; native work remains executor-owned. Deadline/shutdown sends a typed Error if its bounded terminal-delivery grace (100 ms) succeeds; otherwise EOF is an explicit unfinished gap. Never discard a queued token to insert an Error and never claim channel acceptance is end-user acknowledgement. Finish follows final receipt and canonical text equality. HTTP/1 keepalive/pipelined reuse is disabled in this first local profile.

Shutdown stops accepts, wakes producers and closes connections within a configured grace; wait for producer ownership to end before shutting down Engine/native stores. Test with a synthetic gated backend that proves deadline/disconnect do not abandon an in-progress step, and then the actual Combined model. Gates include body/header pressure, malformed controls/UTF-8/context, numeric loopback authority, deadline before first token, disconnect after token, bounded slow-output stall, three concurrent streams, shutdown with outstanding work and zero resident drain. Existing AMST Rust/TS/parser/native checks remain required. No TLS, remote/authenticated service, OpenAI compatibility or HTTP throughput claim follows from these local qualification gates.

## HTTP performance qualification preregistration

Expose a required HTTP-only `--execution-profile target|mtp|mtp-audited` flag before model load, reusing the seven explicit policy controls. `target` selects the existing ScheduledDevice/Device state backend, not maintained MTP fallback; `mtp` and `mtp-audited` differ only by the existing validated-batch metadata audit hook. Stamp the profile into AMST metadata and record the final proposer ledger as plain numeric diagnostic fields after draining. No request selects or changes server policy.

First HTTP pilot: two separately started persistent profiles (target then mtp-audited), each with one warmup and 2,000 exact-reference tokens at one and three closed-loop lanes, cycling the four existing short prompts, 64 output tokens/request with final truncation to reach the exact budget. Preserve every validated frame and client monotonic arrival time, request start/end, metadata identity/profile and actual command/binary hash. Report client TTFT, frame arrival ITL distribution including burst zeros, post-first decode rate, aggregate request-envelope throughput and 1k/1500/2k cumulative windows. Native publication timings remain separate; HTTP metadata arrival is not TTFT. Learned acceptance excludes known seeds but cumulative shutdown ledger includes warmup and both lane cells, which must be stated. No physical tensor batching or continuous/open-loop claim follows. Prior immutable target token references provide exact-output controls; no prompt alteration is allowed.

Gates: parser rejects malformed profile controls before load; the TS client/benchmark requires successful Finish and exact token parity. Save partial raw observations and nonzero driver status on failure; drain only the owned server. Validate measurement arithmetic on synthetic timestamps before timing native traffic. The pilot has fixed profile order, shared-host interference, short contexts and one repetition; reverse-order/no-audit/open-loop sweeps and broader quality remain necessary before a production performance claim.

## Next physical batching design gate

Current ScheduledDevice and ScheduledMtp iterate rows and invoke the native executor per row. HTTP stream concurrency therefore does not establish batched tensor execution. Before replacing that loop, introduce a typed per-row transaction descriptor containing sequence/parent cursor, positions, committed-length and provisional draft span, with immutable model/tokenizer/quant identity. Pack compatible rows only; different positions require explicit per-row masks, and attention KV, convolution history and recurrent state must advance or roll back together. No shared mutable view may outlive a native completion fence or cross into another row after cancellation.

Qualification must compare batched versus scalar target tokens/state across heterogeneous prompt lengths, partial draft acceptance and rejected-draft rollback; inject one-row failure, cancellation and output backpressure while native work remains in flight. Verify row permutation invariance, no cross-lane contamination and zero charged state after shutdown. Record actual packed-row counts, padding cost, completion ownership and fairness. This is a preregistered next gate, not an implemented batch API or a claim from the current HTTP pilot. Native paging/compressed KV and Dspark target taps remain separate integrations after this state contract is proven.

## Reverse-order, audit-off and open-loop preregistration

Keep the Rust/native binary unchanged from the first HTTP pilot. Extend the offline TS driver with required explicit `--profiles` (ordered unique target/mtp/mtp-audited list), `--arrival closed|closed+open` and `--tokens-per-cell 1000..2000` controls, validated before creating files or spawning a process. Run `mtp-audited,mtp,target`, each with the same eight-token warmup and 2,000 exact-reference tokens at one/three synchronized closed-loop streams. Stamp all benchmark controls and the benchmark source hash separately from the native binary command. Preserve explicit cell start/end clocks and normalize host-capture trailing whitespace before sealing.

With closed+open, follow each profile's closed-loop cells with exactly 16 independent client attempts scheduled at absolute 250-ms intervals, each requesting 32 tokens. No attempt waits for a previous completion; all 16 remain accounted for and are awaited before shutdown. The server's existing limits remain three native lanes, bounded admission, eight accepted connections, and a 30-second request deadline. Count HTTP 503 as expected overload rejection using a typed status error; any other failure (including a valid binary Error, missing Finish, parity mismatch or unexpected HTTP status) fails qualification after every attempt settles. Require both successful work and rejection, and zero unfinished native ownership at drain. Preserve empty/prefix archives for rejected/failed attempts without calling them valid generation streams.

Report offered versus successful requests/tokens, per-attempt schedule/start/end and dispatch jitter, end-to-end completion latency, client TTFT/ITL and exact-output parity for successful work. Metadata arrival includes transport/tokenization/admission and is not a pure queue-time measurement. Client outstanding attempts are not native active lanes. Continuous arrivals under bounded admission do not prove physical tensor batching or a qualified continuous-batching implementation. Cumulative shutdown acceptance includes warmup, closed-loop and open-loop traffic and must not be mislabeled as a per-cell rate.

Synthetic gates must prove strict parser/ordering limits, absolute arrival scheduling independent of completion, exact final-window token budgets, typed overload recognition, bounded shutdown timeout ownership and finite timing arithmetic. Cancel the losing shutdown timer rather than retaining a referenced 60-second timer after a clean process exit. Retain the old fixed-order evidence as historical; reverse-order/no-audit results are new measurements, not replacements. Shared native/file compiler caches are not reset, and this remains one reverse-order repetition on a shared host with short prompts and unqualified broader model quality.

## First-token scheduling experiment — preregistered design, implemented opt-in

The current MTP prefill path stages the prompt, runs `mtp_start` (target hidden blocks plus shifted draft prefill), then runs `no_draft` before publishing the first token. That token is already determined by final prompt logits, but `no_draft` also consumes it into target/draft state before delivery. Candidate: publish the validated prefill seed first and represent its not-yet-consumed state explicitly, then perform the exact existing seed advance before any later speculative transaction. This moves work, not eliminates all of it; the next gap may worsen, so TTFT alone is not a promotion gate.

Use a typed SeedPending/Ready row state rather than pretending the cursor's materialized position already includes the published seed. Retain immutable completion-owned prompt state; the next input token must equal the published seed and virtual position must match. Advancing that seed may not publish it again, increment committed output twice, or open a speculative branch on a stale parent. A one-token/EOS request must release safely without unnecessary advancement. Explicit counters must separate prefill-derived seed delivery from deferred target/draft maintenance, avoiding fabricated target calls or double-counted controller progress.

Before enabling this candidate: compare old/eager and deferred full outputs and unified target/draft state; cover all four prompts, output lengths 1/2/4/8/64, EOS, cancellation immediately after seed, bounded backpressure, three lanes, failed deferred advance, repeated parents and rejected-draft rollback. Require unchanged native ownership/accounting and target parity. Benchmark both TTFT and first-to-second/p95 gaps and full request throughput in alternating order; reject a cosmetic TTFT win that causes unacceptable tail or total-throughput regression. Deferring *all* MTP prefill would additionally require independently leased hidden windows rather than retaining complete historical KV/recurrent states; that is a separate unimplemented design, not assumed safe by this seed experiment.

### Seed experiment implementation contract

Keep eager as default. Add an explicit HTTP `--seed-mode eager|deferred` control (deferred is refused for the target-only backend before model load) and an opt-in ScheduledMtp builder. Rows carry a typed Ready/SeedPending(seed) state; the cursor retains its actual materialized prefix and is not relabeled as already advanced. On the next decode, require empty buffered output, advance via the exact existing no_draft operation, require its sole emitted token equals the previously published seed, discard that duplicate receipt, then obtain the next speculative output. Failure leaves the old row snapshot unchanged and normal Engine poison/release rules apply.

Expose diagnostic counters for backend prefill-seed receipts, deferred advancement attempts/completions and pending-seed releases. These are not end-user delivery acknowledgements. The existing controller baseline records seed maintenance once when it actually runs; one-token/EOS/canceled pending rows incur no fabricated maintenance or target-call record. Engine output counts remain authoritative for delivered/committed tokens; controller baseline progress may lag a prefill-derived seed until its later maintenance. Counters must not call this another newly delivered token. This first version still performs shifted draft prefill before the seed; deferring all draft prefill is outside its scope.

## Physical batching stage 1 — typed scalar transaction preregistration

Audit: native/qwen_model.inc hardcodes batch dimension one in attention, GatedDelta and embedding reshapes. native/qwen_device.inc owns 80 state arrays per immutable native version, with completion evaluated before publishing a state-table handle. ScheduledDevice already delays map updates until all scalar rows finish, but this discipline is handwritten and separate from the proposed packing contract. Treating concurrent streams or a list of row calls as physical batching would be incorrect.

Implement a private typed PreparedBatch/CompletedBatch boundary for the target scheduler. Preparation owns cloned parent leases and typed ModelStepIr per row, validates the entire request set before model work, and derives compatible cohorts by exact kind, position and token count. Reject duplicate sequences, external histories, invalid output budgets/masks, resident-row overflow and parent-position mismatch. The model/config/quant identity is fixed by the owning ScheduledDevice executor; no caller-supplied cross-executor handles or identity labels enter this plan. Initial preparation may create empty Rust parent values but must issue no native work.

Execution remains explicitly scalar and preserves input order. Each result must have expected materialized position and finite full-vocabulary logits before selection; validate all rows before exposing any commit. PreparedBatch retains an exclusive borrow of the exact committed map through execution; CompletedBatch consumes itself to install every next parent together without accepting a replacement map. A compile-fail documentation gate proves the map cannot be mutated while that transaction is retained. On later-row failure, completed provisional leases drop while original map leases remain. Cancellation cannot interrupt native ownership: Engine retains the step future through completion, then applies release. This stage does not support SeedPending, MTP branches, padding/masks for heterogeneous positions inside one tensor, or physical tensor fusion. Cohorts are geometry metadata only and must not be reported as packed GPU work.

Preregister gates: duplicate/invalid later row causes zero execute callbacks; complete row permutations preserve sequence-to-parent/output mapping and deterministic cohort assignment; later native/selection/position/nonfinite failure leaves every committed parent unchanged and drops all provisional leases; success releases old parents only at atomic commit; native real-model target HTTP single/three-lane token parity, deadline, disconnect/recovery and drained memory. Keep the existing MTP suite and eager default unchanged. Run default/debug/minimal suites, TS 7.1/tsrs and strict Clippy. No throughput claim or new tuning flag is warranted for this refactor.

Stage 2 requires a separate native ABI and graph design for batch-shaped linear/MoE operations, per-row attention offsets/masks and recurrent/convolution views. Preserve a scalar control lane. Demand exact selected-token parity and unified-state comparison, row permutation/cancellation/failure isolation and measured packed-row/padding telemetry. Only after these gates may the command surface expose an opt-in physical mode; performance promotion needs alternating-order HTTP 1k/1500/2k windows and sustained one/three-lane arrivals without a tail/fairness regression. Numerical differences from batched matrix geometry must be measured, never hidden by relaxing token parity.

### Stage-2 graph/ABI sketch — not implemented

Initial cohort: B=1..3 target decode rows, one input token each, identical materialized position P, one native session/model/quant/FP32 activation profile. Preserve scalar fallback for all other geometry and the existing MTP adapter. Proposed native batch_step takes B validated immutable parent handles and B tokens, and returns B child handles plus B×248320 logits as one completion-owned result. Validate all descriptors, generations, vocabulary and combined slot/byte capacity before reserving any child. On reserve/evaluation/finite-check/publication failure, roll back every reserved child, including any already inserted payload; return no successful subset. Rust may expose no partial CompletedBatch.

The actual architecture has 30 recurrent layers and 10 full-attention layers, two state arrays per layer. Required packed layouts: hidden [B,1,2048]; convolution [B,3,8192]; recurrent [B,32,128,128]; each attention K/V [B,2,P,256], advancing to P+1. Q uses [B,16,1,256]. Flattening MoE to [B,2048] is valid only with independent top-8 routing, scores and expert reduction for each original row. Generalize every batch-one reshape/slice/diagnostic assumption; do not flatten batch into time or silently change recurrent update ordering. Equal P avoids a first-version padding/mask policy, but still requires per-row RoPE/state isolation and permutation controls.

Memory gate: slicing one row from a packed child may retain the complete packed allocation. A view's logical row size cannot be used as proof of owned physical bytes. Choose and qualify either explicitly detached row storage or a completion-safe shared backing lease with a charge retained until the final row releases it. Include gathered inputs, packed intermediates, output splitting and cancellation-retained backing in peak/budget telemetry. Do not assume contiguous() guarantees detachment. Native reserve rollback and ownership tests must cover one surviving row after siblings cancel.

A separate source-audit finding is the current per-layer eval+synchronize in qwen_forward_core. Deferring those boundaries could reduce host waits but grows the lazy graph and changes memory liveness. Treat that as a separate ablation with native state/finite-output/peak-memory/cancellation gates; do not combine it with the first batching trial and attribute a combined speed change solely to tensor packing. No gain is established by this audit.

### Native batch prototype implementation gate

Implement an explicit Rust decode_batch API and completion-owned worker command first; keep HTTP/scheduler selection scalar until native qualification passes. Accept 1..3 nonempty device parents with equal P in 1..511, one vocabulary-valid token per parent, matching executor ownership, and no diagnostic/hidden/MTP request. Execute one batch-shaped Qwen graph, preserving the existing per-layer completion boundaries. Do not change scalar default or introduce an unqualified server flag.

Child row views deliberately charge each surviving handle for the *entire* B-row packed state backing: bytes = B × scalar_state_bytes(P+1), with B such child charges reserved upfront. This conservatively overcounts by up to B and keeps the full backing charged after sibling release without relying on contiguous() detachment. The native resident limit remains a logical-state limit, not a process/weights/workspace physical-memory ceiling; report MLX active/peak separately. Validate every parent and token before reservation, roll back all reservations on any failure, and assign output handles only after all payloads publish. Rust installs retirement ownership for all returned handles before fallible receipt checks; dropped result receivers must retire all children after completion. Invalid descriptors/budget pressure must not poison admission; native computation failures do.

Native qualification uses the designated Combined checkpoint and unchanged prompts. Compare B=1/2/3 children against independent scalar calls from the identical parents, then repeated greedy advancement, reverse row order, unequal-position/foreign-owner/invalid-token rejection, bounded resident pressure and sibling release. Require exact selected-token parity at every comparison. Report bitwise equality and per-row full-logit/full-unified-state max absolute and NRMSE errors; numerical gate is NRMSE ≤1e-5 and max absolute ≤1e-3 for both logits and state (FP32 model lane, not the integer-only Metal contract). Any token or numeric failure blocks scheduler promotion. Drain all state, and retain failed observations. This native gate does not establish HTTP batching, MTP integration or throughput gains; benchmark only after it passes.

### Scheduler promotion preregistration

Expose opt-in `--tensor-batching scalar|physical` for target HTTP only; reject MTP combinations before model loading. Default remains scalar. Physical mode uses 2 GiB logical native and scheduler budgets, and reserves three times the scalar maximum per sequence (260,505,600 bytes), because packed row views retain the whole backing. At most three live parents plus three provisional children require at most 18 scalar backing charges (1,563,033,600 bytes). Workspace is separately 512 MiB; weights and transient MLX allocations remain outside this logical limit.

Execute exact-geometry cohorts transactionally: only multirow Decode cohorts at positive equal positions use the native packed graph; prefill, singleton and incompatible cohorts execute scalar. Preserve original response order and commit no parent until every cohort passes cardinality, position, vocabulary and finite-logit checks. Cancellation or a later cohort failure drops all provisional children. Count actual successfully completed packed calls/rows and scalar calls at the adapter boundary; these are completion counters, not intent or throughput claims. Require malformed-cardinality and later-cohort rollback tests plus existing cancellation/permutation gates, then real HTTP three-lane parity and zero drain before any performance claim. Physical mode remains experimental and unqualified until those live gates pass; MTP and per-layer synchronization remain unchanged.

Scheduler adversarial review: native packing may retain B-row backing after a sibling exits, so physical mode increases per-sequence reservation rather than charging only the visible row. Parents remain leased until the complete transaction commits; worst-case three packed parents plus three packed children fit the explicit logical budget. Group callbacks return owned children; cardinality mismatch, invalid logits, later failure and dropped futures therefore reclaim all provisional leases without changing committed parents. The scalar default uses its original input-order execution path. Adapter counters advance only after successful executor completion and can include later-rolled-back work; they do not represent committed tokens. Engine retains completion ownership during HTTP cancellation. Remaining limits: no native post-publication injected fault, no heterogeneous padding, no MTP packing, no sustained performance or held-out quality promotion from this functional gate.

### Physical HTTP throughput smoke preregistration

Compare target scalar versus target physical, order scalar/physical then physical/scalar, each with a fresh owned server and an eight-token warmup. Each server runs closed synchronized waves at one and three lanes with exactly 2,000 generated tokens per cell; retain cumulative 1k/1500/2k arrival windows. Preserve the four existing prompts, reference tokens, greedy target path, prefill chunk 8, stream capacity 1, three in-flight limit and per-layer synchronization. The only server differences are the explicit tensor-batching mode and its preregistered backing-aware logical reservation budget. Shared host/compiler caches and ordered one-before-three lane cells remain confounds; keep both orders, commands and binary hashes.

Extend only offline TS benchmark controls to label target-physical separately and verify the corresponding native metadata flag. Require every token to match the frozen reference, every archive to pass the Rust reader, zero unfinished native ownership/failed or cancelled requests at drain, and actual packed completion counters for physical runs (scalar must report zero). Independently recompute clocks, windows and token totals. Report client TTFT, callback/publication ITL, per-lane post-first decode throughput, aggregate throughput, open-arrival outcomes and total packed-call/row coverage. Draft acceptance is not applicable with speculation disabled. Closed waves are not continuous batching; bounded open arrivals exercise scheduler admission but do not prove sustained heterogeneous tensor scaling. Any quality/parity, drain or accounting failure blocks a speed claim. Do not promote physical mode by default from this short-context smoke.

### Throughput-bound source audit and next discriminator

The current 40-layer target graph calls eval plus synchronize inside every layer (native/qwen_model.inc), retains 80 state-finiteness reductions before returning, and exports 248,320 FP32 logits (993,280 bytes) per scalar step for native/Rust validation and greedy selection. The packed graph retains the same per-layer boundaries and full-state finite checks. This is a correctness-first native graph, not a qualified fused production implementation. In the first scalar one-lane HTTP cell, mean callback versus native-publication gaps were 49.427147 versus 49.427710 ms; that measurement does not implicate HTTP transport as the principal inter-token delay.

These observations do not attribute wall time to synchronization, finite checks, quantized projections, recurrent update, attention, MoE or full-vocabulary export separately. Next preregister native stage timing/dispatch counts and MLX peak memory before claiming a bandwidth or ALU ceiling. Isolate boundary grouping/deferred evaluation, then sampled versus full diagnostic state scans, then device-side greedy selection with lowest-ID tie/finite-policy parity; preserve immutable parent rollback and exact tokens. Keep any validation-posture change explicit and separate from the inactive production sampling-policy draft. Unequal-position packing requires its own RoPE/mask/state-layout design. Do not combine these ablations and assign their total gain to one optimization.

## Native execution-boundary ablation — preregistered 2026-10-23

Objective remains 100+ generated tokens/s and >=90% measured draft acceptance on the Combined checkpoint, with held-out quality and state safety. Neither is established. First isolate synchronization grouping before changing arithmetic, finite checks, precision, drafter or scheduler geometry.

Add explicit target-only `--layer-group 1|4|10|40`, default 1. The device executor owns the setting; snapshot reference and diagnostic trace calls always retain per-layer completion. Evaluate all pending activation/state outputs at each group boundary, synchronizing there and at final output publication. Keep every state finite check, FP32 arithmetic, full logits and immutable parent ownership. Both scalar and packed target graphs use the setting. Reject invalid/duplicate values and nondefault MTP combinations before weights load. No environment toggle or silent default promotion.

Preregister native qualification against the unchanged snapshot reference: all four frozen prompts, greedy continuation, full logits and unified-state comparisons, repeated parent use and packed B=1/2/3, existing foreign-parent/invalid-token/resident-pressure/cancellation gates. Require exact token equality; NRMSE <=1e-5 and max absolute <=1e-3 for state/logits; preserve any failed observations. Run separate processes for each group so MLX peak-memory counters are independent. Capture model/binary identity, wall-clock prefill/decode, MLX active/peak, zero ownership residue and full commands. Increased lazy-graph memory or any parity/rollback failure blocks promotion. A short native gate does not establish HTTP throughput or broad held-out quality. After the gate, benchmark HTTP 1/3 lanes with the same fixed-window protocol and both orders; retain group 1 as control.

Subsequent separate experiments: fused SDPA and GatedDelta kernels; device-side greedy selection; explicit validation-posture configuration; qualified FP16/BF16 activation lanes; efficient multi-token target verification and compatible MTP/custom proposer; heterogeneous state-safe continuous batching. Acceptance alone is not profitability: measure committed tokens divided by total draft+verify+commit time. Preserve integer-only custom Metal kernels as a separate numerical contract. No 100 tok/s or 90% acceptance claim until observed on this exact model/workload.

### Fused primitive feasibility gate (separate from boundary grouping)

MLX SDK 0.32.4 exposes C++ `fast::scaled_dot_product_attention` and `fast::gated_delta_update`. Inspect the matching upstream implementation before mapping parameters: gated delta expects the already-exponentiated decay, beta, normalized q/k and FP32 `[B,Hv,Dv,Dk]` state. First run a standalone native component judge at the real key/value head geometry, B=1/3 and T=1/8, comparing against the existing arithmetic recurrence. Check all outputs/state, repeated branches from the same immutable parent, and zero parent mutation. Component success permits a separately flagged full-model experiment only; it does not qualify model tokens or speed. Keep arithmetic changes out of the completion-group matrix. The source contract is https://raw.githubusercontent.com/ml-explore/mlx/v0.32.4/mlx/fast.cpp .

### Fused recurrent full-model ablation — preregistered before integration

Expose a separate target-only `--recurrent-kernel reference|fused`, default reference; do not change attention or activation precision. The native device constructor owns this selection and refuses fused CPU sessions. Reference/snapshot and diagnostic trace executions force the original recurrence and layer boundaries. Map the existing normalized q/k, v, already-exponentiated decay, beta and FP32 immutable state directly into MLX 0.32.4 `fast::gated_delta_update`; keep convolution, gating, projections and all output/state finite checks unchanged. Preserve the baseline executable for the remaining completion-group measurements.

Require the same four-prompt full-logit/full-state numeric limits and exact frozen tokens, plus packed B=1/2/3 and context/cancellation/branch isolation at the selected configuration. Keep the fused primitive's changes separate from group-only measurements. Benchmark fresh reference and fused runs at the same group, with repeated reverse-order controls and peak memory. No MTP, activation-precision or production-sampling change is part of this switch. If numerical/token gates fail, record the failure and leave the candidate unqualified; never hide it by widening tolerances after seeing results.

Fused HTTP functional gate: extend the existing physical HTTP smoke with explicit group/kernel arguments and verify those values in the actual TS client's metadata callback. Preserve one/three-lane exact reference tokens, deadline, disconnect/recovery, both archive readers and real packed completion counters with zero drain. This short repeated-prompt gate is not a throughput or sustained continuous-batching claim.

First candidate server measurement may use one explicit target-physical profile, 2,000 tokens at each of one/three lanes plus the existing open-arrival pressure test. Verify with an explicit `--single` offline reader mode that requires exactly that coverage, unchanged raw archives/clocks/reference tokens and declared group/kernel metadata. Label it absolute candidate throughput, never an A/B speedup or default-promotion result. The original two-profile/both-order contract remains required for comparative claims; the verifier's default continues to enforce it.

### Next design gate: persistent compiled decode blocks and host/wait attribution

The first fused recurrence remains far below the 100 tok/s objective. Do not attribute the remainder to bandwidth from a source audit. Next record bridge wall time, host graph-construction time, eval/completion-wait time and logits-export time without inserting new GPU barriers. Completion wait includes submission/dispatch and GPU waiting; it is not a GPU-only timer. Report instrumentation overhead against the same uninstrumented configuration.

Candidate after attribution: persistent MLX C++ compiled recurrent decode blocks under an explicit `--graph-mode eager|compiled-decode`, default eager. Cache functions within the immutable weight owner, keyed by layer, batch geometry, dtype, quantization and kernel mode; pass x/convolution/recurrent state as inputs, never capture a request parent. Restrict the first compiled lane to T=1, B=1/2/3 with fixed recurrent-state geometry, bounded to 90 layer/geometry entries per selected profile. Preserve the existing eager prefill/attention and snapshot controls. Offset-dependent attention must not be cached with a stale captured position. Ensure function-cache destruction precedes weight destruction and canceled receivers retain completion ownership.

Before activation: exact frozen and held-out token comparisons, existing strict numerical gates, repeated immutable-parent branches, B=1/2/3 state packing, cancellation, budget rejection and max-context pressure. Record cold compilation/first-token latency separately from warmed decode, plus cache entry/miss counts and MLX peak memory. A compiler-created FMA change is a numerical change, not presumed bitwise parity. No post-hoc tolerance widening. Only if the measured path remains GPU-bound should the next experiment prioritize projection/MoE layout or a separately qualified lower-precision model lane. Efficient multi-token verification and compatible draft scheduling remain required for the independent >=90% acceptance target.

## Execution attribution implementation contract — 2026-10-23

Implement explicit `--execution-timing off|on` for target-only profiling, default off. Use steady-clock counters inside the native weight owner for timed forward attempts/completions, host graph construction/bookkeeping, existing eval+completion waits, validation/logits export, wall time and actual completion groups. Add no GPU barriers. Instrument both scalar and packed paths, but exclude snapshot/trace controls. Counters are cumulative worker-confined diagnostics, not GPU timestamps or committed-token counters; failed attempts may leave unclassified wall-time residue. Report counter deltas across timed prefill/decode separately, and compare instrumented/uninstrumented exact outputs and overhead before causal attribution. Do not change numerical operators, finite checks or group boundaries in this stage.

Then, only after the host/wait split is observed, implement the already-designed persistent compiled recurrent decode cache. The upstream MLX 0.32.4 C++ compiler erases captured-function cache entries when their owning callable is destroyed; retain those callables in the weight owner and destroy them before weights. Source: https://raw.githubusercontent.com/ml-explore/mlx/v0.32.4/mlx/compile.cpp . A bounded per-weight function cache cannot capture a request state or stale attention offset.

Attribution observation before compiled-block implementation: the exploratory instrumented eager fused/group-10 run recorded 252 one-token forwards averaging 0.876 ms graph construction/bookkeeping, 36.574 ms eval/completion wait, 0.017 ms validation/export, and 37.476 ms instrumented wall time. It used the initial in-bridge timer implementation; the separately tested header refactor needs a fresh control. Host graph construction is small, so caching alone cannot explain a path to 100 tok/s. Proceed with compiled recurrent blocks as a bounded fusion/dispatch ablation because eval wait includes dispatch, not as a claim that graph construction dominates. Require explicit environment refusal if MLX_DISABLE_COMPILE is present or the default MLX device is not GPU, avoiding a mislabeled silent eager fallback. Existing compiled-function API owns captured callable lifetimes; do not capture parents. Baseline and compiled source/binary identities and profiler posture must be saved.

### Operator-attribution gate after compiled recurrence

Compiled recurrence alone has not moved warmed decode materially above ~26 tok/s. The next discriminator must measure separately: embeddings; recurrent input/output projections and state update; attention projections and attention reduction; MoE routing, selected expert projections and shared expert; final vocabulary projection; state finite scans. Add an explicitly diagnostic profile with stage completion barriers and separate counters. Stage barriers perturb overlap/fusion, so their sum is an attribution experiment, not production throughput or GPU-only timing. Compare the same outputs and unchanged eager arithmetic; never use instrumented stage timings to assert an optimized end-to-end speedup. Hold model/quantization/context/precision/prompt fixed and retain uninstrumented whole-forward controls before and after.

Only optimize the largest measured contributor. Candidate mechanisms include fused SDPA for the ten attention layers, routing/projection fusion and expert weight layout, device-side greedy selection, and a separately qualified lower-precision activation profile. Each requires an explicit flag, untouched snapshot control, exact selected-token checks, full state/logit numerical gates, frozen 60-case behavior preservation and cancellation/packed-state ownership tests. The snapshot baseline currently has 58/60 task successes; preservation must not be presented as 60/60 semantic qualification. Speculative verifier work remains separate: >=90% draft acceptance is not a throughput gate without draft length, accepted tokens per target call and total draft+verify+commit cost.

### Stage-profile implementation preregistration — 2026-10-23

Add `--operator-profile off|stages`, default off, diagnostic target/scalar/eager only; refuse physical batching, compiled decode, MTP and non-off `--execution-timing` combinations before model loading. The ordinary eager/fused profile remains unchanged. Stage mode inserts explicit eval+stream-completion cuts at embeddings, input norm, recurrent projections/prepare/update/output, attention projections/reduction/output, residual+post norm, MoE routing/up-gate/down/mixture/shared/merge, layer output, final projection and unified-state finite scans. Each cut records successful cut count, host construction interval and eval/completion-wait interval. Record the entire output inventory needed by later stages so dependency work is not silently reassigned. These are serialized diagnostic host intervals, not GPU timestamps, throughput or additive production latency; no nested stage timers. Any failed cut has no successful-cut claim, and the existing executor poisons admission on compute failure.

The native ABI gains an explicit profile selector and fixed-size per-stage counters. Disabled mode performs no stage clock reads or evaluations. Snapshot/reference/trace calls force stage mode off. Hold immutable parents and FP32 arithmetic unchanged; existing finite checks remain mandatory. Extend the four-prompt qualification runner to capture prefill/decode stage deltas, exact frozen tokens and strict full-logit/state comparisons. Require unchanged stage inventory/counts for each completed one-token decode, fail-closed flag tests, fake-clock ASAN/UBSAN tests and offline mutation readers. Run actual model attribution first; only then select the next operator change. Never use the deliberately serialized profile as an end-to-end performance score.

### User-edit audit and reposition — 2026-10-23

Audit the new weight-constant caches, SDPA, compiled post-MoE/multi-token blocks, target hidden-step output, margin-gated drafting and accepted-window reuse against the frozen model gates before throughput claims. Preserve the user's changes while repairing contract defects: full device-state finite checks must not silently disappear while the production sampling policy is inactive; execution-timing must not choose different completion boundaries; metadata must describe actual MTP settings; compiler inventories/caps must cover every owned function cache; the snapshot control must bypass new constant caches. Retain the initial candidate observations separately. Longer native ABI context bounds do not qualify or expose 262K serving while Rust IR/scheduler limits remain 512. Test target and MTP paths independently, exact tokens and unified state first, then TTFT/ITL/committed decode and learned-draft acceptance with seeds excluded.
