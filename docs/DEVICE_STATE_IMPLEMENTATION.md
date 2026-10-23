---
title: "Completion-owned MLX state — implementation and measured scheduler smoke"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-23T00:42:45Z"
implementation_date: "2026-10-22"
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
    lines: "1-94"
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
    lines: "1-49"
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
  - path: "judges/native_http_smoke.py"
    lines: "1-81"
    role: "test"
  - path: "judges/native_stream_disconnect.py"
    lines: "1-36"
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
    lines: "1-117"
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
    lines: "1-809"
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
  - path: "src/serving/native_http.rs"
    lines: "1-217"
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
    lines: "1-206"
    role: "primary"
  - path: "src/serving/native_mtp_cli.rs"
    lines: "1-344"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-436"
    role: "primary"
  - path: "src/serving/native_mtp_scheduler.rs"
    lines: "1-726"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-694"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-386"
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
    lines: "1-568"
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
    lines: "1-6"
    role: "test"
  - path: "ts/http-bench-config.ts"
    lines: "1-21"
    role: "primary"
  - path: "ts/http-bench.ts"
    lines: "1-48"
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
related_documents: ["../README.md", "DEVICE_STATE_OWNERSHIP.md", "NATIVE_LAYER_REFERENCE.md", "ADVERSARIAL_REVIEW.md", "NEXT_STEPS.md"]
tags: [mlx, rust, lifetime, scheduler, qualification]
---
# Scope and state contract

Only `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`, identity
`509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b`.
FP32 activations, explicit FP32 attention, regular dense KV; 40 hybrid layers,
context 512, chunk up to 8. Custom integer Metal kernels are unchanged. No Python
worker or JSON runtime transport is introduced; JSON files are offline evidence.

The old snapshot ABI and device session call the same computation. Device state
contains 80 evaluated immutable arrays in a worker-confined table. Each descriptor
binds native owner, slot, generation, position and full logical bytes. Reservations
charge slots and bytes before any model encoding. Publication follows completion,
finite checks and exact state inventory; an exception rolls the reservation back.
Generation exhaustion retires a slot permanently, and owner counters never wrap.
Logical charges deliberately overcount buffer sharing and are not physical GPU
allocation bounds. Process-wide MLX allocator active/peak bytes include weights,
other MLX allocations and earlier windows; unique state allocation is unavailable.

Rust `DeviceState` clones one Arc lifetime token. Its last drop sends one release
record on a separate unbounded channel, logically bounded by resident versions;
there is no full-work-queue release deadlock. Commands retain prior state through
native completion. A dropped response retires the newly completed version. Only
the worker calls FFI or destroys arrays/weights/streams. Reclamation precedes new
reservations. Computation errors latch admission closed; an explicit resident
budget rejection preserves admission so released capacity can recover.

Shutdown stops admission and drains accepted commands, then releases queued
retirements and explicitly clears the native table. Native stats must report zero
residents/bytes. Live external version tokens are counted as invalidated (unique
versions, not Arc clone count); their later drops are safe after worker shutdown.
The existing snapshot and diagnostic snapshot types share one Rust owner allocator,
preventing cross-executor epoch collisions. Native IDs independently enforce owners.

# Scheduler integration and limitations

`ScheduledDevice` is an explicit experimental adapter with separate snapshot/device
fingerprints. It checks the complete input batch before any row executes and
commits all sequence versions only after all rows succeed. It caps three resident
sequences and three logical admissions; the smoke uses eight version slots and a
768 MiB conservative resident budget, covering old/new rows at batch publication.
Rows call native execution serially. Continuous iteration scheduling interleaves
prefill/decode requests; it does not fuse a tensor batch. No HTTP/SSE token-stream
endpoint or autonomous-agent loop is measured. Default `ScheduledQwen` remains the
snapshot reference, with no automatic promotion or mid-sequence fallback.

