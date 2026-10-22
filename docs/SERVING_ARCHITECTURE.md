---
title: "Apple serving architecture, tuning controls, and qualification roadmap"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-22"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: ".gitignore"
    lines: "1-9"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-452"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-28"
    role: "config"
  - path: "backend/rapid_worker.py"
    lines: "1-165"
    role: "primary"
  - path: "build.rs"
    lines: "1-54"
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
  - path: "fixtures/native-model.path"
    lines: "1-1"
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
  - path: "judges/benchmark.py"
    lines: "1-51"
    role: "test"
  - path: "judges/cordic.py"
    lines: "1-30"
    role: "test"
  - path: "judges/generate_cordic.py"
    lines: "1-44"
    role: "test"
  - path: "judges/macro.py"
    lines: "1-14"
    role: "test"
  - path: "judges/model_ab.py"
    lines: "1-93"
    role: "test"
  - path: "judges/resources.py"
    lines: "1-35"
    role: "test"
  - path: "judges/run.py"
    lines: "1-202"
    role: "test"
  - path: "native/mlx_bridge.cpp"
    lines: "1-165"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-46"
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
    lines: "1-415"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
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
    lines: "1-81"
    role: "primary"
  - path: "src/serving/native_mlx.rs"
    lines: "1-925"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-563"
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
    lines: "1-426"
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
related_documents: ["SPEC15_IMPLEMENTATION.md", "SAMPLING_POLICY_STATUS.md", "NEXT_STEPS.md"]
tags: [scheduler, kv, turboquant, tuning, hardening]
---
# Executive summary

Build the first complete model-serving vertical slice before further isolated kernel tuning: immutable model loading, real prefill/decode, regular KV production and consumption, cancellation, and a reproducible quality baseline. Then introduce continuous batching and TurboQuant behind the same typed backend and cache interfaces. This document proposes architecture and commands; it does not implement or activate them.

The target is an Apple-native inference server with Triton-class serving contracts, observable scheduling, model management, and repeatable performance qualification. It is not a claim of NVIDIA compatibility or completed accelerator coverage.

# Problem and current boundary

The runtime already has bounded Rust admission, three in-flight Metal command buffers by default, callback-owned resource lifetimes, integer kernel chains, background benchmark validation, and Spec 15 cache ownership controls. Current workers dispatch bounded requests; they do not continuously batch model tokens. Cache payloads are CPU-owned opaque bytes with a fixed bytes-per-block configuration. There is no actual model KV attention consumer, model repository, or serving transport yet. AMX and ANE execution remain unqualified.

The existing deterministic Q1.31 kernels remain their own numerical contract. General pretrained inference requires operator coverage, activation ranges and scaling, attention normalization, and model-level correctness evidence; integer GEMM/CORDIC success does not establish these.

# Proposed command and configuration architecture

Use one typed, versioned configuration shared by CLI, scheduler, backend and telemetry. Reject unknown fields and flags. Precedence: documented defaults, configuration file, allowlisted environment variables, explicit CLI overrides. Print every resolved value and its origin, then hash the canonical resolved configuration. Model artifacts and qualified profiles have separate immutable fingerprints. Secrets are redacted and excluded from public diagnostics.

| Proposed command | Purpose |
| --- | --- |
| `rust-server inspect MODEL` | Weights, architecture, tokenizer, attention/recurrent layers, numeric formats, hardware compatibility |
| `rust-server config check --config server.toml` | Validate budgets, units, capability constraints, and qualification requirements without loading a model |
| `rust-server config print --resolved --config server.toml` | Show effective settings and provenance |
| `rust-server serve --config server.toml --profile PROFILE` | Start a qualified immutable deployment profile |
| `rust-server bench --config server.toml --workload WORKLOAD` | Reproducible mixed-load serving measurements |
| `rust-server tune --config server.toml --search SEARCH --workload WORKLOAD` | Bounded offline candidate search; emit a report and candidate profile |
| `rust-server verify --profile PROFILE --suite SUITE` | Quality, isolation, cancellation, pressure and numerical gates |
| `rust-server doctor --config server.toml` | Capability, allocation, dependency and health diagnostics |

