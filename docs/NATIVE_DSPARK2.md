---
title: "Native DSpark2 — cached Koopah and Combined target"
document_type: feature
version: "0.1.0"
date: "2026-10-25T03:53:19Z"
implementation_date: null
author: Codex
reviewers: []
status: draft
related_files:
  - path: "README.md"
    lines: "1-1518"
    role: supporting
  - path: "build.rs"
    lines: "1-71"
    role: config
  - path: "native/mlx_bridge.cpp"
    lines: "1-189"
    role: primary
  - path: "native/mlx_bridge.h"
    lines: "1-136"
    role: primary
  - path: "native/qwen_model.inc"
    lines: "1-378"
    role: primary
  - path: "native/qwen_device.inc"
    lines: "1-75"
    role: primary
  - path: "native/dspark.h"
    lines: "1-78"
    role: primary
  - path: "native/qwen_dspark.inc"
    lines: "1-39"
    role: primary
  - path: "judges/device_abi.cpp"
    lines: "1-82"
    role: test
  - path: "judges/native_cpp_bench.cpp"
    lines: "1-199"
    role: test
  - path: "judges/native_dspark_bench.cpp"
    lines: "1-73"
    role: test
  - path: "judges/native_dspark_gate.cpp"
    lines: "1-22"
    role: test
  - path: "judges/dspark_reference.py"
    lines: "1-48"
    role: test
  - path: "judges/native_dspark_report.py"
    lines: "1-49"
    role: test
  - path: "judges/test_native_dspark_report.py"
    lines: "1-34"
    role: test
  - path: "scripts/build_native_dspark.sh"
    lines: "1-16"
    role: config
  - path: "docs/NATIVE_DSPARK2.md"
    lines: "1-558"
    role: primary
  - path: "evidence/native-dspark2/abi-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/abi.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/accounted-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/accounted-integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/accounted-integration.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/accounting.json"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/artifact-config.json"
    lines: "1-63"
    role: supporting
  - path: "evidence/native-dspark2/bf16-accounting.json"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/bf16-checkpoint-accounting.json"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/bf16-checkpoint-head.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/bf16-checkpoint-head.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/bf16-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/bf16-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/bf16-integration-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/bf16-integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/bf16-integration.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/bf16-reference.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/boundary-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/boundary-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/boundary-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/boundary-reference.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/checkpoint-head.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/checkpoint-head.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/clippy.log"
    lines: "1-33"
    role: supporting
  - path: "evidence/native-dspark2/confident-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/confident.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/confident.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/dspark-before-lazy-markov.h"
    lines: "1-78"
    role: supporting
  - path: "evidence/native-dspark2/final-abi-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/final-abi.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/final-clippy.log"
    lines: "1-33"
    role: supporting
  - path: "evidence/native-dspark2/final-gate-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/final-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/final-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/final-integration-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/final-integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/final-integration.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/final-rust-debug.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/final-rust-default.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/head-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/identity.json"
    lines: "1-45"
    role: supporting
  - path: "evidence/native-dspark2/integration-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/integration.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/lazy-bf16-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/lazy-bf16-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/lazy-fp32-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/lazy-fp32-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/lazy-gate-build-fixed.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/lazy-gate-build.log"
    lines: "1-8"
    role: supporting
  - path: "evidence/native-dspark2/linkage.txt"
    lines: "1-4"
    role: supporting
  - path: "evidence/native-dspark2/offline.log"
    lines: "1-16"
    role: supporting
  - path: "evidence/native-dspark2/precision-abi-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/precision-abi.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/precision-clippy.log"
    lines: "1-33"
    role: supporting
  - path: "evidence/native-dspark2/precision-fp32-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/precision-fp32-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/precision-gate-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/precision-rust-debug.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/precision-rust-default.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/pressure-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/pressure-integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/pressure-integration.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/qualified-abi-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/qualified-abi.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/qualified-accounting.json"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/qualified-bf16.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/qualified-bf16.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/qualified-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/qualified-clippy.log"
    lines: "1-33"
    role: supporting
  - path: "evidence/native-dspark2/qualified-fp32.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/qualified-fp32.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/qualified-integration-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/qualified-integration.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/qualified-integration.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/native-dspark2/qualified-offline.log"
    lines: "1-16"
    role: supporting
  - path: "evidence/native-dspark2/qualified-rust-debug.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/qualified-rust-default.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/refactored-build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/refactored-gate.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/native-dspark2/refactored-gate.stderr"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/reference-packages.json"
    lines: "1-4"
    role: supporting
  - path: "evidence/native-dspark2/reference-scope.json"
    lines: "1-17"
    role: supporting
  - path: "evidence/native-dspark2/reference.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/native-dspark2/rust-debug.log"
    lines: "1-866"
    role: supporting
  - path: "evidence/native-dspark2/rust-default.log"
    lines: "1-866"
    role: supporting
