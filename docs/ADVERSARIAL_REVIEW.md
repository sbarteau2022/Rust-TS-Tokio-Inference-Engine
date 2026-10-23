---
title: "Adversarial review of the native kernel prototype"
document_type: "research"
version: "1.1.0"
date: "2026-10-23T12:43:42Z"
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
    lines: "1-102"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-64"
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
  - path: "examples/native_physical_batch.rs"
    lines: "1-213"
    role: "primary"
  - path: "examples/native_seed_qualification.rs"
    lines: "1-271"
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
    lines: "1-64"
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
  - path: "judges/native_batch_http.py"
    lines: "1-81"
    role: "test"
  - path: "judges/native_http_smoke.py"
    lines: "1-81"
    role: "test"
  - path: "judges/native_physical_http.py"
    lines: "1-88"
    role: "test"
  - path: "judges/native_seed_http.py"
    lines: "1-84"
    role: "test"
  - path: "judges/native_stream_disconnect.py"
    lines: "1-36"
    role: "test"
  - path: "judges/operator_reference.cpp"
    lines: "1-29"
    role: "test"
  - path: "judges/physical_throughput_verify.py"
    lines: "1-80"
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
    lines: "1-113"
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
  - path: "native/qwen_batch.inc"
    lines: "1-40"
    role: "primary"
  - path: "native/qwen_device.inc"
    lines: "1-61"
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
  - path: "src/serving/batch_transaction.rs"
    lines: "1-591"
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
    lines: "1-119"
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
    lines: "1-1001"
    role: "primary"
  - path: "src/serving/native_device_scheduler.rs"
    lines: "1-205"
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
    lines: "1-333"
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
    lines: "1-207"
    role: "primary"
  - path: "src/serving/native_mtp_cli.rs"
    lines: "1-344"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-275"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-436"
    role: "primary"
  - path: "src/serving/native_mtp_scheduler.rs"
    lines: "1-1057"
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
    lines: "1-391"
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
    lines: "1-14"
    role: "test"
  - path: "ts/http-bench-config.ts"
    lines: "1-27"
    role: "primary"
  - path: "ts/http-bench.ts"
    lines: "1-55"
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
related_documents: ["DESIGN.md", "../README.md"]
tags: [rust, metal, prototype]
---
> Numerical update: float/matrix-kernel descriptions and measurements below refer to the original prototype. The active integer/shuffle contract is documented in [FIXED_POINT.md](FIXED_POINT.md) and README.md. The user confirmed Q1.31 storage; sin, cos, atan2 and tanh are now wired through the runtime. The runtime now has semaphore-capped 2/3-slot completion callbacks; see ASYNC_COMPLETION.md. Earlier single-worker descriptions below are historical.

# Question investigated

Can malformed traces, incorrect dependency edges, invalid bindings, overload, cancellation or nonfinite math produce silent success or unsafe GPU accesses?

# Methodology

Single-author manual review of the complete handwritten Rust, TypeScript, C++ bridge and judge code, followed by deterministic corruption, resource pressure and live GPU tests. This is not an independent security audit. No subagents or other reviewers were used. The documentation remains draft; no merge or external approval is claimed.

# Findings and fixes

| Finding | Resolution | Evidence |
| --- | --- | --- |
| An epilogue max could hide a NaN accumulator | Preserve nonfinite accumulators for explicit result rejection | `tampered_plan_and_nan_intermediate_fail_closed` |
| Public plan fields could be altered after lowering | Revalidate dimensions, budget, input IDs, shapes, layout and names before emission/execution | Same test plus `residency_is_consumed_by_emitter` |
| Debug operation projection materialized an owned wire-op list | Runtime now consumes verified borrowed FlatBuffers accessors directly into SSA | `borrowed_lowering_matches_debug_projection` |
| Queue could otherwise grow without bound | try_send admission, 1–32 queued jobs and 4 MiB named input budget per admitted job | 100-request pressure and eight concurrent producers |
| Runtime construction outside Tokio could panic | Return a named error when no Tokio handle exists | `constructor_without_tokio_is_an_error` |
| CLI could read arbitrary-size input before enforcing budget | Bounded file readers, including a max+1 detection byte | CLI code and malformed-file checks |
| Uninitialized output could disguise missing stores | Poison output bytes with nonfinite f32 bit patterns before dispatch | Tail and zero-output GPU tests |
| Every-truncation-is-invalid test rejected harmless padding removal | Accepted truncations must decode to the identical graph | `malformed_wire_never_panics`, initial evidence retained |

