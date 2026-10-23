---
title: "First-divergence native and stock layer/state experiment"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T23:59:13Z"
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
    lines: "1-49"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-62"
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
    lines: "1-220"
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
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-607"
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
    lines: "1-678"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-340"
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
related_documents: ["NATIVE_SUITE_REFERENCE.md", "ADVERSARIAL_REVIEW.md", "NEXT_STEPS.md"]
tags: [native, fidelity, reference]
---
# Executive summary

Locate the first measurable divergence on the unchanged s10 and r11 prompts.
Matching generated answers does not explain the observed 6.54% logit NRMSE.
This experiment records intermediate values before making a correction.

# Problem and design before implementation

Use the exact Combined indexed main shards, F32 floating parameters, fresh state,
eight-token prefill chunks and unchanged boundary tokenization. No expected
answer participates in forward execution. Native's existing trace exports five
stages per layer: input norm, mixer, mixer residual, MoE and final output. Export
these plus per-chunk full unified state and logits through an opt-in offline
command. Do not expose mutable state or add a production environment switch.

On stock, use in-memory observation wrappers that delegate to original modules;
never edit installed source. Capture the same stages, plus original cache state,
using stock's own graph. Check observed versus untouched stock logits and state
from fresh caches, and traced versus untraced native from the same prior state.
Any material observer perturbation invalidates localization. Use descriptive
finite/bounded metrics and identify both first nonzero drift and first NRMSE
above 1e-4 (localization marker, not a quality acceptance threshold).

Compare per-chunk and per-layer states with explicit layout: linear history
[1,3,8192], recurrent [1,32,128,128], attention K/V [1,2,position,256]. Keep full
vectors in new scratch directories; retain hashes, metrics and compact findings.
Reject incomplete inventories, wrong provenance, unexpected geometry and NaNs.
Use the existing stock source/RECORD and view guards. Keep stock baseline
unchanged; any targeted operator substitution is a separately labeled causal
control, not the accepted reference.

# Correction and qualification gate

After localization, reproduce the suspected operator on identical inputs and
compare mathematically independent controls where feasible. A correction must
have a failing negative control, meaningful regression coverage, complete frozen
suite A/B, actual model smoke and existing full compiler/resource/adversarial
checks. Never change the original quality expectations to obtain a pass.
If the observed difference is an intentional stock numerical approximation,
record that evidence and avoid silently making the native reference less accurate.

# Risks and next steps

Observers may change graph fusion or kernel dispatch; controls are mandatory.
Shared MLX primitives limit independence. Stock cache capacity must be sliced to
logical position, not compared as unused padding. No performance or production
claim follows from an instrumented trace. Independent review remains outstanding.

# Results

Pending measurement.

# Preregistered causal attention control

The initial observed trace locates the first >1e-4 NRMSE at layer 3's full
attention mixer on the first chunk of both prompts. Earlier layers and layer-3
K/V differ only around 1e-7. Both observers reproduce their untouched controls
exactly. Before changing server arithmetic, compare stock SDPA and native-style
explicit FP32 reduction on the same native layer-3 normalized input, projected
Q/K/V, gate and output weights. Use independent CPU NumPy FP64 causal GQA as the
operator oracle. Record K/V agreement with native state as a composition control.

Separately substitute only stock's SDPA call in memory with the explicit FP32
causal operator. Rerun both full graphs from fresh state with the same observations
and untouched-versus-observed controls. Label this a causal-control graph, never
stock baseline or runtime promotion. Reject unsupported mask/sink/layout/precision
instead of silently approximating. If it closes full-graph drift and FP64 favors
the existing native operator, retain native arithmetic and document the differing
numerical profile. No post-hoc acceptance threshold or quality rewrite is allowed.

# Whole-suite causal-control extension

The two-prompt SDPA-only control reduces final-logit drift to around 1e-6.
Extend that explicitly labeled control to all 60 frozen executions, with a fresh
native capture built from current source. Keep the ordinary stock runner's default
unchanged and expose the control only as an explicit offline CLI choice. Compare
exact trajectories and full first-answer vectors; report metrics without creating
a numerical production acceptance threshold after seeing the data. No native
arithmetic replacement is proposed: on identical operands it matches the explicit
FP32 operator and is closer to the independent FP64 oracle than stock SDPA.

# Measured outcome and engineering decision

The first *nonzero* differences are ordinary small floating reductions in layer
0's recurrent mixer. The first stage above the predeclared 1e-4 localization
marker is layer 3's full-attention mixer on the first chunk of both prompts:
NRMSE 0.000464039 (s10) and 0.000454299 (r11). Earlier layer outputs and layer-3
K/V are approximately 1e-7 relative error. All native trace controls and stock
untouched/observed controls are bitwise equal in this experiment. The marker
locates drift; it is not a model-quality acceptance threshold.

