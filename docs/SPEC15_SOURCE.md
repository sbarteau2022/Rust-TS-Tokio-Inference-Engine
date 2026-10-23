---
title: "Spec 15 — user-supplied amended source"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22T06:27:43Z"
implementation_date: null
author: "Stewart Barteau; supplied design text"
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
    lines: "1-98"
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
  - path: "judges/native_seed_http.py"
    lines: "1-84"
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
    lines: "1-267"
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
related_documents: ["SPEC15_IMPLEMENTATION.md", "SAMPLING_POLICY_STATUS.md"]
tags: [source, provenance, cache]
---

The text below preserves the supplied amended specification. Implementation choices, qualification limits, and the later ruling to keep sampling values inactive are recorded separately in SPEC15_IMPLEMENTATION.md and SAMPLING_POLICY_STATUS.md.

15 — Meta Automaton for Standard Models — Design Spec
Status: Derived 2026-10-22 from Spec 14 (Nested Memory Graphs) at
Stewart's order. Amended 2026-10-22 (Stewart): §5 adds five binding
rules that must be specified before any implementation — safe
reclamation, cache identity, coverage scope, deterministic decay, and
pressure handling — and corrects two claims carried over from Spec 14
(the scope of "exact" coverage, §2/Rule 3; and the cost bound of a
wrong plan, §1/Rule 5). Spec 14 nests the hybrid server's KV cache in the
one-mind map's hyperbolic tree; a standard model (e.g. the Qwen MoE)
has no map, no carriers, and no Poincaré coordinates — there is
nothing to nest into. This spec is the same meta-automaton with the
host replaced by the two trees a standard model actually has: the
token-prefix radix tree and the plan tree. Everything in Spec 14 that
never depended on geometry carries over unchanged (§1); everything
that did is translated in §2; what the translation loses is stated
plainly in §3, not smoothed over.
0. What this is — and what it is not
The meta-automaton (Erlang — Meta Automaton — Full Scope) is a
  control design over measured signals. It was never a geometric
  object: it audits, anchors, prefills, and releases. Only its
  original host was geometric.
This spec changes the host, not the automaton. No new memory system
  is introduced: the storage substrate is unchanged (Spec 13 §12.1 —
  standardized exact token blocks, 16-token allocation, blocks as the
  eviction / copy-on-write unit; refcount protocol with frees
  committing at step boundaries).
The standard model's attention math is untouched, exactly as in
  Spec 14: the automaton governs residency and assembly of blocks,
  never the attention computed within them.
1. What carries over unchanged
The supervisor. One meta-automaton holds full scope over the
  four cache knobs: reuse, recompute, speculation (guess-ahead), and
  state/placement.
The per-turn audit. Every turn, the audit asks: does the cache
  hold the context the next turn(s) need, and what fraction of the
  cached tokens covers the lookahead window (multi-step; e.g. 50% of
  the KV covering the next 3 turns)?
Coverage from the plan. Under plan-before-executing, the tool
  calls a block of work will issue are declared before execution, so
  the tokens they will touch are known with high probability. The
  plan is the prediction; no learned predictor is used anywhere
  (speculative-automatons commitment, Spec 13 §1).
Audit outcomes. Covered → hold and replay (tiers 1–2).
  Uncovered → release to the outer tiers (3–4) to dissipate.
  Needed-but-absent → predictive prefill of the plan-declared context,
  scheduled into idle (tool-wait) windows where possible.
Bounded error (corrected — see Rule 5). The audit re-runs
  every turn and re-planning is already a runbook move, but a wrong
  plan's cost is NOT bounded by one turn: mis-anchored and
  mis-prefilled blocks persist until their anchors are released or
  their mass decays below threshold. The bound is (anchor budget) ×
  (anchor release latency), stated precisely in Rule 5. This
  correction applies to Spec 14's claim as well.
The residency rule. Store only the context necessary to
  complete this block of work.
Anchors. Plan-covered blocks are anchored (leak rate ~0);
  anchors are the successor concept to TTL pins. Refcount discipline
  is dominant over everything: anchored or in-use blocks are never
  reclaimed, and frees commit at step boundaries.
Dissipation as law, not verdict. Forgetting is exponential
  decay under a dynamic per-block leak rate set by the audit
  (plan-covered ~0; off-plan fast; global memory pressure raises
  rates everywhere). Exponential decay is memoryless — the automaton
  needs only a block's current mass and current rate, no history
  bookkeeping: stateless and deterministic.
SAM's reduced role. SAM remains the re-tiering pass, driven by
  plan-derived coverage. The SAM eviction automaton stays retired as
  a scoring judge; what remains is a dumb sweeper that returns bytes,
  no decisions taken.
2. The translation — what replaces the geometry
Nesting host: two trees, one join. In Spec 14, blocks live on
  the map's tree and inherit parentage from it. Here:
the radix tree (token-prefix tree over block hashes — the
    standard prefix-cache structure) supplies storage parentage: a
    block's parent is its prefix ancestor, and context shared by
    every branch is stored once at the shared prefix and inherited
    by reference, copy-on-write at divergence — the same sharing
    Spec 14 got from the map, supplied here by exact token identity;
the plan tree (the runbook's declared structure: packets,
    subtasks, branches) supplies work parentage: every block of work
    owns a node, its blocks hang off that node, and branch closure
    releases the branch's private blocks — the residency rule stays
    structural, not a policy something must enforce, because
    lifetime comes from the plan tree exactly as it came from the
    map tree.
**"Geometry": depth and divergence instead of radius and
  distance.** There is no continuous space. The discrete quantities
  that do the same work:
generality = radix depth: shallow nodes (system prompt, plan
    prefix, shared tool schemas) are the shared center — costly to
    lose, anchored by default; deep nodes are private detail;
closeness = longest common prefix: two contexts share exactly
    the blocks above their divergence point, no more;
position = the block's path in the radix tree plus its owning
    work-node in the plan tree. Placement, recall order, and death
    are decided from these, never from coordinates.
Dissipation without drift. Blocks do not move; only their
  priority mass decays. Each resident block carries {refcount,
  anchored?, mass, leak rate, last-turn-used}. The sweeper, under
  memory pressure, reclaims refcount-0 blocks of lowest mass first.
  "Past the dead radius" becomes "mass below the dead threshold and
  refcount 0" — that is the definition of gone. The memoryless
  property is preserved: mass × rate is the whole state the law
  needs.
Tiers are classes, not radii. Tier 1: anchored (audit-pinned,
  never reclaimed while anchored). Tier 2: resident, plan-covered.
  Tier 3: resident, off-plan, leaking fast. Tier 4: mass below
  threshold, awaiting the sweeper. The audit moves blocks between
  classes every turn; SAM's pass re-tiers by plan coverage.
The audit, restated for a standard cache. Required context for
  the lookahead window is computed from the plan as block-hash
  sequences (declared tool calls and their schemas render to known
  tokens; the user's next message is the one unknowable input and
  enters as new blocks when it arrives). Coverage = |required ∩
  resident| / |required| over the window, counted exactly from the
  block table — over the DECLARED, TOKENIZED context only (Rule 3):
  future tool outputs have no tokens yet and are outside the count,
  handled by reservation, never by the exact figure. Outcomes as in
  §1: anchor the covered, prefill the needed-but-absent into
  tool-wait windows, raise leak rates on blocks no open branch
  requires.
Recall. Assembling a context = walking its radix path and
  gathering the branch's blocks; resident context is never
  re-prefilled (context is paid for exactly once, at creation — the
  "paid-for" doctrine). A block that is gone is re-prefilled:
  recompute is the fallback knob, priced by the audit like any
  other. If the model carries recurrent state (hybrid stacks), that
  state is snapshotted at branch points or rebuilt by bounded
  suffix replay from the branch anchor (Spec 13 §5); a pure
  transformer has no such state and this clause is inert.
3. What the translation loses (named, not hidden)
Graded reuse is gone. Hyperbolic distance made near-miss
  blocks serve with a small patch-up. A radix tree shares
  exact-prefix-or-nothing: one divergent token voids sharing below
  the divergence point. Block granularity (16 tokens) bounds the
  loss to the blocks after divergence, but there is no patch-up.
No geometric recall. Nothing selects or weights a neighborhood
  by distance; torAttention over the nested cache has no analog. The
  model attends over exactly the assembled blocks, with its own
  attention, unweighted by any host geometry.
One scale, not three. Spec 14's forgetting law ran at three
  scales — mind, recurrent state, cache — because the monster's
  geometry and the GDN decay gates share it. On a standard model
  the law runs at the cache scale only.