related_issues: []
related_documents: [CPP_BRANCH_PROOF.md, DESIGN.md]
tags: [cpp, mlx, dspark, speculative-decoding]
---

# Native DSpark2 design and qualification

## Executive summary and problem

Implement the user-selected cached Koopah drafter against the exact
Qwen3.6-MoE-Combined target, entirely in native C++/MLX. Preserve target-authoritative
output and measured learned acceptance. The cached snapshot is pinned at
815f706fe138402c9e446b2e2787da23ff7ba9e7; its actual repository name omits v2.
Do not silently substitute another checkpoint or advertise artifact-version
claims based on the runtime's DSpark2 name.

## Implementation contract, before coding

Use the installed DSpark/DFlash reference as the executable math specification:
post-layer (zero-based) unnormalized target taps [1,6,11,16,22,27,32,37],
concatenate along channels; project16384->2048, RMS norm, six Qwen3 layers,
full noncausal proposal attention over committed context plus proposals. Apply
full128-dimensional NeoX RoPE at1e7, Q/K RMS normalization, final RMS norm,
autoregressive rank256 Markov correction and the trained confidence head.
Sample-from-anchor means width8 verification consumes7 draft positions (anchor
plus6 masks) and yields7 learned proposals. Raw artifact block8 means8 proposals
(max verifier9), which the current ABI cannot accept. Fail explicitly beyond
width8. No change to the native target's FP32 lane or numerical tolerance.

First implement a standalone FP32 draft math lane with immutable committed-context
KV. Proposal KV is ephemeral; only target context enters the persistent cache.
Return a new cache, never mutate the parent, and publish only after finite gates.
The caller commits that cache only after target verification; target rejection
must replay a committed prefix and supply its matching target taps next round.
Use the checkpoint's own embedding. Expose the output projection separately:
installed bind() replaces lm_head with the bound target head, so real target
integration must use the Combined quantized head rather than silently claiming
checkpoint-head equivalence. The standalone oracle can test the own-head lane.

## Gates and language choice

C++20/MLX keeps the serving path native. Python is a test oracle only, never a
model worker or serving transport. Compare independent installed reference
FP32 logits, hidden states, Markov transitions, confidence and committed KV over
multiple calls; include malformed descriptors, nonfinite inputs, immutable
parents and context boundaries. Require NRMSE<=1e-5 and maxabs<=1e-3 for FP32
component/state gates, exact greedy IDs, and frozen target outputs for integration.
No performance promotion without measured combined serving and acceptance gates.
This is an engineering port of local reference sources, not a literature claim.

## Reproduction, rollout and next stage

Build controls with the existing pinned MLX SDK. Record commands, source and
artifact hashes and gate outcomes in evidence/native-dspark2. Default HTTP flags
remain unchanged until complete proposer/rollback qualification.125 tok/s and
TurboQuant integration remain independent unmet gates. No deployment or merge.

## Changelog

2026-10-24: preregistered exact artifact, tap convention, cache/width semantics,
precision, head binding and qualification sequence before implementation.

## Target integration transaction

Add an opt-in native feature-capture step/block ABI; default step exports and
state accounting stay unchanged. Retain eight post-layer tensors in tap order,
charge their bytes before reservation, include them in finite gates, and publish
features only with a successful target state. Ingest only committed features
whose start position equals the previous drafter-context position. Store draft
contexts behind a separate bounded, owner/generation-checked StateTable. Proposal
queries never enter committed KV. Bind proposal logits to the live Combined
quantized lm_head. On partial acceptance replay the authoritative committed
prefix, ingest its matching features, and atomically replace caller-owned target
and drafter context only after both succeed. Invalid/stale/foreign handles and
budget failures must leave parents live and output handles unchanged.

## Confidence admission experiment (preregistered)