# Safety argument

Only the exact load A/load B/load row bias/dot/add/relu/store dataflow can reach code generation. Names never enter shader source; only validated integer dimensions and fixed memory qualifiers do. Input lengths match those dimensions. A/B loads are guarded independently for row, column and K-tail, with zero fill. Every SIMD lane reaches all barriers; only final writes are conditional. One 32-thread SIMD group owns each output tile. Shared storage remains alive until synchronous command completion. Unsafe host writes require exclusive mutable access; opaque buffers prevent external dispatch through this wrapper. Output reads occur after Metal reports Completed.

FlatBuffers root size, identifier, version, depth/table verification and operation budget precede typed field access. Unsupported unions or dataflow are refused. No CPU fallback masks Metal failures. Exact reference validation is intentionally expensive and included in job wall time.

# Residual limits

The public low-level Gpu compiler accepts source for internal testing; untrusted remote clients are not exposed. The high-level runtime accepts trace bytes only. This is a single supervised worker with serialized GPU jobs, not a concurrent GPU scheduler. Recoverable Rust panics are caught around job work, but allocator aborts, native process crashes, GPU hangs and device loss do not have process isolation or a hard execution deadline. Queue-full and pre-admission budget errors return immediately rather than producing admitted-job receipts. Cancellation drops the receiver; admitted GPU work safely drains. There is no model loader, authentication, HTTP, KV, persistent cache, production traffic, energy measurement or long-duration thermal qualification.

Residency selection counts trace consumers plus the eight matrix-lane reuses per A/B tile. The emitter consumes and checks those decisions; this MVK always chooses threadgroup A/B staging. It is not a general graph optimizer. Accumulation uses SIMD matrix registers; the matrix-to-scalar epilogue uses a threadgroup scratch tile before bias/ReLU, all inside one dispatch. No intermediate is flushed into a separate device allocation.

# Recommendation

Proceed to a separately designed reusable backend interface and second kernel family. Keep production serving and model-level claims gated by their own parity, isolation and load tests.

# Literature review

No research-paper claims. Review evidence is local source and reproducible tests. Apple API reference: https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf .

# Time invested

Performed during this build; no independently timed review duration.

# Changelog

1.0.0 — 2026-10-21 — Codex — Manual review, findings, mitigations and remaining risks.


# Persistent-resource and second-family follow-up review

Reviewed the full backend/arena/cache implementation without subagents. Added 12 Rust tests (two backend units and ten resource/integration tests) plus the TypeScript second-family test. Cases include cache identity/source-byte pressure/LRU, independent kernel entry points, tight actual-allocation budgets, page-granularity admission, complete input overwrite/output re-poisoning, failed-capture/math cleanup, compile failure, mixed-family receipts and accurate capability reporting. Found and corrected three lifecycle/measurement risks: missing autorelease scopes, heap-vs-direct reservation mismatch, and a retained backend overlapping the recreated benchmark's allocation phase. Initial reservation failure evidence is preserved. Reference CPU map lookups were hoisted without changing accumulation order or tolerance. Remaining risks include native process/GPU failure isolation, opaque pipeline/driver memory, conservative reservations and lack of model-serving/thermal qualification. No independent audit or human review approval is claimed.


# Native operator and ingestion review — 2026-10-22

Self-review only; no independent reviewer or signoff is claimed. Explicit CLI
checks remain off the hot path. The private diagnostic bitmask rejects unknown
bits, full coverage without both oracle modes, missing trace capacity, duplicate
or out-of-range output channels and malformed layer programs before store use.
Raw-byte export validates shape/dtype/range and caps each transfer at 4 MiB;
checkpoint stamps are checked before and after verification. Scalar projection
tiles cap output channels at 256 and join all eight possible workers before
return. RoPE bounds avoid overflowing offset+tokens on hostile descriptors.

A real failure was uncovered by extending coverage to embedding lookup: BF16
affine arithmetic survived a requested FP32 output. The correction casts affine
parameters before arithmetic. The BF16 synthetic fixture and reverting-cast
negative control demonstrate that the regression detects this defect. Full
scalar coverage uses observed native operands and cannot prove a separately
composed graph. Stable semantic failure remains a failed gate, never rewritten
as a passing quality claim. Native process/GPU fault isolation, physical MLX
command-buffer cap, tensor batching and sustained serving SLO qualification
remain limitations despite the logical three-lease admission bound.


# Frozen-failure discriminator review — 2026-10-22

