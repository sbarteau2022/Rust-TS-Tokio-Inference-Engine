---
title: "Combined native C++ throughput and DSpark handoff"
document_type: implementation
version: "1.0.0"
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
related_documents: [docs/CPP_BRANCH_PROOF.md, docs/NATIVE_DSPARK2.md, docs/DSPARK_TUNING.md]
tags: [handoff, cpp, metal, dspark, throughput]
---

# Read first

Continue in this attached worktree:
`/Users/stewartbarteau/.codex/worktrees/decode-arch-kernel-proof/Rust-TS-Tokio-Inference-Engine`.
Base is user-selected `decode-arch-v1`, HEAD328a6bb, detached when created.
Resume branch: `codex/cpp-dspark-kernel-proof`. Source, scripts, docs and evidence
are included together in the publishing checkpoint. Verify origin tracking before
resuming; do not replace this work with the separate original-checkout branch.
The original checkout at `/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine`
has separate dirty descendant work; preserve it and do not mix its measurements.

User requirements: native C++ path, preserve output congruence/learned acceptance,
MTP+SPEC+TurboQuant >=125 tok/s, cached Koopah drafter with Qwen3.6-Combined.
User explicitly states that drafter is tuned for Combined. No agents without
explicit approval. No training/fine-tuning was performed. Last requested pass
adds real inference tuning and this durable handoff, not a production promotion.

# Exact runtime/artifacts

Target: `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`.
Identity:509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b.
Native target is GPU/FP32/full finite checks, group40, fused recurrent,
reference attention/experts, context512, 4-bit/group64 plus checked overrides.
Do not substitute OptiQ/base models.

Drafter directory:
`/Users/stewartbarteau/.cache/huggingface/hub/models--Koopah--Qwen3.6-35B-A3B-NVFP4-DSPARK/snapshots/815f706fe138402c9e446b2e2787da23ff7ba9e7`.
model.safetensors SHA256:ab0beb963036e558c7378b55e7188c800455ef3a493f3d9edd3f6b9a0ab4e0e2.
Runtime work is called DSpark2; the explicitly selected cached repository name
omits v2. Do not silently download or substitute another checkpoint.

SDK: original checkout `vendor/mlx`, native0.32.4. Set MLX_ROOT explicitly;
this managed worktree does not contain that vendor directory. Python reference
oracle uses `/Users/stewartbarteau/.local/share/uv/tools/mlx-dspark/bin/python`.
Python is TEST ONLY; no Python model worker or serving JSON transport exists.

# Established results before the final tuning sweep

- Target-only C++27.03 tok/s; MTP56.29 on unchanged base,166/172 learned acceptance.
- Opt-in FP32 Metal attention plus allocation correction reaches roughly59 tok/s.
  MTP width8/margin2.5 repeated59.58 with172/178 acceptance; fixture-only, not promoted.
- These are decode rates, not HTTP end-to-end. The125 combined gate remains unmet.
- Native DSpark core is implemented, not a stub: six-layer backbone, trained
  Markov/confidence, full128 NeoX RoPE, noncausal proposal attention, transactional
  context KV and exact eight Combined target taps.
- FP32 component gate:60 comparisons, max NRMSE5.31e-8/maxabs5.96e-8.
  BF16 component gate:60 comparisons BIT-EXACT; widths2/4/8, context512,
  exact greedy IDs, immutable parent KV,8 invalid descriptors. Target stays FP32.
- Target integration128/128 frozen outputs exact; independent tap-trace, feature
  charging, foreign/dead IDs, cross-branch and exhausted-slot rollback gates pass.
- Fixed width8 trained BF16 + lazy device-side Markov:54.65 diagnostic tok/s,
 88/164 learned acceptance, BELOW the retained166/172 floor. Do not promote.
- FP32 confidence0.98 admitted51/489 generated, accepted51/51, only27.90 diagnostic
  tok/s: conditional100% acceptance hides low coverage; also rejected.
- Both bound Combined head and checkpoint-own head were tested; neither fixed8
  configuration meets the acceptance floor. See raw head/precision controls.

Final tuning sweep outcome is appended below after all configurations finish.

# Source map and invariants

`native/dspark.h`: exact75-tensor BF16 artifact contract; FP32/BF16 draft math;
Markov chain stays on-device, every score row finite-checked before export.
`native/qwen_dspark.inc`: bounded context table, target-bound head/precision modes,
exact target lineage and stale-handle guards. Mode0=Combined FP32,1=own FP32,
2=Combined BF16,3=own BF16. Target must GPU/FP32/full-validation/context512.
`native/qwen_model.inc`/`qwen_device.inc`: optional post-layer tap capture;
reserve/charge before forward; tap order [1,6,11,16,22,27,32,37], unnormalized,
concatenated channels. Default target step ABI and HTTP selectors stay unchanged.
`native/dspark_tuning.h`: bounded driver policy/parser. These are harness controls,
not a deployed adaptive HTTP controller.
`judges/native_dspark_bench.cpp`: native end-to-end greedy proposal/verification/
replay driver. Records all generated/admitted/accepted counts and raw output IDs.
`judges/native_dspark_report.py`: independent frozen-ID/counter/scope/rate checks.

Proposal KV is EPHEMERAL; only committed target taps enter persistent draft KV.
Context handles bind exact target branch descriptors, not just positions. Keep
parents alive until target and drafter replacements succeed. Destroy draft before
target before weights. Width includes anchor: verifier cap8 means7 proposals;
raw checkpoint block8 means8 proposals/verifier9 and cannot silently pass this ABI.

# Reproduction

