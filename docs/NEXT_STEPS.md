---
title: "Next stages toward an Apple native inference server"
document_type: "architecture"
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
related_documents: ["DESIGN.md", "../README.md"]
tags: [rust, metal, prototype]
---
> Numerical update: float/matrix-kernel descriptions and measurements below refer to the original prototype. The active integer/shuffle contract is documented in [FIXED_POINT.md](FIXED_POINT.md) and README.md. The user confirmed Q1.31 storage; sin, cos, atan2 and tanh are now wired through the runtime. The runtime now has semaphore-capped 2/3-slot completion callbacks; see ASYNC_COMPLETION.md. Earlier single-worker descriptions below are historical.

# Current delivery order — native Combined

The typed native decoder and lexical I/O boundary are implemented. The constrained 20-case smoke passes; the original raw 16-case profile remains 15/16. Earlier stage tables below are historical and do not describe current completion status.

1. Context drift is corrected by the FP32 attention reduction reference: logit NRMSE 0.000000379 / state 0.000000336 at context 512, with trace controls and an independent scalar attention oracle. Preserve the failed fused/matmul/short-context experiments and unchanged fixtures. Resolve remaining semantic qualification failures without changing expected answers; operator-level attention evidence does not qualify the full model.
2. Sampled independent projection checks now pass (871 CPU / 871 GPU probes, including both decision-head channels); r10 still chooses `no` on both backends. Implement independent checkpoint-byte ingestion and scalar GDN/recurrent, normalization and RoPE checks next, then full-logit/tokenization parity. Sampled channels and a CPU rerun of the same model equations do not establish a full-model oracle. Preserve r10 and its expected answer; do not fit a correction to the held-out fixture. Broaden domain and actual scheduler cancellation/restart qualification after fidelity is established.
3. Move state to completion-owned device handles; prove snapshot/fork/rollback parity before genuine tensor batching and scheduler performance qualification.
4. Integrate native paged regular/TurboQuant attention with hybrid prefix-state boundaries and measured quality/memory/SLO gates.
5. Evaluate a custom drafting head only with exact target verification and atomic hybrid-state rollback. Native MTP and proposed production sampling stay disabled; ANE/AMX execution stays capability-gated until measured.

# Context

The source document explicitly limits this milestone to one kernel compiler pipeline. The user’s broader goal is an Apple-native counterpart to NVIDIA’s serving ecosystem. NVIDIA Triton Inference Server (serving/model management) and the Triton kernel language/compiler are separate layers; this milestone implements a small kernel-compiler foundation.

# Decision

Keep the conventional Metal/MLX architecture as the baseline. Use the separate `Special Ops/Meta/13-rust-mlx-hybrid-server.md` as the serving design reference. This spike does not replace its MLX backend or prove token generation. Retrieval through the one-mind map remains an isolated context-in/context-out plugin; hyperbolic cache policy stays phase 2.

| Order | Next stage | Concrete judge | Dependency | Target |
| --- | --- | --- | --- | --- |
| 1 | Persistent device, reusable UMA arenas and bounded compiled-pipeline cache | Same full parity suite; concurrent ownership stress; measured cold/warm and memory caps; cache keys include graph, layout, dtype, device and compiler options | This pipeline | Completed — evidence/resources-summary.json |
| 2 | Second kernel family and explicit backend interface | Match MLX across shape/dtype/tail/adversarial cases; demonstrate real reuse of IR/layout APIs | Stage 1 | Completed — both family parity judges |
| 3 | Model repository and MLX-backed loader per spec 13 | Tensor/quantization census; same immutable model artifact; forward and sampler parity token for token against incumbent lanes | Separate model-forward design | After kernel/backend contract |
| 4 | Tokio serving transport and lifecycle | Health/readiness, model load/unload, cancellation, deadlines, bounded admission and streaming under concurrent clients; process recovery tests | Qualified loader/forward | After model parity |
| 5 | Iteration scheduler, exact KV and continuous batching | State and logit parity, isolation, refcount/CoW edge cases, replayed workload latency/fairness and memory limits | Exact model state | Separate preregistered judge |
| 6 | Incumbent server comparison | Same artifact/prompts/sampler on oMLX, rapid-mlx, SGLang and DSpark dspark/dflash where available; include acceptance/waste, TTFT, throughput, memory and thermal context | Serving and scheduler | Before promotion |
| 7 | Optional phase-2 cache-policy experiment | One-variable A/B behind cache-policy seam, same harness and output-quality gates | Qualified conventional baseline | Only after baseline |