The work channel caps three queued requests plus the single worker's active work;
step permits cap three admitted model steps. These are software limits, not a
physical MLX command-buffer cap. A separate resident table budgets persistent
state; temporary workspace is bounded by fixed geometry/queueing, not a measured
hard allocator reservation. Proposed production sampling remains inactive.

# Numerical preservation and lifetime pressure

The first device/snapshot run shares one immutable weight store and tests every
step of the 60 frozen suite executions (360 steps). Logits, five stages per layer
and unified state have **zero observed differences**, with finite/shape checks
and preregistered <=1e-5 relative L2. Original quality stays **15/16 raw, 20/20
contract and 23/24 domain**; no prompts, labels or masks are repaired. This is
implementation preservation, not a new quality qualification.

The hardware pressure run covers eight resident versions, rejection/recovery,
branch re-execution, unchanged shared prefixes, cancellation after a request is
pending native completion, three admitted jobs with a cancelled queued sibling,
fourth-step admission rejection, context 504+8 and eight single-token continuation,
overflow rejection, shutdown with one live version and foreign epoch rejection
after restart. At context 512 state is 86,835,200 bytes. Chunk/single relative L2
is 4.45e-7 logits and 1.87e-7 state in the first run. A no-export section records
zero full-state CPU uploads/exports; diagnostic A/B exports are reported separately.
Counters do not describe all Metal transfers: weights, tokens, logits, traces and
scalar finite flags have their own I/O and are excluded from full-state counters.

Pure lifetime tests cover clone/final-drop, dropped replies, full work queues and
safe drops after channel closure. The generic native table is ASan/UBSan tested
for foreign/stale/forged geometry, ABA reuse, budget exhaustion, rollback,
generation exhaustion and clearing live versions. Native ABI pressure uses valid
session metadata with a sentinel weight pointer only for cases rejected before
weight dereference; 14,000 iterations check rollback and output-ID nonpublication.
It does not instrument upstream MLX or prove GPU race freedom.

# First measured scheduler windows

Apple M5 Pro, 48 GiB unified memory. Same weights and arithmetic, warmup excluded,
six requests per window from three authored coding/security prompts, maximum 48
new tokens, greedily decoded. All 24 requests match exact trajectories. Token
publication uses a monotonic clock from Rust submission, including queueing;
it is not client-arrival timing. Tokenization/chat rendering precede submission,
so TTFT excludes them; request latency also excludes post-receipt detokenization.
Aggregate wall time includes local tokenization, detokenization and report writes.
Request completion includes configured length stops and is not semantic task
acceptance. Nearest-rank quantiles over six requests have no
confidence interval and are not a sustained production benchmark.

| State | Lanes | TTFT p50/p95 ms | Inter-token p50/p95 ms | Aggregate output tok/s | Median per-lane decode tok/s | Request p50/p95 s |
| --- | --- | --- | --- | --- | --- | --- |
| Snapshot | 1 | 326 / 407 | 54.1 / 56.9 | 16.45 | 18.43 | 2.88 / 3.02 |
| Snapshot | 3 | 1053 / 1083 | 161.8 / 206.3 | 16.18 | 5.89 | 8.62 / 9.06 |
| First device | 1 | 363 / 443 | 62.3 / 64.2 | 14.43 | 15.94 | 3.30 / 3.33 |
| First device | 3 | 1164 / 1169 | 187.3 / 191.5 | 14.41 | 5.37 | 9.92 / 9.93 |

Aggregate output includes prefill wall time and EOS; per-lane decode uses tokens
after the first divided by their elapsed publication interval and includes EOS.
Request admission/completion is 100% inside these configured windows. There is
no speculative decoding: speculative acceptance is **not applicable**, not 100%.
MLX process peak was approximately 19.69–20.08 GB (decimal) across the first run;
scheduler logical peak 623,706,112 bytes (one lane) or 797,376,512 (three lanes)
is separate bookkeeping, not that allocator peak.