```sh
export MLX_ROOT=/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/vendor/mlx
scripts/build_native_dspark.sh /tmp/dspark-native integration
# Set DS_ARTIFACT to the pinned cached directory above.
/tmp/dspark-native evidence/decode-arch-kernel-proof/cpp-plan.txt "$DS_ARTIFACT/model.safetensors" /tmp/fresh-report.json bf16 --width=4 --confidence=0.9 --adaptive
python3 judges/native_dspark_report.py /tmp/fresh-report.json
```

Flags: width2/4/8; confidence none/0.8/0.9/0.95/0.98; --adaptive; positional
bf16/checkpoint-head/checkpoint-bf16 select explicit precision/head. Duplicate or
unknown flags fail before model loading. Adaptive starts min(4,ceiling), grows
on a fully accepted admitted block, shrinks after rejection/zero admission.
Every report path must be fresh (exclusive publication). More oracle, ABI and
Rust test commands are in docs/NATIVE_DSPARK2.md and the two build scripts.

# Next work, in order

1. Read final tuning results and retain ALL rejected controls. Choose no policy
   solely on the highest observed rate or filtered acceptance. Four reused
   prompts are discovery only; they are not held-out qualification.
2. Freeze fresh target-only outputs for new prompts BEFORE tuning/selection;
   extend the independent reader to consume explicit held-out prompt metadata.
   Current reader and native plan are deliberately four-prompt/32-output controls.
3. Qualify adaptive confidence and cost admission. If DSpark is not cost-effective,
   dispatch MTP before paying a full DSpark call, rather than invoking both on
   every fallback. Account raw generations, admitted coverage and useful yield.
4. Run matched warmed MTP/DSpark controls and committed-state gates, then 1/3-lane
   and overload/drain HTTP qualification. There is no native DSpark HTTP selector
   yet; wire it only after the quality/ownership gates hold.
5. TurboQuant KV and the125 tok/s MTP+SPEC+TurboQuant gate are NOT implemented or
   qualified by this work. Do not market a kernel-only or prototype rate as it.

# Delivery constraints and documentation

Checkpoint publication was explicitly requested on2026-10-25. No merge,
deployment, model training or model-weight edits were performed.
Do not remove the managed worktree; it contains the active implementation.
User wants concise responses (<3000 characters), test output at end of coding
runs, design before code, README updates, edge/adversarial/comprehensive tests.
Documentation standard: `/Users/stewartbarteau/Desktop/Special Ops/DOCUMENTATION_STANDARDS.md`.
All docs remain draft; no independent review approval claimed.

# Final inference-tuning pass — completed

No training or checkpoint updates. Nine configurations completed serially; each
passed128 frozen output tokens and active lineage/budget/zero-drain pressure gates.

| Policy | tok/s diagnostic | Accepted/admitted | Generated | Coverage | Floor |
|---|---:|---:|---:|---:|---|
| w2-none | 31.96 | 38/64 | 64 | 100.0% | FAIL |
| w4-none | 54.63 | 80/103 | 103 | 100.0% | FAIL |
| w8-none | 51.30 | 88/164 | 164 | 100.0% | FAIL |
| w4-c90 | 37.48 | 59/59 | 194 | 30.4% | pass |
| w4-c95 | 36.24 | 57/58 | 199 | 29.1% | pass |
| w8-c90 | 36.61 | 64/64 | 410 | 15.6% | pass |
| w8-c95 | 32.77 | 59/59 | 440 | 13.4% | pass |
| adaptive8-c90 | 39.53 | 67/72 | 180 | 40.0% | FAIL |
| adaptive8-c95 | 37.50 | 61/68 | 185 | 36.8% | FAIL |

Fastest floor-passing discovery setting: BF16 + Combined head, fixed width4,
confidence0.9:59/59 admitted accepted,194 generated (30.4% coverage),37.48 tok/s.
This is a useful acceptance control, NOT a throughput promotion: it is below the
roughly59 tok/s MTP decode control, and the scopes are not matched warmed runs.
There is no configuration here qualifying125 tok/s or held-out production.
Adaptive8 at0.9/0.95 misses the acceptance floor; do not inherit fixed4 confidence
calibration across changing widths. Native proposal forward accounts for only
roughly8–18% of these diagnostic envelopes; repeated target verification/replay
and ingestion remain material. No GPU timestamp or isolated kernel claim.

Evidence: `evidence/dspark-tuning/summary.json`, `decision.json`, raw per-config
JSON/logs, `commands.json` (exact argv), `identity.json` and final test logs.
The checked serial runner recreates the sweep in a NEW directory:

```sh
python3 scripts/run_dspark_tuning.py /tmp/dspark-native evidence/decode-arch-kernel-proof/cpp-plan.txt "$DS_ARTIFACT/model.safetensors" /tmp/fresh-dspark-sweep
```

Fresh test results for this pass: sanitized tuning transition/parser judge17
invalid cases plus valid boundary transitions;60 offline tests (including
malformed CLI/evidence-preservation and policy/timing corruption); Rust release
207/default and207/debug-validation; strict Clippy and existing unchanged native
ABI ASan/UBSan120049 checks. The reused ABI binary covers the unchanged native
backend; the new harness policy has its own freshly built sanitized judge.
The previously qualified FP32/BF16 component gates remain in native-dspark2.
No servers were started by this pass; serial sweep and regression subprocesses
have finished.

No deployment or default promotion. User's requested handoff is this worktree-root
file. Resume the pushed `codex/cpp-dspark-kernel-proof` branch; the publishing
commit contains the implementation and complete evidence.


This is an engineering handoff from local executable evidence, not a literature
claim. The component specification and reference provenance remain in
docs/NATIVE_DSPARK2.md.