# Consequences

The current prototype sets up the goal because trace validation, typed lowering, shared Metal execution, supervised submission and evidence collection now run together. It does not establish a production inference server. Do not promote tiny GEMM host-latency comparisons into model-throughput or GPU-superiority claims. The user has explicitly broadened the long-term goal to CPU AMX and ANE alongside Metal. Those accelerator backends now have a qualification roadmap in ACCELERATOR_ROADMAP.md; neither is registered as executed. Full MLIR remains a later design decision.

# Alternatives considered

Building transport before model parity would create an endpoint with unqualified inference behavior. Replacing the MLX serving backend with this one-kernel spike would discard model operator coverage. Both are deferred until their prerequisites pass.

# Literature grounding

This roadmap is engineering planning, not a published-research result. It inherits the supplied source spec and spec 13; local results are indexed in the README.

# Changelog

1.0.0 — 2026-10-21 — Codex — Dependencies and explicit next-stage judges.


# Current next action after resource/backend stages

Stages 1–2 are now implemented and judged; see RESOURCE_IMPLEMENTATION.md. Proceed next with model repository/loader and exact forward/operator parity before transport. Accelerate CPU/AMX and Core ML/ANE proofs can be developed as separate capability-gated lanes under ACCELERATOR_ROADMAP.md. The gold-standard target includes dynamic precision/shape coverage, sustained throughput, memory, latency, fairness and thermal behavior; no current small-kernel result establishes that target.

# Current next action after Q1.31/CORDIC

The six integer graph families are now the active numerical contract. Implement async Metal command-buffer completion with bounded in-flight permits and retain each buffer until completion, including cancellation/error paths. Then measure grid choices, define per-model scale/range calibration and qualify full model operators. The earlier model-first ordering above is historical planning; async completion is the next proposed runtime stage.

# Current next action after asynchronous completion

The 2-slot default / 3-slot optional semaphore and native completion callback path are implemented. Next compare kernel grid/reduction variants under the exact Q1.31 contract, measure queue overlap and cold/warm latency at both limits, and evaluate a shared immutable pipeline cache. Preserve aggregate buffer limits and report lane-local versus device-wide metrics. Model calibration, operator coverage, quality and thermal serving tests remain required before promotion.

## After the 64/128/256/512 experiment

Completed all 1000/1500/2000-request windows at caps 2/3, three repetitions each, with exact validation enabled. See GRID_THROUGHPUT.md and evidence/macro-summary.json. Preserve current defaults: this macro benchmark is dominated by CPU oracle work and does not establish a model-serving grid winner.

Next: add a separately labeled production-style performance mode using preregistered correctness checks outside the timed window, retain the validation-inclusive benchmark as a regression gate, and measure representative model graphs with sustained thermal windows. Then evaluate shared immutable pipeline caches and end-to-end model quality before AMX/ANE routing or throughput claims.

## After production sampling and kernel chaining

Full CPU validation is now test/debug-feature gated, runtime default cap is 3, and every 1000th successful receipt receives a bounded background check. Linear stage chains run in one command buffer with explicit activation resource barriers. Next qualify representative model graphs, define a persistent service health endpoint for sampled-check failures, and measure sustained thermal throughput. AMX/ANE routing and model quality remain separate qualification work. See PRODUCTION_PIPELINE.md.

## After Spec 15 control-plane implementation