Self-review, no delegated or independent reviewer. The new native diagnostic uses
only fixed fixture prompts, public typed executor state, bounded 32-token decode,
actual EOS and checked finite/bounded logits. Stable logsumexp tests cover offset
invariance and malformed inputs. Its native feature requirement preserves minimal
builds. Ordinary application serving code and both original fixtures are unchanged.
Source indexing now includes examples. Bad diagnostic CLI forms reject before
opening weights. Source hashes demonstrate the isolated embedding control changes
only one arithmetic line while canonical corrected source remains untouched.

The first stock comparison was rejected as a reference after discovering MTP
sidecar-driven norm shifting. Accepted comparison uses symlinks to identical main
shards, unchanged stock engine code and exact boundary token/template parity.
Installed package RECORD and imported-module checks narrow contamination risk;
these are recorded provenance, not an independent supply-chain audit. The regex
warning is retained; parity covers tested prompts only. Stock BF16/F32 varies more
than KV precision, so no KV-only causal claim is made. The async-cleanup probe's
profile-dependent winner and thin cancellation margins remain explicit. A shared
wrong answer does not establish all-route model correctness or production quality.

# Full frozen-suite reference review — 2026-10-22

Self-review only; no delegated or independent reviewer. New offline runners
separate declared output choices from expected answers and require exact inventory,
model identity, template/token parity, contract fingerprints and complete drains.
Malformed/truncated/duplicate reports, bad vector paths, wrong model identity,
nonfinite/wrong-shape vectors, illegal trie paths, terminal/shared prefixes and
signed-zero/lowest-ID ties are covered. A missing NumPy installation skips only
vector unit tests in the portable environment; those run in the stock environment.
Both Rust examples are now explicit default-judge gates, with native feature
requirements preserving minimal builds.

The first actual run failed on a genuine EOS-language discrepancy before any
stock generation. The correction reads fingerprint-bound checkpoint policy with
bounded strict configuration parsing, top-level precedence and explicit generation
overrides. Scalar/list/duplicate EOS, invalid/unknown IDs, empty/oversized lists,
noninteger/negative/overflow values and malformed config objects are tested.
All original native generated IDs/first vectors remain unchanged by this fix.
The rejected attempt is retained instead of silently aligning stock's policy.

The accepted comparison uses unchanged RECORD-verified stock code and the exact
index-only view, excluding the previously demonstrated MTP loader contamination.
60/60 exact trajectories are observed, but only 44 unique prompts. Both old
quality misses remain. Full first-answer vectors differ substantially on some
cases (up to 6.54% NRMSE); no bitwise/numerical graph-equivalence or production
qualification claim is supported. Layerwise drift localization is the next gate.
GPU/CPU state resources, physical command-buffer limits, fault isolation, long
context, tensor batching and sustained load are not covered by this offline run.

# Layer/state localization review — 2026-10-22

Self-review only. The opt-in native capture reuses checked typed IR, existing
trace ABI and immutable snapshot state; no pointer or mutable state is exposed.
Wrong CLI arity and existing output directories reject before weights open.
Capture checks finite values, hashes, bounded shapes and drain. Source arithmetic
and default native execution remain unchanged, confirmed by 60 bitwise-identical
first vectors and generated trajectories relative to the preceding checkpoint.

Stock observations delegate to original module bodies; installed sources remain
RECORD-verified. Both original/observed controls are bitwise equal. Comparisons
slice KV to logical position and verify every cache array's shape/F32 dtype;
malformed/incomplete/out-of-order reports and geometry are rejected. Independent
cached causal/GQA tests expose off-by-one visibility and wrong head grouping.
Operator controls reject unsupported masks/scale/sinks/quantization/non-F32 input.
MLX-specific tests also run in the external stock environment, without weights.

The first bitwise difference is not hidden: recurrent reductions already differ
slightly in layer 0. The first material >1e-4 marker is layer-3 full attention.
Same-input projected K/V and recomposition controls are exact; an independent
FP64 oracle favors native explicit attention. A separately labeled SDPA-only
stock control closes the observed full-graph drift. The ordinary stock baseline
is preserved, and no low-level internal-dtype claim is inferred from these errors.
No replacement of the more accurate native operator or new throughput claim is
justified. Known quality misses remain failed; shared libraries and finite test
coverage do not establish universal equivalence or cross-hardware determinism.

The next device-state document is explicitly a design, with no implemented native
session ABI or zero-copy capability claimed. It requires worker-only destruction,
immutable branches, resident lifetime accounting and failure/cancelled-publication
retirement before enabling serving. Physical command-buffer bounds remain separate.

