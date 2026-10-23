---
title: "Frozen-suite native and stock reference qualification"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T23:29:35Z"
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
related_documents: ["NATIVE_DISCRIMINATORS.md", "NEXT_STEPS.md", "ADVERSARIAL_REVIEW.md"]
tags: [native, reference, qualification]
---
# Executive summary

Compare all frozen raw, explicit-contract and domain cases against unchanged
stock mlx-lm on the designated Combined checkpoint. Output agreement and answer
quality are different gates; neither implies production readiness.

# Problem statement

Agreement on two failures cannot qualify the other prompts, output contracts or
decode trajectories. An independent whole-suite comparison is missing.

# Design before implementation

The inventory is exactly 16 raw + 20 contract + 24 domain executions. The contract
suite reuses the raw 16 and constrains only rust-borrow and four choice-* rows to
[yes,no], matching the production qualification harness. Domain rows use their
explicit choice field; '-' remains unrestricted. Original fixtures never change.
Expected answers are used only after generation to report trim-plus-EOS quality.
Backend parity requires exact prompt IDs, canonical choice fingerprints, generated
IDs (including EOS), decoded bytes and finish reason. No normalization hides drift.

Both lanes use fresh state per execution, eight-token prefill chunks, greedy
lowest-token-ID ties, a 32-token output budget, thinking disabled, no speculation,
prefix reuse or quantized KV. The primary comparison is native F32 against stock
F32 floating parameters; packed integer weights remain unchanged. This is not a
KV-only precision experiment. Stock must load the verified index-only symlink
view, excluding the MTP sidecar; source files must match installed package RECORD.
The Python program is an external offline experiment, never a server dependency.

Capture complete first-answer F32 vectors in a caller-owned scratch directory;
compare all vocabulary entries, record hashes, finite checks, max absolute error,
NRMSE and centered NRMSE. Retain compact top-token and candidate scores plus
comparison aggregates in evidence; scratch vectors are reproducible, not committed.
Numerical drift is descriptive: no post-hoc tolerance can convert a token mismatch
into a pass. This covers the tested trajectories and first-answer vectors, not
all-layer/state equivalence. A mismatch blocks promotion and requires localization.

# Implementation and validation plan

Add a bounded offline Rust example using the production Choices selector and a
separate independently tested Python trie/reference runner. Test shared/terminal
prefixes, ties, invalid inputs, exact inventory and scoring/execution separation.
Reject malformed/truncated/duplicate/missing records, wrong model views and source
provenance failures. Report per-suite quality and exact backend matches separately.
Run existing regression, minimal-feature tests, example tests and the actual model
comparison before shipping. Update README, evidence manifests and file map.

# Risks and next steps

No serving defaults change. This does not establish scheduler concurrency,
physical command-buffer bounds, batched tensor execution, TurboQuant, AMX/ANE,
long-context quality or production safety. Independent review remains outstanding.
After parity, unify state ownership and exercise scheduler cancellation/isolation
before introducing tensor batching. Keep all known semantic/orthographic misses.

# Results

Measurement was pending at design time; the completed results are recorded below.

# Boundary discrepancy found before comparison

The first attempt stopped before generation: native EOS was hardcoded to both
im_end (248046) and endoftext (248044), whereas unchanged stock uses top-level
config eos_token_id=248046. The nested text_config's 248044 is not the effective
stock policy. Preserve the rejected run and original native capture. Correct the
native boundary to read bounded, fingerprint-bound configuration: use top-level
EOS, with an explicit generation_config override if present; reject malformed,
empty, oversized, unknown or out-of-range EOS values. Never silently add alternate
terminators. Add scalar/array and adversarial parser tests, then recapture all
60 native cases. This corrects termination semantics, not model answer content.

# Measured outcome

All 60 native/stock F32 executions match exact prompt IDs, canonical contract
fingerprints, generated token IDs including EOS, decoded text and finish reason.
There are 44 unique prompts: the contract suite deliberately reuses the raw suite.
This is tested-trajectory agreement, not 60 independent model-quality examples.

| Suite | Exact backend agreement | Native quality | Stock quality |
| --- | --- | --- | --- |
| Raw | 16/16 | 15/16 | 15/16 |
| Explicit contract | 20/20 | 20/20 | 20/20 |
| Rust/security domain | 24/24 | 23/24 | 23/24 |

Raw borrowing remains `No.` versus expected `no`; contracted cancellation remains
`no` versus expected `yes`. Neither expected answer was changed. The EOS fix
changes the accepted termination language from [248044,248046] to [248046]; all
60 observed generated trajectories and first-answer vectors are bitwise unchanged
by that boundary fix. Previously committed termination fingerprints are historical.

The 14,899,200 paired first-answer logit entries are **not numerically equal**.
NRMSE median is 0.008851783 and maximum 0.065367653 (domain/s10); maximum absolute
logit difference is 0.753627896, also s10. Centered NRMSE peaks at 0.065593906
(domain/r11), so a common logit offset does not explain the discrepancy. All 60
unconstrained first-token argmaxes agree. No numerical-equivalence threshold was
preregistered and none is invented after measurement. Before a wider fidelity
claim, localize the first layer/state divergence on s10 and r11 against stock.
Do not attribute the drift to one operator without that experiment.

# Reproduction and retained evidence

Run from the repository with the designated model present. Create a fresh scratch
name; the Rust example refuses existing directories. Then use the authorized
stock environment's Python; no package install or source patch is required:

```sh
cargo build --release --locked --example native_suite_reference
target/release/examples/native_suite_reference /tmp/NEW_SUITE_CAPTURE
/Users/stewartbarteau/.local/share/uv/tools/rapid-mlx/bin/python judges/suite_reference.py \
  --scratch /tmp/NEW_SUITE_CAPTURE \
  --view /tmp/rust-server-combined-indexed-stock-view \
  --out evidence/NEW_SUITE_COMPARISON
```

The index-only view inventory and exact symlink targets are recorded in
`evidence/native-discriminators/indexed-stock-view.json`; create those symlinks
if the temporary view is gone. Never include model-mtp.safetensors. The runner
verifies file stamps against the prior ingestion/discriminator provenance and
all installed stock Python sources against distribution RECORD before/after the
run. If artifacts changed, re-audit them rather than bypassing the checks.

`evidence/native-suite-reference/` retains accepted and pre-fix native captures,
the rejected EOS-contract attempt, stock results/source provenance, per-vector
hashes/drift summaries, EOS control and pressure/regression evidence. Complete
vectors remain in local scratch, not in Git. Full vectors can be regenerated;
compact reports cannot reconstruct them. Performance numbers from this diagnostic
are not production throughput measurements. No serving admission/sampling defaults,
weights, fixtures, scheduler or kernel arithmetic were changed by this phase.

# Changelog

1.0.0 — 2026-10-22 — Codex — Design, implementation, whole-suite measurements and
checkpoint EOS correction. Draft self-review; independent review outstanding.