These names are proposed, not currently accepted by `src/main.rs`. Existing commands remain available. Never silently reinterpret existing flags.

| Proposed controls | Meaning / constraints |
| --- | --- |
| `--model`, `--model-version`, `--backend metal-fixed|mlx|coreml` | Explicit artifact and backend capability selection; proposed backends require implementation and qualification |
| `--max-in-flight 2|3` | Preserve existing bound, default 3; never increase silently during tuning |
| `--threadgroup-threads auto|64|128|256|512` | Per-kernel qualified selection; `auto` uses measured profiles and preserves existing family defaults until qualified |
| `--scheduler iteration`, `--max-batch-sequences`, `--max-batch-tokens` | Bounded iteration composition, with model-specific shape compatibility |
| `--prefill-chunk-tokens`, `--max-queue-delay-us` | Profile chunk candidates 256/512 initially; do not present these as measured winners |
| `--ttft-budget-ms`, `--tpot-budget-ms`, `--admission-max-queued` | SLO objective and bounded admission; numerical values require workload qualification |
| `--memory-budget-mib`, `--kv-budget-mib`, `--workspace-budget-mib` | Account actual allocations under one global budget; reserve explicit driver/OS headroom |
| `--kv-codec regular|turboquant`, `--kv-profile PATH` | Codec plus qualified numeric/layout profile; no implicit codec fallback or live reinterpretation |
| `--prefix-cache off|exact`, `--cache-policy PATH` | Exact identity-safe reuse; explicit Spec 15 policy configuration |
| `--validation-policy PATH`, `--metrics-listen`, `--trace-sample-rate` | Explicit validation posture, bounded observability; no request-readable sampling secrets |

Mandatory policy files have no hidden production defaults. Model/profile identity, validation obligations, namespace isolation and memory safety cannot be disabled by a tuning flag. Profile changes apply at an explicit model-generation boundary. Changing KV format drains or segregates old cache generations; it never relabels resident bytes.

Tuning searches a finite declared grid, under fixed model, quality suite, thermal conditions and workload. Optimize quality-qualified requests/tokens meeting SLOs, not raw dispatch count. Reject failures before ranking. Keep held-out workloads, repeated trials, confidence intervals, candidate hashes, and rollback profiles. Automatic promotion is a separate explicit deployment action.

# Scheduler overview

```mermaid
flowchart LR
 A[Admission and tenant budgets] --> B[Per-model lane queues]
 B --> C[Iteration planner]
 C --> D[Reserve KV and workspace]
 D --> E[Backend encode: at most 3 in flight]
 E --> F[Native completion]
 F --> G[Commit state and stream tokens]
 G --> C
 F --> H[Release leases and budget]
```

One Rust coordinator owns scheduling state. Each iteration admits compatible decode sequences and bounded prefill chunks. Compatibility includes model generation, adapters, numeric contract, codec layout and backend support. Do not impose arbitrary batch-size preferences without measured kernel benefits.

Use interactive, code and plan lanes from Spec 13, with deadline urgency inside bounded weighted service shares. Preserve continuation preference while adding aging and an explicit minimum prefill service allocation so continuous decode cannot starve new work. Count attained service and DAG critical-path hints where explicitly supplied; never infer workflow structure from opaque prompt text. Slow policy adaptation stays outside the token path.

Reserve actual KV/workspace bytes before admitting work. Acquire block leases before encoding and retain them until native completion, including cancelled receivers. Tool pauses release compute slots and retain KV only under explicit bounded TTL. Cancellation removes queued work immediately; submitted work completes resource cleanup before reclamation. Never hold coordinator locks across await or GPU work.