## Completion-owned device-state self-review

The review challenged stale/forged geometry and ABA handles, arithmetic overflow,
reservations that fail after encoding, a full inference queue during last-reference
drop, dropped response receivers, shared-prefix mutation, shutdown with live
references, and native array destruction while commands are readers. The design
uses full native descriptors, no generation wrap, pre-encoding reservation,
rollback-before-publication, a separate retirement channel, immutable evaluated
arrays and worker-only FFI. Actual-model cancellation/branch/context tests remain
separate from metadata-only sanitizer pressure. A discovered cross-executor
snapshot-owner namespace collision was corrected by sharing one checked Rust owner
allocator; native device owners retain their independent validation.

The first performance claim was rejected by measurement: eliminating full-state
copies regressed throughput 11–12% because finite checks added per-array barriers.
The optimized path keeps validation but moves reductions to the final completion
barrier; it requires a fresh full-model A/B and throughput run. Do not describe
three logical lanes as three physical command buffers, tensor batching, client
streaming or speculative acceptance. This is self-review, not independent review.


## Native linear verifier self-review

Challenged the off-by-one relation between initial logits and candidate-position
rows; zero, partial and full acceptance; EOS inside a rejected suffix; missing bonus
capacity; signed-zero and tie selection; nonfinite rejected rows; and arbitrary
cursor/state pairing. Private cursors bind state/logits, and the selector matches
existing total-order greedy semantics. The old final-logit ABI/default path is intact.

Challenged cancellation before/after verification, bonus allocation failure after
successful verification, shared prefixes, three concurrent transactions, bounded
history and foreign state epochs. Transactions publish only after unified-state
completion and retire provisional branches through the existing worker channel.
KV-only truncation was rejected as invalid for the recurrent/convolution model.
This is bounded replay, not a claim of factorized tree verification.

The measured retrieval lane reduced target calls without demonstrating a speedup;
it stays experimental. Floating chunk-vs-sequential drift remains nonzero, so finite
trajectory preservation is not an all-input proof. No stochastic, constrained-choice,
trained-head, early-exit or production-serving equivalence is claimed. This review
was performed by the implementing agent; no independent reviewers were used.


## Spec 15 draft admission extension

Challenged acceptance-only admission, forged outcome counts, cumulative overflow, cross-identity baselines, foreign controller handles, registration bounds, cold-start starvation, stale queue eligibility, duplicate probes, continuation priority, shared queue depth and epoch expiry. Nine policy tests include 10,000 audit cycles. Failed or incomplete work cannot earn false committed progress; unfinished cost gaps remain visible even after later requalification.

Challenged caller cancellation before proposal completion, command lifetime after cancelled verification, runtime abort, proposer panic, oversize output and invalid token IDs. Each native command owns the completion guard independently of its waiter. A cancelled result channel records zero internal published progress and preserves the immutable parent. The actual native smoke and default preservation tests remain model-specific evidence, not general bitwise speculative equivalence. Plugin code is trusted and must obey the completion contract. Benchmark ordering, limited cold trials and namespace cost mixing can bias profitability, so this adapter remains experimental. No trained MTP, true continuous batching, client latency or production gain is claimed.

A further fault control exhausts the native state table after block verification and before bonus replay. Both admitted target calls remain charged even though the transaction fails and publishes zero tokens; parent state and residency must recover. Target-call accounting counts admitted native commands, not successful full forward graphs.

## Native MTP ingestion review — 2026-10-23

The sidecar is accepted only for the explicit 42-tensor Combined layout. Validation checks mixed packing using each module override, not a global four-bit assumption. All required tensors are removed in turn by tests; malformed geometry, dtype, offsets, extras, truncation, trailing payload and symlinks are rejected. Metadata tests use sparse files and do not establish numerical correctness. The native MLX store is separate from the target store because target norm sanitization currently reacts to MTP keys. Artifact identity and stamp checks bind opening to the same checkpoint; this follows the existing local immutable-file trust boundary, not protection against a malicious local writer preserving timestamps. Loading establishes ingestion only: MTP forward, quantization application during projections, hidden-state alignment, transactional draft cache and profitability remain open.

## Opt-in target hidden payload review — 2026-10-23

Hidden activations are stored in an explicit optional payload, never appended to the 80-array state vector. Reservation includes count × hidden × four bytes before encoding; publication and rollback cover both unified state and hidden output. The same native generation/owner descriptor and Rust retirement lifetime bind both. Snapshot size is now computed from unified state geometry rather than total charged bytes, avoiding a hidden-byte buffer accounting error. Hidden diagnostics reject ordinary states. Matched all-position logits/state, continuation, scalar post-norm values and slot pressure are tested by native_hidden_smoke. Default no-hidden behavior requires the broader frozen-suite preservation gate; sidecar ingestion alone provides no evidence here. No MTP forward or throughput claim is made.

