---
title: "Completion-owned native device state — implementation contract"
document_type: "architecture"
version: "1.1.0"
date: "2026-10-23T00:42:45Z"
implementation_date: null
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
  - path: "ts/http-bench.ts"
    lines: "1-32"
    role: "primary"
  - path: "ts/http-client.test.ts"
    lines: "1-16"
    role: "test"
  - path: "ts/http-client.ts"
    lines: "1-11"
    role: "primary"
  - path: "ts/http-metrics.test.ts"
    lines: "1-6"
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
related_documents: ["NATIVE_LAYER_REFERENCE.md", "NATIVE_QWEN_GENERATION.md", "NEXT_STEPS.md"]
tags: [device-state, ownership, cancellation, scheduler]
---
# Executive summary

Keep KV, convolution history and recurrent state in native MLX arrays between
steps. Rust owns immutable versioned references; the executor worker owns all
native handles and releases them after computation completes. This removes the
current full-state CPU export/upload cycle while preserving the snapshot path as
an explicit diagnostic reference. This document is a design contract, not an
implemented device-state backend.

# Problem statement

Current QwenExecutor reconstructs each layer's state from an Arc<Vec<f32>> and
copies every next-state array back to that vector each step. At context 512 the
joint state is 86,835,200 bytes per sequence. Logical three-command admission
bounds work, but does not bound all resident branch/prefix snapshots or physical
MLX command buffers. Removing copies without replacing ownership/accounting
would introduce use-after-free, cancellation leaks and branch contamination.

# Numerical and model contract

Target only the Combined checkpoint's existing 40-layer program, context <=512
and chunks <=8. Preserve the accurate explicit FP32 causal-attention profile;
stock fused SDPA is a separate numerical profile, not a synonym for FP32.
Keep unchanged scalar operator gates, declared output contracts and boundary
EOS. Native arithmetic and the snapshot reference remain available for A/B.
No expected answer, tokenizer replacement, codec, MTP or tensor batching enters
this change. BF16 is separately labeled and must pass its own corresponding
snapshot controls; never silently substitute it for F32.

# Ownership and publication

A worker-local session owns weights/config/stream plus a bounded table of
immutable state versions. A state consists of 80 typed arrays ordered by layer:
30 convolution histories [1,3,8192], 30 recurrent states [1,32,128,128] and ten
pairs of K/V [1,2,position,256]. F32 recurrence remains explicit. An empty state
has position zero and no native allocation.

Rust StateRef holds session owner, slot/generation ID, position, logical bytes
and an Arc lifetime token. It never contains a native pointer and does not expose
mutable arrays. Cloning/forking aliases an immutable version; writes create a new
version. No in-place reuse of a prior KV/recurrent buffer is permitted merely
because one request is active: prefix/fork readers may still own it. Generation
reuse is checked and generation exhaustion fails closed rather than wrapping.

The worker executes old-state reads and builds a private next state. Only after
MLX evaluation/synchronization succeeds and outputs pass finite/shape checks does
it install the new ID and publish logits/StateRef together. An error publishes
neither partial state nor a usable ID. Preserve explicit layer evaluation boundaries
initially; fusion/dispatch changes require a separate numerical experiment.
Cancellation drops the response owner; the completed unpublished result is then
retired, with identical cleanup to an ordinary last-reference release.

# Native ABI proposal

Use an opaque session and a checked state descriptor carrying owner epoch, slot
and generation (not a bare slot index); validate all three in native code as well
as Rust so equal slot numbers in different sessions cannot alias. Keep the existing snapshot
ABI intact. Proposals below are names/contracts, not callable functions today:

- create session: bind weights, compiled layer program, device and numerical profile.
- step(session, previous ID, expected position, tokens, output capacity): validate
owner/geometry/capacity; return a new ID, position, logits and measured allocation
accounting after completion. Roll back private allocations on every failure.
- release(session, ID): retire one table entry exactly once after all command
readers finish; stale/wrong-generation IDs fail without touching another entry.
- snapshot(session, ID, expected length): explicit opt-in CPU diagnostic export.
- stats(session): resident versions, conservative logical state charges, measured
global allocator bytes, pending completion/release counts and unpublished
retirements. Unique physical state bytes are optional and availability-labeled;
never infer them from logical array sizes when backing allocations may alias.
- destroy(session): finish submitted work, release every table entry and weights,
then destroy the stream/session on the same worker. Return drain accounting.

No ABI call occurs from a Rust arbitrary-thread Drop. Last-reference Drop posts
one nonblocking release record to a worker-owned reclamation queue. That queue is
separate from the bounded work queue so a full admission queue cannot block
cleanup. Outstanding records are bounded by the resident version cap, with one
record per lifetime token. The worker drains releases before allocating and during
shutdown; rejected/closed sends are accounted for by session-wide teardown.

# Resource admission and cancellation

