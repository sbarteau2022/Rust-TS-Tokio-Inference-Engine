---
title: "DSpark inference tuning — final session pass"
document_type: feature
version: "0.1.0"
date: "2026-10-25T04:13:29Z"
implementation_date: null
author: Codex
reviewers: []
status: draft
related_files:
  - path: "README.md"
    lines: "1-1518"
    role: supporting
  - path: "handoff.md"
    lines: "1-322"
    role: primary
  - path: "docs/DSPARK_TUNING.md"
    lines: "1-204"
    role: primary
  - path: "native/dspark_tuning.h"
    lines: "1-35"
    role: primary
  - path: "judges/native_dspark_bench.cpp"
    lines: "1-75"
    role: test
  - path: "judges/native_dspark_report.py"
    lines: "1-57"
    role: test
  - path: "judges/dspark_tuning.cpp"
    lines: "1-13"
    role: test
  - path: "judges/test_dspark_tuning_cli.py"
    lines: "1-51"
    role: test
  - path: "judges/test_dspark_tuned_report.py"
    lines: "1-31"
    role: test
  - path: "scripts/run_dspark_tuning.py"
    lines: "1-49"
    role: config
  - path: "evidence/dspark-tuning/adaptive8-c90.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/adaptive8-c90.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/adaptive8-c95.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/adaptive8-c95.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/build-final.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/dspark-tuning/build.log"
    lines: "N/A (empty artifact)"
    role: supporting
  - path: "evidence/dspark-tuning/clippy.log"
    lines: "1-32"
    role: supporting
  - path: "evidence/dspark-tuning/commands.json"
    lines: "1-85"
    role: supporting
  - path: "evidence/dspark-tuning/decision.json"
    lines: "1-33"
    role: supporting
  - path: "evidence/dspark-tuning/git-state.txt"
    lines: "1-35"
    role: supporting
  - path: "evidence/dspark-tuning/identity.json"
    lines: "1-15"
    role: supporting
  - path: "evidence/dspark-tuning/native-abi.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/offline-final.log"
    lines: "1-16"
    role: supporting
  - path: "evidence/dspark-tuning/offline.log"
    lines: "1-16"
    role: supporting
  - path: "evidence/dspark-tuning/rust-debug.log"
    lines: "1-865"
    role: supporting
  - path: "evidence/dspark-tuning/rust-default.log"
    lines: "1-865"
    role: supporting
  - path: "evidence/dspark-tuning/summary.json"
    lines: "1-155"
    role: supporting
  - path: "evidence/dspark-tuning/unit-final.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/unit.log"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w2-none.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w2-none.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w4-c90.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w4-c90.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w4-c95.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w4-c95.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w4-none.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w4-none.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w8-c90.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w8-c90.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w8-c95.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w8-c95.log"
    lines: "1-10"
    role: supporting
  - path: "evidence/dspark-tuning/w8-none.json"
    lines: "1-1"
    role: supporting
  - path: "evidence/dspark-tuning/w8-none.log"
    lines: "1-10"
    role: supporting
related_issues: []
related_documents: [NATIVE_DSPARK2.md, ../handoff.md]
tags: [cpp, dspark, inference-tuning]
---

# Preregistered inference tuning

Problem: native math qualification passes, but fixed-width8 proposer acceptance
misses the retained96.51% learned acceptance floor. No fine-tuning or training is
authorized or performed by this pass. Tune inference controls with all raw
proposal/coverage/fallback counts visible. Keep target-authoritative output,
FP32 target/full finite checks, trained BF16 draft and Combined head fixed.

Add strict runtime flags --width=2|4|8, --confidence=none|0.8|0.9|0.95|0.98 and
--adaptive. Preserve existing named head/precision positional controls. Reject
unknown/duplicate flags and malformed values before loading any model.
Adaptive mode starts at min(4, ceiling), doubles after a fully accepted admitted
block and halves (floor2) after rejection or confidence-driven single-token
fallback. Compare fixed ceilings and thresholds; report policy and useful output
without hiding generated proposals discarded by confidence admission.

Run a bounded sweep: fixed widths2/4/8 with no confidence cutoff, fixed4 and8
at0.9/0.95, adaptive8 with0.9/0.95. Existing unfiltered8 final run is a historical
control, not a fresh matched speed comparison. Require128 exact frozen outputs,
zero drain, >=166/172 accepted/admitted ratio and then compare useful throughput
against the current MTP control. No production selection from these four reused
fixtures; held-out calibration and matched warmed HTTP remain required.

Add independent policy validation and malformed-flag/transition edge tests.
Pressure gates in the existing integration executable remain active for every
configuration. Record all outcomes and explicit failures in evidence/dspark-tuning.
C++ is used for the native driver; Python is offline independent accounting only.
This is an engineering experiment, not a literature claim. Nothing deploys or
changes HTTP defaults. Update the worktree-root handoff.md before ending.

## Outcome, review and rollout

All nine discovery controls finish and independently validate128 frozen output
tokens each (1152 total) with zero drain. Fixed4/confidence0.9 accepts59/59 admitted
out of194 generated (30.4% coverage),37.48 diagnostic tok/s. Four configurations
pass the retained acceptance floor; none establishes the combined throughput
gate. Adaptive8 at0.9/0.95 fails acceptance. No production selection or training.
Full table and next steps are in worktree-root handoff.md; decision.json keeps
matched warmed comparison, held-out qualification and125 completion false.

Review hardened duplicate/unknown flags, finite confidence policy values, power-
of-two adaptive states, impossible acceptance counts and exhausted evidence
directories. Explicit confidence=none overrides the legacy confidence-build
default. Every sweep configuration retains target branch/budget pressure gates.
Tests: sanitized policy judge17 invalid cases and boundary transitions;60 offline
tests;207 Rust tests per profile, strict Clippy and unchanged native ABI120049
sanitizer checks pass. No native model math changed in this pass.

Reproduce with scripts/build_native_dspark.sh and scripts/run_dspark_tuning.py;
the latter refuses an existing directory, records argv and times out failed
controls. Diagnostic timing is single-pass and shape-cold, not a matched warmed
comparison against MTP. No deployment, merge or approved review occurred.
Next: freeze fresh held-out prompts, qualify per-width confidence/cost admission
and MTP dispatch, then repeat warmed comparisons and HTTP/drain gates.

2026-10-24: inference flags, adaptive-window experiment, independent policy
checks, full serial sweep and durable handoff.

These tests prevent misleading acceptance qualification, ambiguous runtime
policies and accidental evidence overwrite.