Depth is a weaker signal than radius. Radix depth measures
  token-sharing, not semantic generality; a long shared prefix of
  boilerplate sits shallow without being important. The audit's
  plan-coverage signal is the primary judge (as it already was in
  Spec 14's revision); depth is only the default when no plan covers
  a block.
4. References
Spec 14 — Nested Memory Graphs: the hyperbolic original; §1 above
  is its automaton, §2 its host with the geometry removed.
Spec 13 §1 — the speculative-automatons commitment. §5 — SAM Monte
  Shuffle and SuffixReplay. §12.1 — standardized blocks and the
  refcount protocol (dominant over this spec). §12.2 — keep/swap/drop
  arithmetic, which under this design sets dissipation rates instead
  of verdicts, as in Spec 14.
5. Five binding rules (Stewart, 2026-10-22 — required before implementation)
Rule 1 — Safe reclamation
Leaf-only. A radix node may be reclaimed only when it is a leaf
  of the resident tree (no resident children). Reclamation proceeds
  leaf-upward, one level per pass at most; an interior node with
  resident descendants is never reclaimed, because its descendants'
  ancestry runs through it. (Spec 13's rule, preserved.)
Two holds, never conflated. A cache reference (refcount > 0)
  means at least one live context names the block; it bars
  reclamation. An in-use lock means a currently executing step is
  reading or writing the block; it bars reclamation AND bars
  copy-on-write sharing for write. References are held by structures;
  locks are held by steps, for the step's duration only.
Anchors are a third, distinct state. An anchored block is
  reclaim-ineligible and its leak rate is 0 until the audit releases
  the anchor. Anchor release is an audit act, never a side effect of
  decay or pressure.
Step-boundary frees. Reclamation decisions made during step N
  commit at the boundary ending step N — unlink, free-list return,
  and refcount finalization happen only there. No executing step can
  observe a block vanish or change identity beneath it. (Spec 12.1
  protocol, preserved.)
Rule 2 — Cache identity
A block's cache key is
  **H(parent_key, token_block, model_id, tokenizer_id,
  quantization_id)**: the parent block's key (prefix ancestry is
  inside the identity, so identity is path-dependent and sharing is
  exactly common-prefix sharing), the block's token bytes, the model
  identity (weights name + revision/hash), the tokenizer identity
  (tokenizer + chat-template version), and the quantization identity
  (KV quantization scheme + parameters).
Token-block identity alone is INSUFFICIENT: identical tokens under
  a different model, tokenizer, or KV quantization are different KV
  bytes. Sharing them would be a correctness fault, not a
  performance bug. There is no cross-identity sharing of KV blocks.
Rule 3 — Coverage scope
Exact counting applies ONLY to declared, tokenized context: plan
  prompts, schemas, and tool-call arguments already rendered to
  tokens. For that scope, coverage is counted exactly (Rule per §2).
Future tool outputs and the user's next message are UNKNOWN at
  audit time — they have no tokens and cannot be counted. The audit
  reports two separate figures and never merges them: (a) the exact
  covered fraction over declared context, and (b) a reservation
  estimate for expected unknown outputs, labeled an estimate.
  Predictive prefill applies to declared context only; unknown
  outputs are handled when they arrive.
Rule 4 — Deterministic decay
Mass and leak rate are FIXED-POINT quantities (Q1.31, the stack's
  arithmetic), never floats. The decay update is
  mass ← mass · (1 − rate), computed in fixed point with
  round-to-nearest-even, saturating at 0 (no wraparound).
Updates run at EPOCHS: one decay application per audit cycle (turn
  boundary), never continuously and never from wall-clock time.
  Epochs are turn counts. The same (initial mass, rate sequence,
  epoch count) yields the same mass, bit-exact, on any machine.
Rates come from a small ENUMERATED set of rate classes (anchored =
  0; covered; off-plan; pressured), assigned by the audit. Global
  pressure multiplies rates by a factor from an enumerated pressure
  table, also fixed-point. No other rate values exist.
Threshold behavior: the dead threshold is a fixed constant in mass
  units. A block becomes reclaim-eligible (subject to Rule 1) when
  mass < threshold, STRICTLY, evaluated only at epoch boundaries —
  eligibility can never change mid-step.
Rule 5 — Pressure handling and the wrong-plan bound
Bounded admission. When anchored + referenced + locked blocks
  exhaust the pool, new work queues to a DEFINED bound (queue depth
  and maximum wait, in epochs). Beyond the bound, admission REJECTS
  with an explicit pressure error — counted and visible — rather
  than blocking forever or reclaiming the unreclaimable. Anchors,
  references, and locks are never overridden by pressure (Rule 1).
Priority. Queued admissions are served continuations-first:
  turns of already-open branches before creation of new branches.
  Abandoning an open branch wastes its resident context; the queue
  order protects sunk residency.
Anchor rationing. The audit may not anchor beyond a defined
  anchor budget (a fixed fraction of the pool). Anchor requests past
  budget degrade to tier-2 residency (covered, unanchored) and the
  degradation is written to the audit record — a plan cannot pin the
  whole pool, and cannot do so silently.
The wrong-plan bound (corrects Spec 14 and §1). A wrong plan's
  mis-anchored and mis-prefilled blocks persist until anchors are
  released or mass decays below threshold. Worst-case cost =
  (anchor budget) × (anchor release latency in epochs) plus the
  wasted prefill of the mis-declared context. The audit's per-turn
  re-run and re-plan (which re-issues anchors) is the mechanism that
  keeps release latency small — but the bound is stated as budget ×
  latency, never as one turn.
Dependency — the sampling policy
The production validation posture this server ships under (dynamic
random sampled validation, Stewart's ruling 2026-10-22) is a
LOAD-BEARING dependency of every production claim made on this
design: the claims hold only if the sampling policy is right. That
policy is not yet specified to the standard of the five rules above.
It requires its own precise rules — selection randomness, the
rate-switching automaton, floors and probation, no-drop commitment,
and claim accounting — before implementation, at the same precision.
A draft is attached to this spec's delivery for Stewart's ruling;
it is PROPOSED, not decided.