The present cache audit requires quiescence. For continuous serving, design generation-based snapshots and deferred per-block reclamation after completion fences, so an audit need not drain every GPU lane. Keep the current proven quiescent path until the replacement passes concurrent ownership tests. Three buffers bound the GPU submission pipeline; they do not mean only three client sequences or authorize reordering dependent decode steps.

Use a single unified-memory broker for weights, KV, activations, pooled buffers, CPU validation snapshots and staging. Include allocator fragmentation and driver headroom. Share immutable compiled pipelines where safe rather than duplicating every compilation per worker. Route to GPU, AMX or ANE only when an implemented backend passes capability, output-quality and end-to-end cost gates; using every accelerator simultaneously is not itself an objective.

# Regular and TurboQuant KV integration

Separate logical cache identity/residency policy from physical codec layout. Retain logical 16-token blocks if appropriate, but replace the single fixed bytes-per-block assumption with per-codec arenas and actual byte reservations under the global broker. Codec descriptors include version, K/V formats, layer/head geometry, packing/alignment, scales/norms, rotation/codebook fingerprints, numeric rules and backend kernel ABI. Include the full descriptor hash in cache identity alongside weights, tokenizer, adapters, positional/attention configuration and namespace.

Proposed interfaces: `KvCodec::descriptor`, `allocation_bytes`, `encode_append`, and `validate`; an attention backend consumes typed `KvView` handles with explicit completion dependencies. A reference decode/materialization path supports testing. The optimized attention path should consume packed KV or fuse unpacking, avoiding whole-cache expansion each token. Ownership guards must retain every buffer backing the view. Partial tails use copy-on-write; migration is explicit decode/re-encode into a new identity, never byte reinterpretation.

Regular KV is the numerical baseline, retaining the backend's approved native representation. FP16/BF16 would require an explicit additional model-backend numerical contract; this proposal does not change the user's integer-only MSL kernels. A fixed-point model backend needs specified per-layer scales and a matching regular fixed-point KV baseline.