## MTP operator and transactional decode review — 2026-10-23

The mixed sidecar requires BF16 dense fusion/router support and exact per-module packed widths. The initial batched dense projection failed its scalar precision gate; bounded row-wise FP32 dispatch fixes that without widening tolerance or changing target-wide precision settings. Separate draft owner IDs prevent cross-table descriptor confusion; combined slots/bytes, cancellation, immutable branch repeatability and context-512 exhaustion are exercised by the operator gate. CPU MTP is refused because it is not qualified.

Transactional decode restores from the committed draft parent after every proposal and uses target hidden rows with shifted committed tokens. Known target seeds are excluded from learned acceptance. Full acceptance still replays, so no acceptance-only profitability claim is valid. Widths outside 1..8, EOS vocabulary, terminal cursors and exhausted context are refused. Real-text output comparison must include sequential target selection and unified recurrent/convolution/KV state rather than just an attention cache or synthetic MTP arithmetic. Remaining risks: cross-geometry near-tie logits, no independent whole-MTP reference, no Spec 15 completion-guarded plugin yet, and intentional replay overhead. The profile remains experimental.

## Accepted-window reuse review

Reuse is allowed only when every proposed token matches target greedy verification. Partial acceptance cannot truncate unified recurrent/convolution state: it drops the provisional state before replay. A target bonus is separately evaluated; EOS and output-budget exhaustion can finish on the verified block without bonus. Draft state always resynchronizes from target hidden rows, never from speculative recursive hidden values. The debug reference retains unconditional target replay. Tests compare decisions, unified state, exact covered token counts and forwarded-work savings from identical parents. The bounded API includes bonus/correction output in its limit. Measured target-token savings are not proof of latency improvement because splitting synchronization can add MTP calls.

## Stateful MTP admission review

The controller counts the known seed separately from learned proposals and rewards only learned acceptance. Adversarial geometry tests reject multiple/overflowing known prefixes, over-cap total candidates, mismatched branch bindings and extra claimed committed progress. Full candidate identity remains bound to the committed parent. Existing retrieval outcomes use known-prefix zero; the existing profitability, queue, epoch and 10,000-audit tests remain mandatory.

The real-model gate compares controller-admitted tokens with independent sequential target decode and checks unified state. Cancellation drops the result receiver after admission: native work drains, the ledger gains cost/observations but no published progress, the committed parent stays bitwise unchanged and unfinished remains zero. Shared resident-table pressure is injected after target verification but before draft synchronization; the failed transaction records one target call, zero accepted/committed tokens and a failure, then recovers after freeing pressure. Controller pressure cancels queued metadata and runs a one-token maintained-state fallback. A deliberately held control permit is abandoned only in the test; its unfinished accounting is expected, unlike receiver cancellation. Shutdown must drain all logical bytes. These are correctness controls, not representative throughput or production policy settings.

Remaining boundaries: prompt initialization cost is reported outside decode admission; fallback baseline includes MTP maintenance under a distinct profile identity; common typed-cursor arbitration with retrieval/Dspark, streaming, sustained multi-lane scheduling and target-only fallback without maintained draft state remain unfinished.

The real cancellation harness uses a current-thread Tokio runtime so the detached proposer cannot finish before the waiter is canceled during its first poll. Native execution still runs on its separate worker. This removes a timing-dependent cancellation-test race; the harness then yields and waits for command/accounting drain.

## Shared streaming and MTP scheduler review

Streaming validates whole backend batches before enqueueing events. The scheduler is the sole stream producer; it checks capacity before encoding, so a receiver can only increase available space during native work. A closed receiver cancels; an impossible full send poisons admission rather than dropping a token silently. A full lane yields other active lanes. A shared Notify wakes consumption/cancellation/admission/shutdown, avoiding busy encoding or a wait-loop spin. Final result drains its own stream; shutdown explicitly errors an undrained blocking stream and releases state. Tests cover capacity extremes, event/final timestamp and token equality, slow-reader fairness, encode pause, result convenience draining, cancellation, shutdown and malformed batches that must publish no tokens.

