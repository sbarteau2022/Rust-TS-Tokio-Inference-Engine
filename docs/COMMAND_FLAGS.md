# Rust server tuning controls

Status: implementation inventory, 2026-10-24. Native execution means Rust/Tokio + C++ MLX/Metal, with text/tokenization at I/O. A listed planned control is **not accepted by the parser**. Qualification and benchmark evidence, not the presence of a flag, determines readiness. Target: 150 generated tokens/s **end to end per stream**, with learned acceptance reported separately from known seeds and aggregate lanes.

## Table of contents

1. [Native server command](#native-server-command)
2. [Kernel and graph controls](#kernel-and-graph-controls)
3. [Scheduling and output controls](#scheduling-and-output-controls)
4. [Draft automaton admission and trust](#draft-automaton-admission-and-trust)
5. [Validation and numerical profiles](#validation-and-numerical-profiles)
6. [KV, state ownership and paging](#kv-state-ownership-and-paging)
7. [Native generation and qualification commands](#native-generation-and-qualification-commands)
8. [HTTP throughput benchmark controls](#http-throughput-benchmark-controls)
9. [Configuration fields and fixed limits](#configuration-fields-and-fixed-limits)
10. [Environment and build controls](#environment-and-build-controls)
11. [External comparison lane](#external-comparison-lane)
12. [Planned tuning controls](#planned-tuning-controls)
13. [Measurement and promotion gates](#measurement-and-promotion-gates)
14. [Parser sources and audit checks](#parser-sources-and-audit-checks)

## Native server command

```sh
./target/release/rust-server native-mtp-serve 127.0.0.1:PORT BODY_TIMEOUT_MS MAX_CONNECTIONS \
  --execution-profile target --tensor-batching scalar \
  --layer-group 40 --recurrent-kernel fused --attention-kernel reference \
  --expert-kernel reference --sampling device --graph-mode compiled-decode \
  --execution-timing off --operator-profile off \
  --draft-width 4 --stream-capacity 1 --qualification-trials 64 --probe-epochs 2 \
  --initial-mass 2147483647 --admission-mass 268435455 --reward-mass 536870911
```

Replace PORT with an available nonzero loopback port. Positional body timeout: 1–30000 ms; connection cap: 1–64. The seven policy flags are mandatory even for target-only operation; they do not enable drafting in target mode. Read metadata to verify actual settings. This command is an experimental tuning recipe, not a 150 tok/s claim.

## Kernel and graph controls

| Flag | Values / default | Scope and qualification |
|---|---|---|
| `--layer-group` | `1`, `4`, `10`, `40`; default `1` | Layers between explicit evaluations; final completion remains mandatory. Target and MTP HTTP. |
| `--recurrent-kernel` | `reference`, `fused`; default `reference` | Selects recurrence implementation; fused requires GPU. Custom integer kernels remain a separate numerical contract. |
| `--attention-kernel` | `reference`, `fused`; default `reference` | Fused SDPA failed prior strict numerical gates; opt-in experiment only. |
| `--expert-kernel` | `reference`, `sorted`, `fused`; default `reference` | Globally sorts flattened expert assignments, gathers matching inputs, inverts permutation before reduction. Sorted passed numerical smoke but regressed single-lane speed; keep reference for performance control. Fused concatenates matching packed up/gate weights once (12 GiB cache cap), projects once and splits; numerical gate passed but decode regressed and peak memory rose to 32.40 GB; not promoted. |
| `--graph-mode` | `eager`, `compiled-decode`; default `eager` | Persistent native MLX compiled blocks; requires GPU/FP32 and absence of `MLX_DISABLE_COMPILE`. Shared-parameter compiled MTP passes grouping4/40 state/pressure gates; explicit HTTP benchmarking is enabled, with defaults still eager. See [ownership repair](COMPILED_WEIGHT_SHARING.md). |
| `--execution-timing` | `off`, `on`; default `off` | Host graph/evaluation/validation-export counters; on/off must retain identical execution boundaries. |
| `--operator-profile` | `off`, `stages`; default `off` | Nineteen serialized diagnostic cuts; target/scalar/eager, timing off only. Not additive production latency or GPU timestamps. |
| `--sampling` | `host`, `device`; default `host` | Scalar target HTTP only for device mode. Device argmax exports a typed u32; constrained choices fall back to logits. Finite-logit and signed-zero/tie rules remain checked. Physical/MTP combinations fail before weights load. |

## Scheduling and output controls

| Flag | Values / default | Effect |
|---|---|---|
| `--execution-profile` | required `target`, `mtp`, `mtp-audited` | Target-only, learned MTP, or MTP with exclusive metadata audits. |
| `--tensor-batching` | `scalar`, `physical`; default `scalar` | Physical graph packing is target-only, equal-position one-token decode, B1–B3. Continuous scheduling alone is not tensor packing. |
| `--seed-mode` | `eager`, `deferred`; default `eager` | Deferred seed requires MTP; controls seed synchronization strategy, not learned acceptance accounting. |
| `--stream-capacity` | required integer `1..128` | Bounded per-request token event queue. Slow readers apply backpressure. |
| `--draft-width` | required integer `1..8`; HTTP MTP restricts to `1`, `2`, `4`, `8` | Verification-window capacity; known seed excluded from learned proposed/accepted counts. Width 1 has no learned proposal. |

The native server uses continuous scheduling with up to three admitted sequences. Scalar execution serializes native model work; physical mode packs compatible rows. Prefill/decode disaggregation across separate processes/devices is not implemented.

## Draft automaton admission and trust

All are required for `native-mtp-serve`, `native-mtp-generate` and `native-mtp-stream`.

| Flag | Meaning |
|---|---|
| `--qualification-trials` | Number of measured proposer qualification trials; validated by the draft policy. |
| `--probe-epochs` | Deterministic audit-epoch interval for probing a benched proposer. |
| `--initial-mass` | Initial Q31 proposer trust mass. |
| `--admission-mass` | Trust mass required for draft admission. |
| `--reward-mass` | Reward mass from successful qualification. |

Q31 masses are bounded by `MAX_MASS = 2147483647` and policy relationships; these are trust units, not probabilities of token correctness. One proposer and three work permits are currently configured. Rejected draft state rolls back; only verified output commits. Decay is by audited epochs, not wall-clock guesses. Deeper MTP proposals currently use a fixed 1.25-logit margin threshold; no public margin flag exists yet. Tuning for apparent acceptance alone can hide reduced proposal coverage: report proposed, accepted, committed tokens per target call, and wall time together.

## Validation and numerical profiles

Native HTTP currently pins **full validation and FP32 activations**. `ExecutionTuning` also exposes API/qualification-runner experiments:

| Control | Status |
|---|---|
| `validation_mode = 2` | Full device state/logit finite checks; default. |
| `validation_mode = 1` | Deterministic BENCH cadence, interval 32 native scalar calls. Not random production sampling; physical batches retain full checks. |
| `validation_mode = 0` | Explicit unchecked state experiment; device-token output still requires finite logits. Not production qualification. |
| `activation_precision = 1` | FP32 numerical control. |
| `activation_precision = 0` | BF16 eager experiment; strict FP32 parity and held-out behavior require separate measurement. Compiled FP32 blocks and BF16 must not be combined. Physical BF16 is unsupported. |

No `--validation`, `--activation-profile`, `--validation-rate` or production-sampling switch is accepted by native HTTP yet. The proposed dynamic random sampling policy remains inactive. A deterministic cadence is never represented as that policy. The native generation command's separate `--activation` flag is listed below.

## KV, state ownership and paging

Current native attention state uses fixed-capacity immutable-parent KV arrays with functional slice updates, plus convolution and recurrent state under the same lease/commit discipline. Fixed shapes are not paged attention. The Rust native context limit remains 512; server admission advertises 511 plus output accounting. Expanded C++ ABI bounds do not qualify longer native contexts.

Native TurboQuant KV, native paging, cache-tier promotion/Monte shuffle, quantized recurrent state and speculative early exits have no implemented public flags. External Rapid codec flags below are not native implementations. Cache tiering alone cannot safely omit arbitrary model layers or necessary attention context.

## Native generation and qualification commands

| Command / flags | Scope |
|---|---|
| `native-generate PROMPT MAX_NEW --activation f32\|bf16 --device gpu\|cpu --choices yes,no` | Native snapshot-generation numerical lane; choices are explicit canonical token paths, not prompt wrappers. Defaults: FP32, GPU, unconstrained. |
| `native-reference --device cpu\|gpu --full` | Independent operator/reference diagnostics; `--full` expands operator coverage. |
| `native-qualify --contracts` / `--extended` | Separate contract/extended held-out suites. Neither flag changes model weights or silently adds prompt anchoring. |
| `native-mtp-generate PROMPT MAX_NEW` + seven policy flags | Experimental native scheduler generation; typed Rust path. |
| `native-mtp-stream PROMPT MAX_NEW` + seven policy flags | Size-prefixed AMST FlatBuffer output. Text/tokenizer remains I/O only. |
| `stream-check FILE` | Validates saved binary stream. |
| `config check\|print --config FILE` | Existing configuration utility, separate from native HTTP startup. |
| `--help` | Built-in command synopsis; this document supplies the complete scope/qualification index. |

`native_group_qualification` positional controls: NEW_DIRECTORY, GROUP, recurrent mode, timing, graph mode, attention mode, operator mode, `logits|sampled` step, `full|sampled|off` validation, `fp32|bf16` activation profile, and `reference|sorted|fused` expert mode. Defaults preserve FP32/full/reference controls. `sampled` step means GPU argmax, distinct from sampled validation.

`native_mtp_bench NEW_DIRECTORY [WIDTH] [GRAPH] [GROUP]` accepts draft width 1–8 (default 4), `eager|compiled-decode` (default eager), and grouping 1/4/10/40 (default40). Compiled-MTP remains an explicit bounded experiment. Its warmed Rust-boundary 32-token decode result excludes HTTP and model loading.

`native_mtp_decode_smoke NEW_DIRECTORY [GRAPH] [GROUP]` exercises accepted-target-window commit, sequential target state/output parity, immutable parents and drain. Defaults eager/group40; compiled configurations remain bounded experiments and emit a flushed phase/memory journal.

`native_physical_batch NEW_DIRECTORY [GROUP] [RECURRENT] [GRAPH] [EXPERT]` exercises B1–B3 state/logit/rollback gates; this is not a throughput qualification.

## HTTP throughput benchmark controls

Use `node --import tsx ts/http-bench.ts NEW_DIRECTORY` followed by paired flags.

| Flag | Values / default |
|---|---|
| `--profiles` | Required comma-separated ordered subset of `target`, `target-physical`, `mtp`, `mtp-audited`; 1–3 distinct entries. |
| `--arrival` | Required `closed` or `closed+open`. |
| `--tokens-per-cell` | Required decimal integer 1000–2000; use 1000/1500/2000 fixed windows. |
| `--layer-group`, `--recurrent-kernel`, `--execution-timing`, `--graph-mode` | Same enum values as native HTTP; group/recurrent controls also support MTP; non-default graph/timing remain restricted to target labels. |
| `--expert-kernel` | `reference|sorted|fused`, default reference. |
| `--draft-width` | Integer 1–8, default 4; MTP HTTP profiles permit 1/2/4/8. Server width metadata is checked. |
| `--sampling` | `host|device`, default host; device requires scalar target label. |

Each profile measures one- and three-lane closed cells; optional open-loop arrivals retain refusals/failures. Records exact commands, binary/source hashes, shared-host state, request bodies, frozen tokens, TTFT, ITL, lane and aggregate speeds, acceptance ledger and drain accounting. Acceptance ledger is cumulative across warmup/cells, not a fabricated per-cell figure. Diagnostic operator mode is deliberately not a benchmark control.

## Configuration fields and fixed limits

`serving::config::Config` is a typed scheduler configuration. These fields are not all public native HTTP flags:

| Field | Generic validation | Native HTTP value |
|---|---|---|
| `version` | `1` | `1` |
| `max_queued` | 1–32 | 3 |
| `max_sequences` | 1–32 | 3 |
| `max_batch_tokens` | 1–4096 | 24 |
| `prefill_chunk_tokens` | 1–batch budget | 8 |
| `max_context_tokens` | 1–4096 | 511; native executor 512 cap still applies |
| `max_new_tokens` | 1–4096 | 128 |
| `model_memory_bytes` | 1 byte–8 GiB | State/workspace reservation: scalar 1 GiB, physical 2 GiB; not total weights memory |
| `max_in_flight` | 2 or 3 | 3 |

Native GPU launches execute on the owning executor thread. Three Rust work permits do not imply three independent GPU model streams. Resident slots are 64 for HTTP; parent leases and provisional children consume the explicit state budget.

## Environment and build controls

| Control | Scope |
|---|---|
| `RUST_SERVER_IN_FLIGHT=2|3` | Integer resource benchmark/runtime path, default 3; does not override native HTTP's independently configured permits. |
| `MLX_DISABLE_COMPILE` | Presence refuses explicitly compiled native profiles. Not a magic automatic mode selector. |
| `METAL_CAPTURE_ENABLED=1` | Metal capture tooling; never assume capture overhead is production latency. |
| Cargo feature `debug-validation` | Extra replay/reference checks and deterministic debug validation; no production-sampling claim. |
| Cargo feature `native-mlx` | Default native MLX/tokenizer build. |
| Cargo feature `legacy-python` | External comparison lane only; not native qualification. |

TypeScript toolchain: TS 7.1 (`npm run typecheck:ts71`) and pinned tsrs (`npm run typecheck`), plus `npm test`.

## External comparison lane

`model-run CONFIG PROMPTS MAX_NEW` invokes the external Rapid Python worker. It is not the Rust-native serving path and cannot qualify native kernel performance.

| Flag | Meaning |
|---|---|
| `--kv-codec` | `regular`, `rapid-turboquant-v4`, `rapid-turboquant-k8v4`. |
| `--max-in-flight` | 2 or 3 scheduler permits. |
| `--prefill-chunk-tokens` | Generic scheduler prefill chunk, constrained by batch budget. |
| `--max-batch-tokens` | Generic scheduler token budget. |
| `--probe-directory` | Save explicit comparison probes. |

## Planned tuning controls

These are design work, **not accepted command flags**. Names are proposals, not promises of implementation:

| Proposed area | Intended control surface | Required gate |
|---|---|---|
| Fused MoE up/gate and expert dispatch | `--moe-kernel`, shape specialization/autotune | Mixed-quant operator oracle, full graph parity, memory budget and normal-path A/B. |
| MTP profitability/depth | `--draft-margin`, adaptive width, per-proposer budgets | Held-out acceptance, proposals/target-call, p95 latency and exact rollback. |
| Native paged KV / regular / TurboQuant | `--kv-layout`, `--kv-codec`, page bytes | Full attention parity, alias/ABA/reclamation tests and deep-context pressure. |
| Prefill token budget | Native `--prefill-chunk-tokens`, `--max-batch-tokens` | Chunk-boundary recurrent/conv/KV equivalence and decode fairness. |
| Native lane/admission limits | Native `--max-sequences`, `--max-queued`, `--max-in-flight` | Bounded memory, cancellations, slow-client backpressure and drain. |
| Random production validation | Explicit policy config + OS CSPRNG | User-approved rates, no-drop sampling, accounting and failure closure. |
| Partial/dynamic forward passes | Explicit experimental execution profile | Proven dependency coverage; no shortcut based only on token confidence. |
| AMX / ANE execution | Explicit supported operator placement | Genuine native implementation and transfer-inclusive measurement. |

## Measurement and promotion gates

Report generated tokens/s end to end, TTFT, p50/p95 ITL, decode-only speed, aggregate and per-lane speed, learned acceptance excluding known seeds, proposed coverage, committed tokens per target call, batching mode, numerical profile, validation posture, model identity and held-out quality separately. Speculative publication can burst several tokens; receipt latency is not per-token kernel time. No change to model weights, quality tolerance or sampling doctrine is justified by a speed target alone.

Promotion requires rigorous full-state/logit comparison against the independent control, exact frozen tokens where applicable, typed-receipt bounds, cancellation/rollback/parent immutability, no unfinished drafts or leaked resident state, finite outputs, bounded memory and an end-to-end measured improvement. Keep unsuccessful candidates opt-in or remove them; record the failure.

## Parser sources and audit checks

Authoritative parsers: `src/main.rs`, `src/serving/native_http.rs`, `src/serving/native_mtp_cli.rs`, `ts/http-bench-config.ts`; scheduler validation: `src/serving/config.rs`. Native numerical configuration: `ExecutionTuning` and `Architecture`. Negative-test strings such as `--surprise` are intentionally unsupported. Read-only shutdown counters expose proposer and no-draft baseline commits separately; neither is silently substituted for delivered tokens. Every documented default must agree with parser/default construction; fail-closed parser tests cover duplicates, missing values, invalid enums and incompatible profiles.

`native_mtp_smoke NEW_DIRECTORY [GRAPH] [GROUP]` now exposes the same bounded compiled component/pressure gate; defaults remain eager/group1. These positional example controls do not change server defaults.
