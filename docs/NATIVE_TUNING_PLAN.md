---
title: "Native MTP integration and tuning control plan"
document_type: "architecture"
version: "1.1.0"
date: "2026-10-23T05:04:01Z"
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
    lines: "1-65"
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
  - path: "examples/native_mtp_decode_smoke.rs"
    lines: "1-90"
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
    lines: "1-228"
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
    lines: "1-324"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-798"
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
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-108"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-417"
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