MTP rows hold a virtual consumed position distinct from the private ahead-of-publication target cursor. Input IDs must match the last published token; verified pending IDs remain request-local and bounded by the scheduler's remaining output budget. Three real-model lanes and 1/8/16-token limits check buffered order against the independent sequential target path. Choice masks and external histories are unsupported and rejected. The profile conservatively exposes context 511 because its native state consumes every returned token within context 512. This is shared iteration scheduling, not physical tensor batching or HTTP serving.

Live audit boundary review: request-side event consumption cannot establish native quiescence. The scheduler now awaits the hook only after successful whole-batch validation; malformed batches skip it, and hook failure follows the poisoned-admission cleanup path before publication. Opt-in MTP metadata audits reject payload blocks, reservations and open branches, preserving the distinction from a shared KV cache planner. The API requires exclusive ownership of the controller audit clock; unrelated concurrent users can force explicit epoch/lease errors and are unsupported. Buffered receipts advance the declared batch clock even without a new target call. This can affect admission profitability and must be measured, not assumed beneficial.

Scheduler matrix review: measured outputs must match independent sequential target prefixes, not merely one another. Group budgets reserve no more than the remaining 2,000-token cell limit; early EOS reduces actual delivered count and subsequent groups refill it, while empty completions fail explicitly. Each independent consumer checks event index, ID and publication timestamp congruence with the final receipt. Macro checkpoints sort receiver-observed times across lanes rather than adding per-lane rates. Warmup ledger counters are subtracted; acceptance excludes known seeds, and speculative-only target-call accounting is labeled. Synchronized groups limit arrival-model claims. Per-cell evidence persists before the next profile; final source and binary hashes bind the experiment. Two opposite profile orders expose but cannot eliminate shared-host thermal/load drift. Repeated prompts establish mechanical parity, not semantic quality or unseen-task generalization.

Text boundary review: independently decoding tokens corrupts split UTF-8 and can emit irreversible replacement characters. Stateful DecodeStream holds pending bytes; finalization compares canonical full decode against all published text and fails if a decoder revised the prefix. A synthetic Fuse+Replace decoder exercises revision errors during subsequent pushes and finalization. Unknown IDs and cap errors permanently poison the stream, preventing ambiguous retries after decoder mutation. Manifest stamps are checked at stream creation and finish; no per-token filesystem checks are added. Every real-checkpoint Unicode prefix is compared with canonical decode, including early stop within a code point. Final terminal replacement follows the tokenizer's existing policy and is not represented as lossless recovery of missing bytes.

AMST framing review: generated accessors alone are not a trust boundary. Deterministic byte mutation exposed an underflow panic in an additional Rust `vtable.num_fields` inspection despite root verification; that accessor was removed and explicit bounded vtable/object/string shape checks now precede verification. TS uses the equivalent byte preflight before generated access, fatal UTF-8 and byte-based string decoding to preserve leading BOM. Parsers cap pending frame storage, poison on failure and reject terminal gaps, stale sequence/index/time and aggregate limits. A writer that fails mid-frame cannot be reused or followed by success. Generated bindings regenerate from both schemas under flatc; Rust fixtures decode in TS and TS fixtures decode in Rust. File tools read through bounded chunks, avoiding unbounded whole-file allocation.

CLI disconnect scope: a native stdout pipe is closed after metadata and the first token. The expected nonzero broken-pipe result must include successful scheduler/native drain diagnostics; the captured prefix remains intentionally unfinished and must fail a complete-stream check. This proves pipe cancellation ownership, not HTTP socket backpressure, deadlines, disconnect races or network throughput.

## HTTP/1 boundary review — 2026-10-23

Reviewed ownership and framing before the native smoke: request parsing precedes model admission; header/body/connection caps are independent of scheduler capacity; numeric authority and no Origin prevent incidental cross-origin browser use (not a remote authentication claim). Unknown/duplicate controls, invalid UTF-8, oversized bodies and nontext content fail closed. The response is explicitly AMST, and clients require Finish rather than treating HTTP 200 or a valid Error as success.

Focused gated-backend tests hold a native step pending across deadline and disconnect, then release it and require zero charged bytes on shutdown. A separate full-output-slot control proves the producer stops consuming scheduler events; it does not assert that kernel/TCP buffers contain only one frame. Socket cutoffs bound stalled consumers independently of cancellation. Partial header/body and shutdown tests exercise idle connection ownership. TypeScript tests reject invalid origin/bounds before network and fragmented Error/unfinished EOF after metadata. Existing strict binary fuzz/mutation cases remain applicable.

