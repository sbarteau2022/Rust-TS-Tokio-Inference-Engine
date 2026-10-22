---
title: "Native checkpoint ingestion and composed operator experiments"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-22"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files: []
related_issues: []
related_documents: ["../../docs/NATIVE_QWEN_GENERATION.md", "../../docs/ADVERSARIAL_REVIEW.md"]
tags: [mlx, qualification, provenance]
---
# Native ingestion/operator qualification — 2026-10-22

Selected checkpoint: `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`.
Identity: `509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b`.
Native Rust-owned MLX with C++ ABI, F32 activation profile, regular KV. No Python
inference worker, alternate model, fixture rewrite or production sampling claim.

## Historical experiments retained within this phase

`ingestion.log`, `ingestion-final.log`, `gpu-reference.log`, `model-smoke.log`,
`extended-qualification.log` and `binary-sha256.txt` predate the embedding fix.
`full-reference.log.gz` / `operator-counts.json` are the intermediate full
projection run before the additional embedding/routing/gating/residual checks.
They are not evidence for the complete corrected source. Original exit-status
files describe these historical runs. `embedding-failure.log` preserves the
first composed-check failure: embedding NRMSE 0.00132344712. The later
`embedding-negative-control.log` reverts just the casts in a BF16 fixture and
fails the same gate (NRMSE 0.00182235381). The casts were restored before the
final tests; no negative-control mutation is shipped.

## Corrected complete composed check

`corrected-full-reference.log.gz`, `corrected-operator-counts.json`, and
`corrected-full-exit-status.txt`: `native-reference --full`. All 31 frozen r10
prompt tokens, ordinary projections, active token/expert projections and every
LM-head output are checked. Scalar FP64 independently unpacks quantization and
computes projections/embedding values; other scalar routines compute operator
outputs from native input operands. All native outputs remain observations;
reference values never replace logits, state or choices.

133,880 projection tiles, 2,768 operator checks and 40 attention checks pass their
fixed 1e-4 / 1e-5 / 1e-5 NRMSE thresholds. Checked/unchecked control is zero;
chunk/single control 6.07e-7. **Command exits 1 because the unchanged semantic
case still fails**, favoring no by 0.107872009 logits. This is not an independently
composed full model graph, nor complete route/context coverage. The logs are
compressed losslessly with deterministic gzip metadata; summary counts are
computed directly from those retained logs.

## Test boundaries

The scalar harness includes 93 projection/embedding cases across 2/3/4/5/6/8-bit
packing and groups 32/64/128, and 43 analytic/malformed operator cases. These
include partial RoPE, shared-key-head recurrence/history, top-k ties, invalid
membership and overflowing RoPE positions. Real checkpoint runs observe 4/8-bit,
group 64; other packing modes remain synthetic coverage. FFI ASan/UBSan harness:
14,000 malformed descriptor cases, output buffers unchanged. Each projection
tile uses no more than eight joined scalar workers and 256 channels. Weight
exports are bounded to 4 MiB and audited against separately read shard payloads.

Final-source model runs and source/binary bindings are added below after the
ordered regression run. Root `source-sha256.json` binds application/test source;
this directory's manifest binds the retained experiment artifacts. No independent
reviewer, production readiness or physical MLX in-flight cap is claimed.


## Final-source results

`final-full-reference.log.gz` and `final-full-summary.json` repeat complete
composed coverage against the final executable and exactly reproduce the counts,
maxima, controls and failed semantic margin above. The final CPU diagnostic also
favors no by 0.107872009 logits with zero checked/unchecked error; no backend
quality fix is claimed. `final-ingestion.log` repeats the complete byte audit.
`final-model-runs.json` records commands, statuses and elapsed times separately:
READY smoke and contracted 20/20 pass; raw 15/16 and extended 23/24 return 1.
Context-512: logit NRMSE 3.7e-7, state NRMSE 2.8e-7, overflow/isolation/drain pass.
The original failures remain unchanged. `final-cli-rejections.json` records nine
invalid CLI forms rejected before opening the model. Final Rust regression is
116 default / 116 debug / 87 minimal, with five TS tests and the existing compiler,
Metal and sanitizer gates passing. Final source, executable, MLX library/metallib,
version header and fixtures have explicit digest bindings. Full model remains
unqualified; the next requested discriminators use stock mlx-lm as a separately
labeled reference experiment, without adding Python inference to the server.