The first device path is **11–12% slower** than snapshots. It synchronized each
of 80 finite-state reductions separately. Preserve this failed performance result.
The revised implementation evaluates all scalar flags with logits at the existing
final completion barrier, retaining every original layer completion boundary.
No state reaches the table before those flags pass. Performance and numerical
preservation for this optimization must be measured independently.

# Consolidated finite-check scheduler run

Moving the 80 scalar finite checks to the existing final completion barrier gives
these paired windows, with identical trajectories and clean drains again:

| State | Lanes | TTFT p50/p95 ms | Inter-token p50/p95 ms | Aggregate output tok/s | Median per-lane decode tok/s | Request p50/p95 s |
| --- | --- | --- | --- | --- | --- | --- |
| Snapshot | 1 | 324 / 403 | 54.2 / 58.2 | 16.43 | 18.42 | 2.86 / 3.01 |
| Snapshot | 3 | 1032 / 1039 | 161.2 / 163.7 | 16.68 | 6.23 | 8.57 / 8.57 |
| Consolidated device | 1 | 306 / 376 | 48.4 / 51.2 | 18.27 | 20.46 | 2.60 / 2.67 |
| Consolidated device | 3 | 978 / 980 | 145.4 / 147.4 | 18.38 | 6.91 | 7.78 / 7.78 |

Observed aggregate output improves 11.2% (one lane) and 10.2% (three lanes) against
the same run's snapshot windows. These fixed-order, unreplicated six-request
windows do not establish a general speedup or its confidence interval. Three
lanes share throughput and increase each lane's inter-token latency; they do not
triple aggregate throughput. The process reports 23,469,735,936 maximum resident
set bytes and 24,182,261,112 peak footprint bytes via macOS `/usr/bin/time -l`;
these cover loading and every window, not per-request or unique GPU state memory.
Both snapshots and consolidated device defaults/arithmetic are unchanged beyond
state transport and finite-check scheduling. Snapshot remains the serving default.

# Final validation checkpoint

The consolidated implementation repeats the full 60-execution / 360-step gate
with **matching float bit patterns** across logits, stage traces and full unified
state; maximum relative and absolute differences are zero. All historical prompt
IDs, generated tokens, text, finish reasons and quality labels match the previous
frozen capture. The strengthened three-admitted-job/cancel/fourth-rejection pressure
and context/restart/drain checks pass. Default and debug Rust suites each pass
122 tests; minimal-feature Rust passes 87. Example/fixture checks, TS 7.1/tsrs,
Clippy, formatting, custom Metal/GPU capture, independent scalar operator checks,
70,015 native ABI pressure checks and 96,024 state-table checks pass. Upstream MLX
is uninstrumented and review is self-review. Original fixtures remain unchanged.
No held-out quality threshold is waived, and snapshot remains the default.

# Reproduction and evidence

Use new directories; existing directories fail before model loading:

```sh
cargo run --release --locked --example native_device_reference -- NEW_AB_DIRECTORY
cargo run --release --locked --example native_device_pressure -- NEW_PRESSURE_DIRECTORY
cargo run --release --locked -- native-device-smoke NEW_SMOKE_DIRECTORY
python3 judges/device_smoke_report.py NEW_SMOKE_DIRECTORY
python3 judges/run.py
cargo test --locked --no-default-features
```

The Python command derives offline rates from saved Rust timestamps only; it does
not load a model, serve requests or enter the runtime. Preserve the first-run and
consolidated runs separately in `evidence/device-state/`, with source/binary and
nested evidence hashes. Full-model scratch exports are diagnostic, never on the
serving hot path. A clean smoke cannot establish production readiness, tensor
batch efficiency, drafting quality, TurboQuant correctness or AMX/ANE coverage.

# Changelog

1.0.0 — 2026-10-22 — Checked native versions, Rust completion ownership and
experimental scheduler lane; preservation, pressure and first throughput evidence.