Accepted limitations: TCP send completion and queued Finish do not establish end-user delivery. Abrupt disconnect may expose an unfinished prefix, not an Error. Synchronous native calls are not forcibly interrupted; completion remains scheduler-owned. Header/HTTP connection errors are counted separately from generation errors. Native HTTP CLI uses one explicit experimental live-audit profile, with a 511-token scheduler context and at most three model lanes; neither a general agent quality claim nor physical tensor batching follows. Production sampling draft values remain inactive. TLS/authentication and remote/public serving are not implemented.

The first real HTTP attempt completed and drained but failed its historical-reference assertion: the driver sent `Complete 1, 2, 3,` while the reference used `Complete: 1, 2, 3,`. Both binary readers decoded that first response successfully; it answered the different prompt. The failed attempt is retained under `evidence/native-http/initial-prompt-mismatch/`. The corrected rerun uses byte-identical prompt text; changing the expectation without matching input was rejected.

## HTTP timing and control review

Execution profile selection fails before loading weights on missing, duplicate, malformed or unknown values; the existing policy parser still rejects unknown controls and requires all seven policy settings. The target profile uses ScheduledDevice, avoiding an unfair maintained-MTP fallback baseline; both profiles still load the same open_mtp executor artifact, so resident head weights are not removed from the target memory footprint. The audited path preserves its validated-batch clock; the no-audit profile disables that hook only.

Offline timing tests separate TTFT, request envelope, burst callbacks, native publication deltas and post-first per-lane decode arithmetic; synthetic 1k/1.5k/2k windows count global arrival order. An initial exact floating-point assertion failed because algebraically equivalent throughput expressions differed by one final bit; the assertion now uses an explicit 1e-9 numerical tolerance without changing measured values. Native benchmark tokens must match immutable target references, and every stream must Finish. Partial observations and normalized frame archives remain on failure; successful summary is written only after owned-process drain. Fixed profile order, one warmup prompt and synchronized waves are explicit confounds, not evidence of continuous batching or production superiority.

The first pilot launch completed its real target warmup with parity and a clean drain, then failed offline artifact serialization because AMST metadata contains BigInt u64 fields. The failed attempt is retained under `native-http-pilot/initial-artifact-serialization/`. Offline serialization now preserves BigInts as decimal strings, with an explicit full-u64 regression test; no server protocol or model computation was changed by this fix.

## Reverse-order and overload control review

The native binary is byte-identical to the first HTTP pilot. The new offline driver validates explicit profile order/arrival/budget flags before directory creation or process startup. A manually advanced synthetic clock proves later requests launch while an earlier attempt is still unfinished, and a failed attempt is retained by allSettled rather than dropping later work. A subprocess-exit test proves the losing shutdown timer is canceled. Typed HttpStatusError distinguishes intentional 503 rejection from malformed AMST, a valid binary Error, EOF, parity or transport failure; only the 503 class is accepted as overload.

An independent offline verifier does not reuse the TS measurement formulas: it recomputes request envelopes, TTFT/ITL percentiles, decode rates and exact macro windows, reconstructs outstanding-client high-water from attempt timestamps, and reconciles every connection, rejection, admitted request, completed receipt and queued Finish against shutdown counters. Successful normalized frame archives must pass the Rust production verifier, then independently extracted IDs must match both recorded observations and immutable target references. Successful sequence IDs may not be reused within one profile process; rejected archives must be empty. Native state bytes, active producers/connections, failed/canceled model work and unfinished proposer accounting must all drain cleanly. Synthetic corruption checks reject retained bytes, missing completions, closed admission, producer/model failures, duplicate drain records and regressed clocks.

Limitations: all control traffic uses short prompts, shared caches and one reverse-order repetition. Open-loop attempts arrive continuously, but the fixed 16-attempt overload burst does not establish sustained production SLA, physical tensor batching or long-context quality. Clock removal changes automaton trust/admission semantics as well as work, so no-audit differences cannot be assigned solely to audit CPU overhead. Metadata arrival is not a pure queue-time measurement; server connections and client outstanding attempts are not native active-lane telemetry. Known held-out model-quality gates remain open.

## Deferred prefill seed review

The eager default is preserved and the target-only HTTP backend refuses deferred mode before loading weights. SeedPending records the unconsumed seed without changing the cursor's materialized position. A later advance must return the identical sole seed and increment materialized position exactly once before speculation; a mismatched seed/position or failed advance cannot update the old row snapshot. The synthetic wrong-position test initially caught a missing position check; it now fails closed alongside wrong-token and injected-failure cases. Deferred mode cannot be enabled after the backend has begun processing.