On identical native layer-3 input, stock's projected K/V match native state
bitwise. Recomposition of original stock SDPA/gate/output projection reproduces
the stock module bitwise. Replacing only the attention operator with explicit
FP32 reduction reproduces native mixer output bitwise. Independent CPU FP64 GQA
favors that explicit path: attention-output NRMSE is 1.53e-7 for native-style
FP32, versus 9.00e-4 / 9.24e-4 for stock fast SDPA. This is observed numerical
behavior on this device/library/geometry, not a claim that every SDPA path uses
a particular lower internal dtype or that all stock attention is inaccurate.

| Full graph | Stock SDPA vs native | SDPA-only FP32 control vs native |
| --- | --- | --- |
| s10 final first-answer logits | 0.065367653 NRMSE | 8.60248357e-7 |
| r11 final first-answer logits | 0.060410334 NRMSE | 6.60977616e-7 |

Under the SDPA-only control all measured layer/state NRMSEs remain <=1.55e-6 on
these two prompts. The controlled graph is explicitly labeled; the accepted
unchanged stock baseline and its larger differences are preserved.

The whole-suite extension matches 60/60 exact generated trajectories over 44
unique prompts. Across all 14,899,200 first-answer logit entries, maximum per-case
NRMSE is 8.75550157e-7, median 6.18719145e-7 and maximum absolute difference
2.09808350e-5. Quality remains 15/16 raw, 20/20 explicit-contract and 23/24 domain
in both lanes. All 60 freshly rebuilt native trajectories and first vectors are
bitwise unchanged from the previous qualification checkpoint.

**Decision:** retain existing native explicit FP32 attention. Do not replace it
with stock fused SDPA to erase a descriptive drift metric. A future fast profile
must be explicitly named and measured for quality, latency, memory and numerical
error; F32 input/storage alone is not a sufficient numerical-profile identifier.
No server arithmetic, weights, fixtures or quality expectations changed here.
The remaining semantic/orthographic misses and production limitations remain.

# Reproduction

Run with the same Combined artifact and strict index-only symlink view described
in NATIVE_SUITE_REFERENCE.md. All output/scratch directories must be new:

```sh
cargo build --release --locked
target/release/rust-server native-layer-reference /tmp/NEW_NATIVE_LAYERS
# Use the authorized stock environment's Python for the following commands.
python judges/layer_reference.py --scratch /tmp/NEW_NATIVE_LAYERS \
  --stock-scratch /tmp/NEW_STOCK_BASE --view /tmp/rust-server-combined-indexed-stock-view \
  --out evidence/NEW_LAYER_BASE
python judges/attention_identical_inputs.py --scratch /tmp/NEW_NATIVE_LAYERS \
  --view /tmp/rust-server-combined-indexed-stock-view --out evidence/NEW_SAME_OPERANDS
python judges/layer_reference.py --scratch /tmp/NEW_NATIVE_LAYERS \
  --stock-scratch /tmp/NEW_STOCK_CONTROL --view /tmp/rust-server-combined-indexed-stock-view \
  --attention-profile explicit-fp32-control --out evidence/NEW_LAYER_CONTROL
cargo run --release --locked --example native_suite_reference -- /tmp/NEW_NATIVE_SUITE
python judges/suite_reference.py --scratch /tmp/NEW_NATIVE_SUITE \
  --view /tmp/rust-server-combined-indexed-stock-view \
  --attention-profile explicit-fp32-control --out evidence/NEW_SUITE_CONTROL
```

The actual stock executable is
`/Users/stewartbarteau/.local/share/uv/tools/rapid-mlx/bin/python`. It imports
unchanged mlx-lm package sources verified against RECORD and no Rapid/DSpark/
DFlash modules. The `explicit-fp32-control` replaces one function binding only
inside the external process; it never edits installed files or server defaults.
Strict runtime checks reject unsupported masks, sinks, quantized caches, batch,
shape, non-F32 operands and scale. The FP64 oracle is NumPy CPU arithmetic on
identical F32 operands, with bottom-right cached causal visibility and GQA.

# Evidence and limitations

`evidence/native-layer-reference/` retains native records, ordinary stock layer/
state metrics, same-operand FP64 comparisons, the SDPA-only controlled traces,
whole-suite output/vector summaries, source/package bindings and regression/smoke
logs. Raw native traces/state are in `/tmp/rust-server-layer-reference-20261022`;
stock observation scratch and whole-suite native scratch are named in the
experiment ledger. Raw files are reproducible scratch, not committed artifacts;
retained hashes/metrics cannot reconstruct them. No throughput comparison is made
from these instrumented runs. This is strong evidence for the tested numeric
profile and trajectories, not all-input/all-hardware equivalence, integer-only
model inference, bitwise cross-chip determinism or production readiness.

# Next implementation contract

[Completion-owned device state](DEVICE_STATE_OWNERSHIP.md) specifies immutable
versioned handles, worker-only native lifetime, completion-safe publication,
nonblocking reclamation, resident-state budgets and cancellation/fork/drain gates.
It is a design for the next implementation, not an already wired capability.
The snapshot reference remains authoritative for that A/B; tensor batching follows.

# Changelog

1.0.0 — 2026-10-22 — Codex — First-divergence capture, identical-input FP64
oracle, SDPA-only causal control and whole-suite numerical-profile confirmation.
Draft self-review; independent review remains outstanding.