The opaque-byte cache controller and native-completion step leases are implemented; see SPEC15_IMPLEMENTATION.md. Next connect an actual model KV producer/consumer and define model-version/tokenizer/quantization fingerprints from immutable artifacts, then prove state/logit equivalence under prefix reuse, tail forks, cancellation, and pressure. Hybrid recurrent models additionally require real checkpoint/suffix-replay integration. Review the explicit cache tables against measured memory and work profiles.

The dynamic random sampling proposal remains inactive by Stewart's ruling. Correct and approve S1–S5 (probation completion, per-version/lane transitions, asynchronous exposure, no-drop/admission scope, shutdown accounting) before implementation or production sampling claims. Existing fixed-cadence observations are BENCH-MODE.

## Current serving roadmap proposal

See [SERVING_ARCHITECTURE.md](SERVING_ARCHITECTURE.md) for the current ordered delivery plan, proposed configuration/CLI, scheduling, dual-codec KV, and release gates. Start with actual model prefill/decode and regular KV. Proposed flags and TurboQuant are not implemented or qualified. Earlier next-action entries above are historical stage notes.

## After actual Qwen / dual-codec integration

The MLX baseline model path is now implemented; see [MODEL_INTEGRATION.md](MODEL_INTEGRATION.md) for the current boundary and recorded A/B. The next action is held-out quality and sustained SLO qualification plus genuine tensor batching for hybrid attention/recurrent state. Then wire completion-fenced Spec 15 prefix/recurrent reuse, packed KV attention and streaming serving APIs. A full integer Qwen model still needs additional operators and calibration. Regular KV remains the default; sampling draft values remain inactive.

## Fabel/Cyber correction and next-round order

