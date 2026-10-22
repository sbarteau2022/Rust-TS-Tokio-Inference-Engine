---
title: "Frozen-suite reference experiment ledger"
document_type: "research"
version: "1.0.0"
date: "2026-10-22T23:29:35Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: "examples/native_suite_reference.rs"
    lines: "1-198"
    role: "primary"
  - path: "judges/suite_reference.py"
    lines: "1-243"
    role: "primary"
  - path: "judges/test_suite_reference.py"
    lines: "1-107"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-213"
    role: "primary"
  - path: "judges/run.py"
    lines: "1-209"
    role: "primary"
  - path: "Cargo.toml"
    lines: "1-37"
    role: "primary"
related_issues: []
related_documents: ["../../README.md", "../../docs/NATIVE_SUITE_REFERENCE.md", "../../docs/ADVERSARIAL_REVIEW.md"]
tags: [qualification, provenance, reference]
---
# Executive summary

Native and stock F32 produce identical token trajectories on all 60 frozen
executions (44 unique prompts). The new comparison found and corrected native's
extra EOS terminator; both original quality misses remain.

# Design and experiment

The design preceded implementation in docs/NATIVE_SUITE_REFERENCE.md. Use its
reproduction commands and exact indexed-main-shard view. The native executable
writes bounded offline JSONL diagnostics and first-answer F32 vectors to a new
scratch directory. This is not the server's transport. Python is an explicitly
authorized external stock reference, not a worker in the native runtime.
The stock package is mlx-lm 0.31.3 / MLX 0.32.4; RECORD-verified sources, artifact
stamps, executable and forbidden-import checks are in stock/provenance.json.
Tested source is bound in source-sha256.json. Parent revision is bf2e90aa0db39d5ff0f4a53ff10407dfb195780f.

# Results and interpretation

Raw: 16/16 exact backend agreement; both quality 15/16 (No. versus no).
Contract: 20/20 exact agreement and quality. Domain: 24/24 exact agreement;
both quality 23/24 (cancellation no versus yes). See stock/summary.json,
coverage.json and full records. Never count reused raw/contract prompts as
independent quality trials. No fixture prompt, expected answer or declared
choice language was rewritten.

First-answer vectors compare all 14,899,200 entries, not just candidates.
NRMSE median 0.008851783, maximum 0.065367653 (s10). Centered NRMSE peaks at
0.065593906 (r11); max absolute difference 0.753627896 (s10). Argmax agrees
60/60, but numerical graph equivalence is not established. No causal assignment
or post-hoc numerical pass tolerance is made. Layerwise localization on s10/r11
is next, ahead of wider fidelity claims.

# EOS negative control

The original native language accepted [248046,248044]; stock accepted [248046].
The first run rejected this before generation; logs and pre-eos-native.jsonl are
retained. The corrected native boundary reads the declared top-level policy,
with bounded strict parsing and explicit generation overrides. Nested model
metadata does not silently override it. eos-control.json proves all 60 observed
generated outputs and first vectors remain unchanged. eos-source.patch records
the change. Existing historical contract fingerprints describe the old policy.

# Validation and adversarial review

117 default and 117 debug Rust tests, 87 minimal-feature tests, two tests per
Rust diagnostic example, seven reference unit tests and five TS tests pass.
Clippy, formatting, TypeScript 7.1/tsrs, Metal integer kernels, ASan/UBSan scalar
judges, resource/capture/CLI gates and the final actual Combined smoke pass.
Smoke returns READY with [44061,248046] and EOS; load/generation timings are a
single diagnostic observation, not throughput qualification. No new ABI was
introduced; this does not instrument upstream MLX with sanitizers.
Self-review is in docs/ADVERSARIAL_REVIEW.md; no independent review is claimed.

# Evidence retention and limitations

Accepted full vectors remain at /tmp/rust-server-suite-reference-eos-20261022;
pre-fix vectors remain at /tmp/rust-server-suite-reference-20261022. They are not
committed and may expire. Retained SHA256 digests and aggregates are not lossless
replacements; use the commands to regenerate vectors. native-binary.sha256 binds
the accepted capture executable; pre-eos-native-binary.sha256 binds the rejected
boundary's executable. Source and package provenance allow reconstruction.

No quality promotion, long-context qualification, actual tensor batching,
physical MLX command-buffer cap, TurboQuant attention, MTP, AMX/ANE coverage or
production sampling decision follows from this experiment. Native state and
scheduler work remain separately gated. No rollback of weights is needed;
restore the parent boundary only to reproduce the rejected control.

# Changelog

1.0.0 — 2026-10-22 — Codex — Whole-suite comparison and checkpoint EOS repair.