TurboQuant is a lossy vector codec, not ordinary weight quantization and not a drop-in cast. The paper uses random rotations and scalar quantization; its inner-product construction adds a residual QJL correction. K and V paths therefore need separately specified accuracy objectives and compatible attention math. Store required norms, codebooks and transforms in the format, and measure their real byte and execution costs. Published experimental quality results do not qualify our models or Apple implementation. [Primary paper](https://arxiv.org/html/2504.19874v1).

A pure integer TurboQuant port is a separate research gate: rotation range analysis, multiword/widened accumulation where needed, normalization, rounding, saturation and error bounds must be specified and independently checked. Do not call a floating-point rotation or reconstructed attention path deterministic Q1.31. Select approved codec profiles rather than exposing arbitrary bit counts as production-ready combinations.

For hybrid architectures, recurrent/GDN state has a separate state interface with checkpoint/replay rules. A KV codec must not be applied to recurrent state merely because both occupy cache memory. First inspect the chosen model's layer census.

# Hardening and adversarial qualification

| Boundary | Required evidence before promotion |
| --- | --- |
| Model / cache identity | Wrong weights, tokenizer, adapter, RoPE, codec, namespace and generation cannot reuse state |
| Codec / attention | Reference vectors, zero/extreme inputs, malformed packed tails, long contexts, position boundaries, independent attention oracle; held-out model quality and logit drift |
| Scheduling | Mixed prompt lengths, saturated decode, bursty arrivals, tenant abuse, aging/fairness, bounded queue delay, cancellation at every state |
| Memory / callbacks | OOM injection, cancelled receivers, callback errors, shutdown, stalled GPU completion, zero lease leaks and double frees, accounted peak bytes |
| Lifecycle / API | Slow clients, disconnects, oversized inputs, model unload/reload under load, authentication/isolation, bounded streaming backpressure |
| Validation | Selected samples cannot drop; version-fenced checks, explicit unfinished shutdown accounting and failure admission latch |
| Sustained performance | Proposed one-hour mixed-load runs; thermal/power state, TTFT/TPOT p50/p95/p99, goodput, fairness, actual resident bytes and errors |

Sampling S1–S5 proposed probabilities and switching thresholds remain inactive by Stewart's ruling. Existing fixed-cadence results remain BENCH/DETERMINISTIC with no production random-sampling claim. An exact integer kernel oracle and a lossy-codec quality gate are separate obligations; one cannot replace the other.

# Ordered delivery and exit gates

1. **Real model baseline:** inventory an immutable model; implement backend load/prefill/decode and real regular KV. Gate: reference output/logit comparison, isolated exact prefix reuse, cancellation and bounded memory. Keep fixed-point operator gaps visible.
2. **Iteration scheduler:** token-level batching, chunked prefill, lane fairness, byte admission and completion-safe state ownership. Gate: mixed-load quality and tail-latency comparison against the same unbatched backend, with no starvation/leaks.
3. **Dual-codec KV:** typed descriptor/arenas, regular reference codec, then experimental TurboQuant attention integration. Gate: held-out quality thresholds set before measurement, long-context tests and end-to-end memory/latency gains including metadata and transforms.
4. **Serving contracts:** health/readiness, streaming/cancellation, versioned model repository and lifecycle; HTTP/gRPC inference and optional OpenAI-compatible facade. Gate: protocol/lifecycle tests and rollback under load.
5. **Apple qualification:** workload-specific Metal dispatch profiles and shared pipeline cache; then separately qualified AMX/ANE adapters. Gate: evidence across supported chips/OS versions and sustained workloads.
6. **Gold-standard release:** reproducible model/profile manifest, quality-qualified goodput scorecards, tracing/metrics, installation and recovery documentation, and independent adversarial review.

The highest-value next implementation is step 1, including the KV producer/consumer seam. More isolated GEMM throughput cannot establish a competitive inference server.

# Literature and design references

Spec 13 §7 supplies iteration scheduling, chunked prefill, lanes and two-timescale control. Spec 15 supplies current ownership and cache policy semantics. These are design inputs, not proof that those scheduler features are implemented.

Triton's dynamic batcher exposes per-model queueing, delays and limits; sequence-aware state requires appropriate routing rather than treating all requests as stateless. Its serving documentation also separates model management, backends, cancellation and observability. This informs the proposed service boundaries, not an assertion of compatibility. [Official batcher documentation](https://docs.nvidia.com/deeplearning/triton-inference-server/user-guide/docs/user_guide/batcher.html).

# Implementation details and verification

Proposed ownership: Rust configuration/coordinator/broker; explicit Rust/C++ backend boundary; Metal attention and codec kernels; TypeScript 7.1 tooling for supported control interfaces. No executable files changed for this proposal. Existing source evidence remains authoritative only for the implemented kernel/cache prototype. Document review checked proposed commands against current CLI, fixed block allocation against codec needs, and completion ownership against scheduler plans. No new runtime test outcome is claimed.

# Deployment and next steps

Keep this proposal draft pending review. Implement each stage with its own design, README changes, pressure tests, adversarial review and full project gates. Define workload-specific quality and SLO thresholds before selecting a tuning winner. No proposed flag, sampling rate or new backend is activated by this document.

# Retrospective

The native lifetime and bounded-cache foundation is useful, but the missing model path determines the next milestone. Codec flexibility and scheduler throughput both depend on explicit numerical contracts and byte ownership.

# Changelog

1.0.0 — 2026-10-22 — Codex — Proposed configuration, scheduler, regular/TurboQuant KV boundaries, hardening gates and ordered delivery.

# Implementation status update — 2026-10-22

The first experimental model integration is now documented in [MODEL_INTEGRATION.md](MODEL_INTEGRATION.md): real Qwen through a private Rapid/MLX worker, bounded Rust token scheduling and explicit dual-codec prefix storage. `model-run` and `config check|print` are implemented with a subset of controls. Earlier command tables remain a proposal for the wider serving system. Genuine tensor batching, native packed attention, HTTP/gRPC and a full integer model are not completed by this milestone.