First identify and fingerprint the intended fine-tuned checkpoint or complete adapter; the retained OptiQ A/B had no Fable adapter. The inspected Fable smoke/nativeprobe trees have configuration but no adapter safetensors. Next verify actual fine-tune loading, repeat regular/TurboQuant comparisons with domain quality gates, and then qualify genuine tensor batching. Details and failure boundaries are in [MODEL_INTEGRATION.md](MODEL_INTEGRATION.md#checkpoint-correction--fabelcyber-qualification-pending).

## Current direction — native Rust first, fused Opus/Cyber target

The selected checkpoint is `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined`; no further model-path clarification is needed. Native Rust-owned MLX tensor execution and joint completion-ledger admission replace the external worker as the default foundation. See [NATIVE_MLX_BACKEND.md](NATIVE_MLX_BACKEND.md) for the Rapid/oMLX/SGLang pattern and its implementation.

Next: persistent native weight handles and immutable fingerprinted manifests; mixed 4/8-bit and optional MTP quantization support; Qwen attention, gated-delta recurrence and MoE routing; exact-boundary recurrent and token-sliceable KV contracts; then genuine tensor batching, generation congruence/domain quality and regular/TurboQuant A/B on this fused checkpoint. Integrate each with completion-owned reservations. Native tensor/matrix results are not evidence that the model is ready. The proposed production sampling policy remains inactive.

## Model scope ruling — 2026-10-22

Implement the native workhorse path for Qwen3.6 Combined MoE first. Abliterated Qwen3.8 27B is the only other model integration target. No other checkpoint or model family is currently in scope. Keep `/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined` as the workhorse smoke target; bind the exact Qwen3.8 abliteration artifact before its own integration or qualification. This narrows the target set without changing the native-first roadmap or claiming either full model path is complete.


## After persistent native weights and recurrent tensor batches

Persistent checkpoint ownership, exact mixed affine quantization, attention/recurrent/router/expert operator interfaces and completion-committed recurrent row batching are implemented. This advances the native operator foundation; earlier full-model work remains unqualified.

1. Compose Combined's complete hybrid layers: embeddings, convolution history, normalized gated recurrence, attention projection gates/RoPE/KV, sparse/shared MoE, residuals, final norm and output head. Match the checkpoint's norm sanitization convention explicitly.
2. Bind native tokenization and template semantics to the fingerprinted workhorse artifacts. No remote code, Python worker, replacement model or JSON transport.
3. Extend row transactions to attention KV plus convolution/recurrent state as one model step. Prove prefill/decode equivalence, batch permutation/isolation and cancellation at every completion boundary before optimizing CPU snapshots into persistent device state.
4. Wire model steps into bounded scheduling and regular/TurboQuant KV interfaces; test admission saturation, retraction, restart, prefix boundaries and concurrent isolation. An adapter interface alone is not codec execution proof.
5. Qualify held-out Opus/Cyber tasks and long-context continuation against an explicitly identified native baseline. Report TTFT, token latency, aggregate tokens/s, memory and quality by concurrency with exact model/kernel/quantization identities. Operator/projection throughput is not model-token throughput. Enable neither production sampling proposals nor a production-ready label without the separate gates.

## After complete native hybrid generation

The former layer/tokenizer/unified-state milestones are now implemented for Combined; see [NATIVE_QWEN_GENERATION.md](NATIVE_QWEN_GENERATION.md). Strict held-out qualification remains failed at 15/16. Regular native KV works; the historical external TurboQuant adapter does not qualify native compressed attention.

1. Establish independent reference logits/tokenization for this exact checkpoint and explicit activation profile. Investigate strict format compliance with a separately designed decoding contract; preserve the original failed suite unchanged.
2. Qualify held-out Rust/Cyber tasks and actual long-context/concurrent workloads, including cancellations and restart. Record identities, precision, TTFT, token latency, memory and throughput together; expand beyond the 16-item smoke.
3. Replace CPU snapshots with completion-owned device state, then implement actual tensor batching without losing atomic row publication or cancellation isolation.
4. Integrate native paged regular KV and TurboQuant attention codecs with unified recurrent/convolution prefix boundaries. Require numerical/quality A/B and reclamation pressure tests before enabling codecs.
5. Only after quality and resource gates, promote serving transports, tune dispatch/workspace policies and benchmark Apple accelerator capabilities. Keep unsupported ANE/AMX paths capability-gated and sampling proposals inactive.

Custom drafting-head candidate: evaluate a checkpoint-bound hidden-state head with exact target verification and atomic KV/convolution/recurrent rollback; native MTP remains disabled. The design and unresolved intermediate-state requirement are in [NATIVE_QWEN_GENERATION.md](NATIVE_QWEN_GENERATION.md). No custom head is implemented or qualified yet.

## Exact output-choice profile

The original 16-item unconstrained suite remains an immutable diagnostic. A separate explicit choice profile constrains the borrowing question to canonical `yes`/`no` paths and adds four balanced semantic questions; both answers are always legal. This fixes the output-format contract without changing weights, prompts, expected answers or raw generation. Keep constrained compliance distinct from unrestricted instruction-following qualification. Expand independent reference and domain tests before promoting the full model.

Monster's boundary-transducer approach is a candidate for typed internal interchange, not a drop-in replacement for Combined's trained token vocabulary. Preserve exact checkpoint tokenization at the model boundary until an independently qualified adapter/model exists. Reversible byte transport cannot correct a model's generated punctuation, and carrier IDs alone do not prove native symplectic IR execution or throughput.


## After complete composed operator checks

Independent native-loaded checkpoint-byte verification and scalar checks of
convolution/recurrent normalization, routing, embedding, gating and all active
projection outputs now run. They found and corrected BF16 embedding affine
rounding inside the F32 profile. The frozen cancellation answer still fails;
full-model qualification is not promoted.

1. Establish a separately composed reference graph with independently owned
state, configuration interpretation and boundary tokenization for Combined.
The current scalar checks on native operands cannot replace this gate.
2. Investigate generic decoding/template profiles only as separately labeled
experiments; preserve the raw, contracted and extended fixtures and thresholds.
Any reasoning profile needs bounded tokens, phase/cancellation state and fresh
held-out cases; never special-case r10 or constrain away its wrong answer.
3. After quality gates, advance completion-owned device state, actual tensor
batching, paged regular KV and separately qualified TurboQuant attention. Keep
MTP/drafting, ANE/AMX and proposed sampling rates gated by their own evidence.


## After requested failure discriminators

Stock mlx-lm, isolated from Rapid/DSpark and loaded from the same indexed main
shards, reproduces both frozen answers. Native baseline has neither speculation
nor prefix reuse nor quantized KV. The cancellation margin remains thin across
stock BF16/F32 and native F32. Core drop semantics probes agree; an async-cleanup
follow-up is precision-sensitive. Preserve original exactness and semantic gates;
use explicit choices only where the API declares that output contract.

Do not treat this agreement as all-route graph equivalence or fix a known model
miss by rewriting expected answers. Finish paired real-prompt embedding margins,
then extend reference comparisons to the frozen suite before assigning a wider
fidelity label. Keep production readiness separate from backend fidelity and
measured model behavior. Device-state/batching/KV work stays experimental until
its numerical, cancellation, resource and quality gates pass.

## After the 60-execution stock comparison

Exact generated trajectories match stock F32 on all frozen executions (44 unique
prompts), including both known failures. The native EOS boundary was corrected
to checkpoint top-level configuration; accepted contract fingerprints now agree.
Quality remains 15/16 raw, 20/20 contracted and 23/24 domain in both lanes.
This does not promote full-model readiness.

1. Localize first-layer/state drift on domain s10 and r11 using paired intermediate
traces at identical prompt/chunk boundaries. First-answer logit NRMSE reaches
0.06537 and centered NRMSE 0.06559 despite matching argmax/outputs. Preserve
untouched stock as the reference and distinguish an intentional precision policy
from an implementation discrepancy before choosing a numerical acceptance gate.
2. Add fresh Rust/Cyber held-out questions after freezing a bounded reasoning/
format profile. Keep the original semantic and orthographic misses visible; never
teach an expected answer through an output-choice mask or change the old labels.
3. Design completion-owned device KV/convolution/recurrent state with transaction
publication, cancellation, fork/prefix isolation and drain accounting. Compare
against the snapshot reference before enabling actual tensor batching.
4. Qualify paged regular KV and native TurboQuant as separate codec policies;
then measure concurrent TTFT, tail latency, throughput and memory together.
MTP/custom drafting, physical MLX command-buffer bounds, AMX/ANE and production
sampling proposals remain separately gated. No new production claim is made.

## After first-divergence localization and causal control

Layer 3 full attention introduces the first >1e-4 material stage drift against
stock; earlier differences are small reductions. On identical Q/K/V, explicit
native FP32 attention is much closer to independent FP64 than stock SDPA. Changing
only stock's attention function to that profile closes full-graph drift and yields
60/60 exact trajectories with maximum first-answer logit NRMSE 8.76e-7. Preserve
the ordinary stock baseline separately. Retain native arithmetic; no model miss
or production gate is waived by this experiment.

1. Implement the checked native session/state table in DEVICE_STATE_OWNERSHIP.md,
including stale/foreign IDs, atomic publication, memory/slot limits and cleanup.
Keep the CPU snapshot ABI/reference intact and keep serving defaults unchanged
until the lifetime and native ABI pressure tests pass.
2. Wire Rust immutable StateRef lifetimes and worker-only reclamation. Test dropped
futures, full queues, branches, shutdown with live references and fault retirement;
then run the preregistered snapshot/device model A/B and copy-byte measurements.
3. Only after those pass enable scheduler device state with a new backend
fingerprint. Measure sustained TTFT/tail latency/throughput/memory before tensor
batching, paged regular KV and separately qualified TurboQuant attention.
4. Address remaining model behavior through fresh held-out bounded-profile
experiments, preserving the existing 15/16 raw and 23/24 domain failures. MTP,
AMX/ANE, physical MLX command-buffer caps and proposed sampling remain gated.


## After completion-owned device state and the first scheduler smoke

Implemented: checked native state versions; worker-only Rust reclamation;
independent resident limits; explicit snapshot export/reference; opt-in scheduler
adapter; monotonic token-publication timing; 1/3-lane A/B and retained first-run
regression. Consolidated finite checks improve the observed paired aggregate
output by 10–11%, with unchanged token trajectories. Both model quality misses
remain; no default backend promotion is implied.

1. Repeat randomized/alternating sustained snapshot/device windows across prompt
lengths, generation lengths and concurrency, with thermal/load controls and
confidence intervals. Add true client streaming/transport timing separately;
current TTFT is Rust submission to internal validated token publication.
2. Profile the remaining 40 layer completion boundaries, routing and projection
kernels. Design graph-wide device evaluation behind an explicit capability gate;
prove cancellation/peak-memory/state fidelity before removing barriers. Then
implement state-safe tensor batching with per-sequence KV/convolution/recurrent
ownership and benchmark aggregate throughput versus per-lane tail latency.
3. Qualify paged regular KV and TurboQuant independently against this reference,
with long-context held-out output checks. Do not infer codec correctness from
state-transport parity. Drafting/custom head work needs measured quality and
acceptance/verification overhead; speculative acceptance is currently N/A.
4. Preserve original held-out failures and qualify fresh Rust/Cyber workloads.
Physical MLX command-buffer limits, AMX/ANE, active dynamic production sampling
and a broad Triton-style deployment claim remain separately gated.


## After the native greedy verifier and retrieval smoke

The first opt-in linear verifier is implemented; see NATIVE_SPECULATIVE_VERIFIER.md.
It returns all candidate-position logits and commits unified state with bounded
replay. Retrieval accepts 25.31% of proposals on four authored code/security probes
but does not improve measured throughput. It remains off the production scheduler.

1. Spec 15 now supplies optional measured draft admission/arbitration and can choose no speculation (SPEC15_DRAFT_ADMISSION.md). Next audit/train a compatible custom MTP or hybrid proposal head, with separate
training/calibration/qualification data and Rust-native execution. Do not activate
checkpoint MTP tensors based solely on their presence.
2. Derive and qualify factorized recurrent acceptance to reduce replay, retaining
the current transaction as the state oracle. Require near-tie token comparisons
and branch/cancel/context pressure before combining with tensor batching.
3. Implement exact page ownership and regular paged attention before page-prediction
experiments. Monte Shuffle may reorder application-managed residency/prefetch;
missing attention or skipped layers are separately labeled approximate modes.
4. Repeat sustained alternating no-speculation/speculation windows, include prefill,
TTFT/tail latency, concurrency and quality. Four 64-token probes are not completed
agents. Native MTP, mixed speculative tensor batching and broad readiness remain open.


## After Spec 15 proposer integration

The shared admission/arbitration interface and native completion guards are implemented. Keep them opt-in. Supply a qualified Rust-native MTP/custom proposer next; retrieval remains subject to measured cost rather than acceptance alone. Serving still needs actual streaming/continuous scheduling, namespace-scoped cost windows, a quiescent audit driver and state-safe tensor batching. Measure real agent TTFT/inter-token tails and multi-lane throughput only after those integrations. Exact paging, compressed KV, physical command-buffer proof, AMX/ANE and active production sampling remain separate gates.


The next proposer needs a worker-confined hidden-state tap bound to the same native state version, a checkpoint-bound head artifact and separate training/calibration/qualification data. Implement it through DraftProposer with the completion guard, then compare held-out output preservation and total proposal/verification/replay cost. The plugin interface alone does not supply hidden features or trained weights. Before production admission, qualify observation-window length and a justified gain margin using sustained alternating windows; the current strict empirical cost comparison can change verdict on timing noise.

## Native MTP continuation — 2026-10-23

Completed strict 42-tensor sidecar validation and isolated MLX ingestion via native-mtp-inspect. Next implement device-owned post-normalized target hidden handles, shifted-token MTP prefill and its separate attention state; apply per-module quantization explicitly. Then register the native proposer under Spec 15, verify rejection/cancellation/state recovery and qualify matched workloads before exposing operational tuning controls. The head has not yet generated a draft.

## Post-norm payload continuation — 2026-10-23

Opt-in device-owned hidden payloads are implemented and the focused real-model 1/2/4/8 gate passes, including scalar normalization and charged slot-pressure recovery. Next add mixed BF16/affine MTP projections, explicitly sanitized sidecar norms, independent draft attention state, shifted-token prefill and native proposer registration. The installed Rapid injection code assumes uniformly packed sidecars in its explicit-sidecar path; this checkpoint has BF16 fusion/router plus mixed packed attention/expert projections, so it must retain per-module metadata instead. This is an observed local compatibility boundary, not a claim about all Rapid versions.

## Native MTP transaction continuation

The mixed MTP operator and conservative target-verified decoder now exist. Complete real-text width-1/2/4/8 qualification and bind its raw evidence. Then retain accepted hidden windows/draft prefixes to eliminate unconditional replay while preserving the same commit/rollback semantics. Integrate the stateful proposer with Spec 15 completion guards and failure-inclusive costs before enabling native scheduler admission. Measure actual net decode speed, TTFT and inter-token latency after this integration; no speedup is established yet.

The cached Dspark proposer needs eight intermediate target taps, its own six-layer backbone and Markov/confidence operators; it cannot consume the existing final-norm MTP handle directly. Port as a second native proposer only after artifact compatibility and numerical gates, then arbitrate by measured committed progress per cost. Keep the designated Combined workhorse and existing comparison server identity separate.

## After accepted-window reuse pilot

The native width-4 pilot shows 33.97 versus 20.35 decode tok/s with output congruence, but first receipt is slower. Finish the current reuse/budget hardening gate, then connect the stateful MTP transaction to Spec 15 admission and completion-owned failure/cost accounting. Optimize target-first-token publication and reduce duplicate draft synchronization only under the same state gates. Extend measurement to 1k/1500/2k windows and single/multiple streaming lanes before any serving promotion. The six-layer cached Dspark remains a separate unported candidate; this pilot does not compare against it.

## After stateful MTP admission and shared scheduler streaming

`AutomatonMtp` now uses the existing controller for bounded draft admission, cost/trust accounting and fallback, with native completion-owned permits and explicit known-seed counters, and `ScheduledMtp` (`src/serving/native_mtp_scheduler.rs`) bridges it into the fair `Engine` scheduler alongside bounded per-sequence token streaming (`Engine::submit_stream` / `Request::next_token`). Re-measure the complete scheduled path across sustained alternating windows; the 33.97 tok/s pilot predates controller and scheduler integration. Next priorities are decoupling target-first-token publication from MTP initialization, comparing maintained-state baseline versus lazy draft rebuild, compatible typed-cursor arbitration across proposers, and sustained single/multi-lane agent windows. The cached six-layer Dspark still requires intermediate target taps and a native port.

## After shared scheduler streaming

The MTP adapter is connected to the shared Engine and bounded Rust token streams, with explicit local CLI policy flags. Bind the current evidence and remeasure this complete path with alternating warmed trials, 1k/1500/2k windows and mixed request lengths. Next add a transport with cancellation/deadline propagation and a correct incremental UTF-8 text decoder; current CLI events expose token IDs and final decoded text. Reduce whole-row batch head-of-line delay and first-token initialization cost while preserving validation-before-publication. Unify configuration/policy fingerprints, own the global audit driver, and distinguish maintained-state fallback cost from target-only fallback before production promotion. Physical tensor batching, paging/compressed KV, native Dspark taps/backbone and broad model qualification remain separate unfinished work.

Live metadata audit clock is now wired at the scheduler's validated boundary. Next measure the complete path with this clock enabled against identical-output target-only and no-live-audit controls: report TTFT, publication ITL distributions, decode throughput, learned acceptance, controller epochs and per-lane fairness. Then integrate HTTP cancellation/deadlines and incremental text decoding. Shared payload-cache audit planning, unified policy fingerprints, adaptive pressure and sustained/model-quality qualification remain open.

The shared Engine throughput matrix now has 24,000 exact-reference tokens across 12 cells (two orders × one/three lanes × target/no-live-audit/live-audit). Promote no broad quality claim from it. Next integrate native HTTP streaming with cancellation/deadlines and incremental tokenizer output, then replay the same timed workload over that transport. Optimize first-token initialization and burst-tail latency; physical tensor batching is necessary before claiming multi-lane compute scaling. Expose an explicit audit-clock cadence and unify policy/configuration fingerprints; batch-count trust decay varies with lane packing. Add open-loop/continuous-arrival sweeps, mixed prompt lengths, deep-context paging/KV and matched native/external controls before production promotion.

The native incremental text boundary now feeds CLI deltas and canonical final text. Implement the proposed bounded loopback HTTP + versioned FlatBuffers stream contract in the tuning plan next, including generated TS 7.1 bindings, disconnect/deadline propagation and slow-client pressure gates. Then measure transport TTFT/ITL against the retained Rust-stream matrix.

AMST v1 framing and native file/pipe output now exist with strict Rust/TS parsers and Unicode deltas. Next put this exact contract behind the proposed maintained-library loopback HTTP endpoint: bound connections/body time and bytes, propagate disconnect/deadline cancellation, enforce bounded network output and qualify slow readers/shutdown. Reuse the actual model and pipe evidence as component controls; do not treat them as network qualification. Bind generated TS 7.1 clients to the endpoint, then rerun the throughput matrix over transport.

## Persistent HTTP checkpoint and next gates

Loopback HTTP implementation connects the existing native MTP scheduler and AMST stream to a persistent process. Keep transport smoke evidence separate from the older 24,000-token scheduler matrix. First qualify actual HTTP arrival TTFT, inter-token gaps, decode and total throughput at single/three lanes under fixed and open-loop arrivals, reporting errors, cancellation, queue time, acceptance, audit epochs and exact commands. Use controlled target/no-audit/live-audit profiles with exact token parity; current native HTTP CLI exposes the live-audit MTP profile only.

Then reduce target-first-token initialization cost and scheduler row head-of-line delay; implement state-safe physical tensor batching with rollback tests. Next add native paged/compressed attention/state ownership, and port the custom model-associated Dspark artifact as a separate proposer only after validating its required target taps and backbone. Extend context and held-out tool/coding/semantic quality gates before promotion. The server smoke does not qualify a general agent workload or resolve the known held-out misses. Production sampling policy values remain inactive until explicitly configured and approved.

The Desktop OPS_BOARD documentation standard path supplied in the instructions is absent on this host. This checkpoint follows the repository's existing front matter, file map, source hashes, design preregistration and bound evidence conventions.

## After HTTP fixed-order pilot

The HTTP CLI now exposes explicit target, MTP/no-audit and MTP/live-audit profiles. Bind the current fixed-order pilot before making network performance claims. Follow it with reverse-order/no-audit controls, mixed context lengths and open-loop arrivals, reporting rejection/queue latency and fairness. Do not call synchronized request waves continuous batching. Improve first-token MTP initialization and burst-tail latency before state-safe physical tensor batching; then native paging/KV and custom Dspark remain the next integration gates. Broad held-out model qualification is still open.

## After reverse-order and bounded open-loop controls

Bind the new control evidence and retain the first pilot as historical. Next implement the preregistered SeedPending/Ready experiment: deliver the validated prefill seed, defer its exact target/draft advancement until needed, and prevent duplicate delivery or progress accounting. Compare eager/deferred state and full output under EOS, length-one, cancellation, failed advancement, backpressure and multi-lane rollback; then alternate timing profiles and measure first-to-second gaps as well as TTFT and request throughput. Do not promote a cosmetic first-token win. The proposal and falsifiers are in NATIVE_TUNING_PLAN.md; it is still design-only at this checkpoint.

After that, implement the typed per-row physical-batching state contract with exact scalar controls, then native paging/compressed KV and the model-associated custom Dspark proposer. Add sustained arrivals, direct queue-time telemetry and deeper contexts. A burst with explicit 503s proves bounded overload accounting, not a general continuous-batching, production SLA or held-out agent-quality result.