The initial fixed-width Combined-bound diagnostic does not satisfy the existing
learned-acceptance gate. Test the trained confidence head at threshold0.98:
admit only the contiguous learned prefix with confidence>=0.98, falling back to
an authoritative single target token if no learned proposal qualifies. Record
ALL generated proposals separately from admitted and accepted proposals; report
coverage and fallback count alongside acceptance. Never claim an acceptance win
by hiding discarded proposals or promote a throughput regression. This is a
fixture-only diagnostic; unchanged MTP remains the control.

## Output-head discriminator (preregistered)

The installed adapter replaces lm_head on bind, while the selected checkpoint
ships an explicit trained head. Test both as named native session modes:0 uses
Combined's live quantized head (reference bind behavior);1 uses checkpoint head
(the already-qualified standalone math lane). Keep output verification strictly
Combined-authoritative in both. Record head identity in every integration report;
do not conflate artifact-head and bound-head acceptance or choose a default from
these four fixtures alone.

## Trained draft precision discriminator (preregistered)

Keep the target and all target finite/output gates FP32. Separately test the
checkpoint's native BF16 draft lane: retain BF16 draft weights and cast captured
FP32 taps to BF16 at the draft projector boundary. The trained confidence head
casts its projection to FP32 before sigmoid. Compare the same installed BF16
reference, greedy IDs and caches without relaxing existing oracle thresholds;
reject the lane if it fails. Record precision separately from output-head mode.
Logical context charges may conservatively remain at FP32 size. No promotion
without the existing Combined-bound acceptance and throughput gates.

## Native Markov scheduling (preregistered)

The reference keeps autoregressive Markov selections as lazy device arrays.
Remove per-proposal host token exports/synchronizations: keep the seven-step
Markov dependency chain on-device, collect every row's finite check, and export
tokens only after a successful whole-block finite gate. This changes scheduling,
not token math or score validation. Require exact FP32/BF16 oracle IDs and the
same hidden/logits/confidence/KV thresholds before any new diagnostic timing.

## Final observations and acceptance decision

The native backend executes the cached drafter against Combined without a
Python worker, Rust scheduler, TS layer or JSON model transport. The final BF16
component gate is bit-exact with the installed reference across60 comparisons;
FP32 maximum NRMSE5.31e-8 and maxabs5.96e-8. Both include widths2/4/8, successive
committed-context updates,512-context boundary, exact greedy IDs, unchanged parent
KV and8 malformed/nonfinite descriptors. The512 boundary is a draft-component
check; the serving ABI rejects verification beyond remaining target capacity.

Target integration independently checks tap order against the target's per-layer
trace, exact feature-byte accounting, foreign/dead handles, cross-branch cache
rejection, slot-exhaustion rollback and zero final target/draft residency. Final
BF16 execution preserves all128 frozen output tokens. Its diagnostic rate is
54.65 tok/s with88/164 learned proposals accepted (53.66%). This FAILS the retained
166/172 acceptance-rate floor. Output congruence passes; proposer acceptance does
not. A fixed-width8 unfiltered proposal ratio is not the same policy as the MTP
control's filtered width4; this result is not proof the selected target is wrong.

Earlier FP32 bound-head execution accepts90/157; checkpoint-head accepts92/150.
BF16 bound/checkpoint-head pre-scheduling controls both accept88/164. Neither head
mode satisfies the gate. The trained confidence>=0.98 FP32 experiment accepts
51/51 admitted proposals, but admits only51/489 generated proposals (10.43%) and
runs at27.90 diagnostic tok/s. That admission experiment is also rejected: low
coverage and slower useful output cannot be hidden behind100% conditional
acceptance. All generated, admitted and accepted counts are retained separately.

These are four-prompt, single-pass integration diagnostics, not the earlier
alternating-order warmed512-token benchmark protocol, sustained HTTP, stochastic
sampling, or held-out calibration. Intermediate precision runs may overlap other
qualification work; do not use their rates as matched speed comparisons. The
final lazy-Markov diagnostic ran in the sequential final qualification script.
No default policy, HTTP flag, serving deployment or125 tok/s gate is promoted.

Final regression: native ASan/UBSan120049 checks (leak detection disabled and
upstream MLX uninstrumented), Rust release default207/debug-validation207,
strict Rust Clippy and56 offline tests pass. Clippy's native compilation retains
pre-existing SDK/unused-offset warnings; this is not a warning-free C++ claim.
Independent accounting checks raw frozen IDs, counter totals, head identity,
finite timings, rates, coverage and pressure flags; corruption tests reject
forged tokens, rates, scope and counters. The accounting output explicitly keeps
acceptance_floor_pass and combined_125_gate_pass false.

