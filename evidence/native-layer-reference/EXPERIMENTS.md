---
title: "Layer/state divergence and attention numerical-profile experiment ledger"
document_type: "research"
version: "1.0.0"
date: "2026-10-22T23:59:13Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: "src/serving/native_layer_reference.rs"
    lines: "1-93"
    role: "primary"
  - path: "src/serving/mod.rs"
    lines: "1-102"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-449"
    role: "primary"
  - path: "judges/layer_reference.py"
    lines: "1-185"
    role: "primary"
  - path: "judges/attention_profile.py"
    lines: "1-44"
    role: "primary"
  - path: "judges/attention_identical_inputs.py"
    lines: "1-72"
    role: "primary"
  - path: "judges/test_attention_profile.py"
    lines: "1-51"
    role: "test"
  - path: "judges/test_layer_reference.py"
    lines: "1-56"
    role: "test"
  - path: "judges/suite_reference.py"
    lines: "1-248"
    role: "primary"
  - path: "judges/run.py"
    lines: "1-211"
    role: "primary"
related_issues: []
related_documents: ["../../README.md", "../../docs/NATIVE_LAYER_REFERENCE.md", "../../docs/DEVICE_STATE_OWNERSHIP.md", "../../docs/ADVERSARIAL_REVIEW.md"]
tags: [fidelity, attention, provenance, state]
---
# Executive summary

The first material drift against stock is full attention in layer 3. Native
explicit FP32 attention is more accurate on the tested operands than stock fused
SDPA against an independent CPU FP64 oracle; native arithmetic is retained.
Changing only stock SDPA to an explicitly labeled FP32 control closes the observed
full-graph drift. This does not fix or hide either original quality miss.

# Design and implementation

Design preceded implementation in docs/NATIVE_LAYER_REFERENCE.md, including
observer controls and a separately labeled causal operator substitution. A new
opt-in native CLI exports existing trace/snapshot state at bounded chunk boundaries;
production serving defaults and the model forward math are unchanged. External
stock wrappers delegate to original module bodies. No installed source file is
patched. The control changes one in-memory function binding only.

Parent revision is f084105e3dd3fd90f0e27c45b8b39c5ca830037c. Tested source is bound
in source-sha256.json. Stock package/view identities are captured in each
provenance file; ordinary stock and controlled graphs are kept separate.

# Experiments and observations

1. Native s10/r11, chunk8, F32: every traced/ordinary logit and state control is
bitwise identical. Ordinary stock observer/untouched controls also match bitwise.
Layer0 recurrent reductions already differ slightly; first NRMSE >1e-4 is the
layer3 mixer, 0.000464039 / 0.000454299 on the first chunk. Earlier layer outputs
and layer3 K/V are around 1e-7. Full per-layer/state metrics are in stock/results.json.
2. Identical native layer3 input: projected stock K/V equal native bitwise.
Recomposed stock SDPA/gate/output equals the original stock module bitwise.
Explicit FP32 attention/gate/output equals the native mixer bitwise. CPU NumPy
FP64 oracle: native-style attention NRMSE ~1.53e-7, stock fast ~9.00e-4 / 9.24e-4.
attention-verified/results.json is the final-source repeat and is identical to
attention/results.json. No internal SDPA dtype or universal-accuracy claim is made.
3. Full s10/r11 SDPA-only control: final-logit NRMSE falls from 0.065367653 /
0.060410334 to 8.60248357e-7 / 6.60977616e-7. All controlled layer/state NRMSEs
are <=1.55e-6; the results are not ordinary stock baseline measurements.
4. Fresh native 60-case capture plus external SDPA-only control: all exact token
trajectories match. Maximum first-answer logit NRMSE 8.75550157e-7, median
6.18719145e-7, max absolute difference 2.09808350e-5 over 14,899,200 entries.
There are 44 unique prompts. Quality stays 15/16 raw, 20/20 contract, 23/24 domain.
All fresh native outputs/first vectors equal the preceding checkpoint bitwise.

# Engineering decision and next step

Retain the existing native accurate FP32 operator. Do not adopt the less accurate
stock SDPA numerical profile merely to erase drift. Future optimized profiles
require explicit names and quality/error/throughput measurements. The next
completion-owned device-state design is documented with immutable owner/slot/
generation descriptors, worker-only native destruction, resident-state accounting
and cancellation/branch/drain gates. It is not implemented device-state serving.

# Validation and adversarial review

118 default + 118 debug Rust tests, 87 minimal-feature tests, two tests per Rust
example, seven reference-contract tests, two layer parser/layout tests, two
attention operator/FP64 tests and five TS tests pass. MLX-specific attention tests
also pass in the external stock environment with no weights. Complete compiler,
Clippy, format, TS7.1/tsrs, integer Metal/resource/CLI/capture and scalar sanitizer
gates pass. No new native ABI is introduced; upstream MLX is not sanitizer-proved.
The actual release model smoke returns READY then EOS [44061,248046].
Self-review is documented; no independent review or production promotion claimed.

# Reproduction and raw retention

Use the full commands in docs/NATIVE_LAYER_REFERENCE.md with new output directories
and the authorized stock executable. Local raw paths are:

- /tmp/rust-server-layer-reference-20261022: native per-chunk traces/state/logits.
- /tmp/rust-server-stock-layer-reference-20261022: untouched stock state/logits,
  observed stock traces.
- /tmp/rust-server-stock-layer-control-20261022: SDPA-only graph control scratch.
- /tmp/rust-server-layer-suite-native-20261022: fresh complete frozen-suite vectors.

Raw arrays are scratch and may expire; Git retains digests, aggregate metrics,
source/package provenance and compact output records, not lossless arrays.
The stock regex warning is retained; prompt tokens agree on tested cases. Controls
cover this host, library, checkpoint and geometry; no all-input/all-hardware,
integer-only model, bitwise cross-chip or production-throughput claim follows.
No weights or fixture expectations changed. Rollback is the parent revision;
there is no runtime numerical profile migration in this checkpoint.

# Changelog

1.0.0 — 2026-10-22 — Codex — Layer/state localization, same-input FP64 evidence,
SDPA-only control and whole-suite confirmation; next device-state design recorded.
