---
title: "Adversarial review of the native kernel prototype"
document_type: "research"
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
    lines: "1-1015"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-53"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-62"
    role: "primary"
  - path: "examples/native_automaton_smoke.rs"
    lines: "1-394"
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
    lines: "1-221"
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
  - path: "src/cache/drafts.rs"
    lines: "1-592"
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
    lines: "1-30"
    role: "primary"
  - path: "src/macro_bench.rs"
    lines: "1-81"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-453"
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
    lines: "1-107"
    role: "primary"
  - path: "src/serving/model_ir.rs"
    lines: "1-244"
    role: "primary"
  - path: "src/serving/native_automaton.rs"
    lines: "1-324"
    role: "primary"
  - path: "src/serving/native_boundary.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-687"
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
    lines: "1-950"
    role: "primary"
  - path: "src/serving/native_mtp.rs"
    lines: "1-468"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-688"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-380"
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
  - path: "tests/draft_admission.rs"
    lines: "1-380"
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