## API controls, ownership and reproduction

`rs_qwen_device_dspark_step` captures feature tensors, with last-row or all-row
logits chosen by its0/1 mode. `rs_qwen_device_dspark_snapshot` is diagnostic only;
production proposal ingestion uses the captured device arrays directly.
`rs_qwen_dspark_create` binds one target session and chooses0=Combined-head FP32,
1=checkpoint-head FP32,2=Combined-head BF16,3=checkpoint-head BF16. Other modes fail.
The target must be GPU/FP32/full-validation/context512. Constructor tensor checks
are specific to this architecture; they do not establish arbitrary checkpoint
compatibility. The harness/evidence pins config and full artifact SHA separately.

`ingest` returns a new bounded context handle; `propose` does not mutate context.
Context handles bind exact target parent descriptors, not merely equal positions.
All handles are worker-confined. Destroy the drafter before its bound target,
and the target before its weight store. The benchmark uses8 draft-context slots,
128MiB conservative FP32 logical context budget and1.5GiB target-state budget.
BF16 cache charges deliberately overcount physical bytes. Weight residency is
separate from those state budgets. Parent handles remain live until replacement
succeeds; failed reserve/validation leaves output descriptors unchanged.

```sh
export MLX_ROOT=/absolute/path/to/pinned/mlx
# Artifact directory is the cached snapshot named above.
scripts/build_native_dspark.sh /tmp/dspark-gate gate
/absolute/path/to/mlx-dspark/bin/python judges/dspark_reference.py "$DS_ARTIFACT" /tmp/dspark-oracle bf16
/tmp/dspark-gate "$DS_ARTIFACT/model.safetensors" /tmp/dspark-oracle/inputs.safetensors /tmp/dspark-oracle/expected.safetensors bf16
scripts/build_native_dspark.sh /tmp/dspark-native integration
/tmp/dspark-native evidence/decode-arch-kernel-proof/cpp-plan.txt "$DS_ARTIFACT/model.safetensors" /tmp/new-dspark-report.json bf16
python3 judges/native_dspark_report.py /tmp/new-dspark-report.json
```

Omit the final bf16 argument for the FP32 gate/Combined-head control; use
checkpoint-head or checkpoint-bf16 in the integration executable for explicit
checkpoint-head controls. Build confidence for the fixed0.98 admission experiment,
or abi for sanitized descriptor/state tests. Reports use exclusive publication;
choose a fresh path. Oracle tensors live outside the repository and are
recreated by the checked test-only source; input/output SHA256s and actual
reference package/source identities are retained in evidence. Model loading and
prefill are excluded from diagnostic decode timing; GPU shape warm-up is not a
separate matched control here.

## Adversarial review, rollout and next stage

Review hardened exact target lineage, stale-target rejection, full-validation
admission, output capacity, maximum verifier width/context, tensor inventory,
nonfinite payloads and transactional publication. Markov scheduling defers every
row's finite check to the whole-block gate before publishing any proposal IDs;
all score rows remain checked. Tokens are packed and the device synchronized
before host export. Proposal KV never enters the committed cache.

Keep the existing MTP path as the control. Next implement and qualify adaptive
confidence/cost admission with held-out coverage and matched warmed controls.
Do not infer robust acceptance or a performance win from these four fixtures.
Stochastic correction/EOS serving policies, MTP+DSpark controller composition,
HTTP qualification and TurboQuant integration remain unfinished. The present
work delivers a tested native backend and transparent failed acceptance gates,
not seamless production serving or the125 tok/s combined outcome. No merge,
push or deployment occurred; document remains draft pending independent review.

## Reference specification and change record

Math sources are the installed mlx_vlm speculative/drafters/dspark/{dspark.py,
config.py}, qwen3_dflash/dflash.py and models/qwen3_5/{language.py,
speculative_verifier.py}. Their local source hashes and package versions are
recorded with the artifact identity. This is a native implementation of those
local executable contracts, not a claim based on published benchmark numbers.

2026-10-24: native backbone/Markov/confidence, FP32 and trained BF16 lanes,
transactional device tap/cache ABI, bound/checkpoint head controls, device-side
Markov scheduling, reference/pressure/acceptance accounting and regression gates.