Counters distinguish backend seed receipts from actual advancement attempts/completions and pending release. They are observed coherently after drain, not described as end-user acknowledgement. Controller baseline maintenance is recorded only when executed and does not create another Engine output token. One-token/EOS releases intentionally omit maintenance. A gated fake backend holds advancement pending across request cancellation, proving the Engine retains completion ownership and only releases charged state after completion. The native qualification separately checks bitwise target/draft primitive state and full scheduler output parity; snapshots are offline qualification work, not synchronous CPU validation on the production hot path.

A later-row failure test also completes one deferred maintenance before failing the second row, then proves *all* stored rows retain their old pending snapshots. The completed-maintenance counter records physical work even when a later batch failure discards it, so it can overlap pending releases in failed batches; these are not a partition of delivered tokens. Engine receipts remain authoritative, and equality of completed advances plus pending releases is asserted only in failure-free/cancellation qualification cases.

The real Combined gate passed four bitwise target/draft primitive comparisons and 160 exact-reference mixed-length streams across two seed orders and one/three lanes, then forced EOS and cancellation. HTTP deferred mode passed parity, deadline, disconnect/recovery, native/TS readers and drained shutdown. Actual CLI negative controls reject unknown/duplicate seed modes and deferred target-only execution before startup. An additional rollback test initially failed to compile because StepInput is intentionally non-Clone; the fixture now builds independent inputs, without changing production behavior.

The measured tradeoff is adverse for promotion: roughly 7% lower Rust publication TTFT accompanies longer first-to-second gaps and worse ITL tails, with less than 1% aggregate throughput change. This is not a server speed win. Preserve eager default and distinguish the native timing pilot from HTTP functional smoke. Bitwise state checks cover the native primitive sequence; complete scheduler output is separately checked against prior references. Neither establishes broad model quality, physical batching, sustained service latency or independently exposed complete scheduler row-state parity.

## Typed scalar batch transaction review

The first batching stage refactors the target scalar adapter; it does not alter native kernels or claim physical batching. PreparedBatch owns parent clones and exclusively borrows the exact destination map. CompletedBatch cannot be redirected to another map; the borrow prevents concurrent mutation until failure, drop or commit. Every input is validated before an execute callback. All result positions, vocabulary lengths and finite logits must pass before a commit exists. Full-token greedy selection reuses its existing finite check rather than adding a second scan. Partial-prefill logits are checked for finite values without independent CPU model recomputation.

Synthetic gates cover duplicate/invalid later rows, resident-row overflow, geometry cohorts with retained input order, failed second-row work, wrong-position/nonfinite/truncated results, explicit rollback by dropping completed work, sequence permutations and exact lease release at commit. These prove the generic transaction discipline, not native bitwise tensor batching. The existing Engine cancellation contract still owns in-flight native completion; dropping an arbitrary public transaction future is not itself a GPU cancellation mechanism. Native target HTTP smoke is the integration gate for this adapter change; physical graph parity, native multi-row failure injection and MTP transaction integration remain future gates.

## Native equal-position graph prototype review

The Rust batch descriptor rejects empty/oversized batches, length mismatch, foreign wrapper owners, empty/exhausted contexts, unequal positions and out-of-vocabulary tokens before queue submission. Native descriptors independently validate state-table generation/ownership/inventory before reserving children. All child reservations roll back on a failed graph or publication; output handles are assigned only after every payload is published. Rust acquires retirement ownership for all returned children before post-completion manifest/receipt validation. Pressure rejection preserves admission, computation failures latch it closed, and a dropped result receiver retires all successful children after completion.

The Qwen attention/recurrent methods now obtain B from the activation shape; B=1 scalar behavior remains independently controlled. The batch graph uses batch-shaped projections/MoE and preserves per-layer synchronization for this trial. Children use immutable row views, each charged for the full packed backing. This intentionally overcounts logical storage; it does not claim a physical process-memory cap or detached row allocations. The real gate exercises actual packed children through multiple continuations and reversed row order. ABI malformed/foreign descriptors use sentinel weights under ASan/UBSan to prove rejection without weight access or output mutation.

Native numerical qualification requires exact selected-token parity and independently measured full-logit/unified-state errors within the preregistered FP32 thresholds. Numerical equality is not claimed bitwise for B>1. The native prototype is not automatically called by the scheduler or HTTP path; broader model quality, actual service batching telemetry and throughput remain unqualified. A separately rerun native MTP transaction smoke protects shared scalar graph behavior. No production sampling policy or integer Metal arithmetic changes are included.