Continue the three-permit work semaphore. Separately charge each resident state
lifetime, not just each Command. A fork shares a lifetime charge; a newly computed
state reserves its own budget before admission. A conservative full-logical-size
charge is safe initially even if MLX shares immutable backing storage. Report it
as conservative accounting, not measured unique GPU allocation. Charge transient
old/new arrays, graph workspace and host logits separately; a memory limit must
include all live states, queued commands and unpublished results.

Reserve bounded table capacity before encoding; release it on failure/cancelled
publication. At capacity, admission waits or returns an explicit resource error;
it does not evict a live prefix/branch. Keep command Arc readers until native
completion even if the Tokio future is dropped. Publish a scheduler row only if
its sequence/generation still matches; otherwise retire the new state. A step's
failure closes the lane under the current fail-closed behavior.

Shutdown closes admission, drains accepted work and releases, retires every
remaining device version, joins the worker and reports zero native residents and
pending completions. External StateRefs that outlive shutdown become invalid
session tokens; they cannot resurrect native storage or call an already-freed
pointer. Account for invalidated external references separately from native leaks.
This still does not impose a physical MLX command-buffer limit; that needs a
separate backend-supported completion/capability gate.

# Preregistered validation before enabling

1. Pure lifetime tests: fork/clone/drop, session mismatch, stale/ABA IDs, slot and
byte exhaustion, generation overflow, full release/work queues, send failure and
shutdown with live references. A dropped request must not release command readers.
2. Native ASan/UBSan ABI pressure: malformed capacities/IDs/positions, double release,
partial-step failure and teardown. Check resident/pending accounting after each
case; upstream MLX is not automatically sanitizer-instrumented.
3. Snapshot/device A/B on all 60 frozen executions: exact prompt/contract/token
trajectories, no quality-label changes. Preregister <=1e-5 relative L2 for logits,
all layer stages and unified state against the corresponding native snapshot
profile, with finite/shape checks. Report max absolute differences as well.
This is an implementation-preservation gate, not new model-quality qualification.
4. Context 512 continuation, one-token/chunk parity, multiple branches from one
prefix, sibling cancellation at every publication boundary, mixed request lengths,
full resident budget, queued cancellation, repeated shutdown and restart.
5. Complete current regression and actual Combined smoke. Verify removal of full
state CPU export/upload with byte counters; measure TTFT, tail decode latency,
throughput and MLX/host peak memory at fixed concurrency against the snapshot lane.
Do not infer a speedup from fewer copies alone.

# Implementation sequence and rollback

First build the checked native session/state table and lifetime/ABI tests without
changing default serving. Then wire Rust immutable StateRef ownership and worker
reclamation, add explicit diagnostic export, and run the entire A/B/pressure matrix.
Only after those pass select device state in the scheduler with a new backend
fingerprint. Keep the snapshot reference selectable for diagnosis and rollback;
never silently fall back mid-sequence. Actual tensor batching follows this gate.

# Scope limits and approval status

Regular dense KV only. Paged KV, TurboQuant, shared writable batch buffers,
longer context, drafting, AMX/ANE and production sampling remain separate work.
The opt-in implementation is documented in DEVICE_STATE_IMPLEMENTATION.md. No
independent review, broad workload qualification or production promotion is claimed.
Snapshot defaults remain in place while measured scheduler performance is assessed.

# Changelog

1.0.0 — 2026-10-22 — Codex — Explicit ownership, completion, resource and
cancellation contract with measured snapshot baseline and preregistered gates.

## Opt-in scheduler smoke measurement contract

The implementation may expose an explicitly experimental device-state scheduler
adapter while retaining snapshot defaults. It must validate the entire batch before
publication, keep six old/new versions for three sequence rows, and use a distinct
backend fingerprint. Rows remain serial native calls; this is continuous scheduling,
not tensor batching. An offline A/B smoke shares one immutable weight store and
switches only snapshot/device state. Measure concurrency 1 and 3 with identical
prompts and generated-token parity. Monotonic timestamps start at Rust submission;
record each validated token's publication time, queue-inclusive first-token latency,
inter-token p50/p95, request p50/p95, aggregate tokens/sec including EOS and separately
excluding EOS. Exclude load/warmup from measured windows. This is a small scheduler
smoke, not an HTTP endpoint or autonomous-agent workload qualification.

Finite state validation remains mandatory before publication. The first measured
implementation synchronized once per state array and regressed throughput despite
zero state copies. The revised implementation collects scalar finite reductions
for all 80 evaluated arrays and evaluates them with logits at the existing final
completion barrier. It reads only scalar flags on the CPU; state arrays remain in
MLX. It retains every prior layer completion boundary and the snapshot arithmetic.
A failing flag rejects publication and latches admission closed. Re-run full-model
state/logit/trace preservation and pressure before accepting the optimization.

1.1.0 — 2026-10-22 — Opt-in implementation and scheduler measurement contract;
consolidated finite-check completion and explicit tuning limitations.
