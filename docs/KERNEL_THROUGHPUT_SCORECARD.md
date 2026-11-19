# Kernel audit and native server throughput — 2026-10-24

Status: experimental native short-context qualification; **150 generated tok/s end to end is not achieved**. Apple M5 Pro, 20 GPU cores, 48 GB. Exact Combined workhorse model identity: 509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b. No base-model substitution, Python model worker or JSON model transport.

Follow-up: the compiled-MTP stall below is now repaired by sharing materialized immutable weights across shape caches. See [the compiled/eager controls](COMPILED_WEIGHT_SHARING.md): latest eager37.05 single /37.00 aggregate E2E tok/s; compilation is not promoted as a speed win. Historical failed artifacts below remain evidence of the original defect.

## Final repaired HTTP control

FP32/full finite checks, eager, group40, fused recurrence, reference attention/expert dispatch, regular fixed-capacity KV, MTP width4, scalar native execution. Command, binary/source hashes, exact token streams, clocks and shutdown counters are in [the final run](../evidence/tonight-kernel/http-rowwise-width4/summary.json).

| Streams | E2E generated tok/s (aggregate) | Mean TTFT ms | Post-first decode tok/s per lane | Callback ITL p95 ms |
|---|---:|---:|---:|---:|
| 1 | 36.10 | 496.07 | 49.83 | 67.87 |
| 3 | 36.22 | 1127.94 | 15.71 | 157.02 |

Learned acceptance: **96.42% (2799/2903)**, excluding known seeds. This ledger is cumulative over warmup, both cells and open arrivals, not a per-cell acceptance measurement. Speculative publication bursts mean near-zero p50 ITL is not near-zero model compute time.

Each closed cell contains 2,000 exact-reference generated tokens (4,000 total), with measured 1,000/1,500/2,000-token windows. Four fixed prompts, 64-token request cap, shared host, explicit profile order; this is not a statistically controlled backend ranking or broad held-out agent qualification. Eight open-loop requests succeeded and eight received bounded overload refusal. No producer errors, deadlines, failed/cancelled scheduler requests or residual native state in this final run. Proposer commits 4169 + no-draft baseline commits 95 reconcile exactly to 4,264 archived delivered tokens including warmup/open traffic.

The scheduler continuously arbitrates admitted work; closed cells use synchronized waves. MTP native model execution is scalar, with no physical MTP tensor packing. Three streams do not establish threefold GPU throughput.

## Repairs and rejected candidates

- Fixed Rust MTP admission/receipt bytes to match context-cap native KV; the prior fixed-shape integration failed before generating a valid MTP receipt.
- Strengthened the component judge to compare complete fixed-cap KV, with correct hidden offsets. Default multirow dense head projection failed count2 chunk/serial logits at NRMSE 2.683946e-4. Rowwise dense arithmetic preserves the cached FP32 matrices and one-row hot path; all sizes1..8 now pass unchanged 1e-5 tolerances, immutable branches, pressure/cancellation and the 512-position bound.
- Wired typed GPU-selected u32 receipts through the atomic target scheduler; masked selections use logits. GPU selection preserves finite total-order semantics including signed zero/ties/subnormals. Separate target HTTP smoke: 2,000 exact-reference tokens, 21.29 single-lane E2E tok/s; no claimed matched speedup.
- Global sorted expert dispatch: strict63 comparisons and256 frozen tokens pass, but decode regressed to18.76 tok/s. Packed up/gate: same numerical gate passes,24.60 tok/s and32.40 GB peak versus25.86 tok/s and20.12 GB in the initial checkpoint control. Both remain opt-in; reference stays default. These executor results are not E2E measurements.
- Pre-norm target-to-MTP hidden ablation regressed:45.96 decode tok/s,158/174 acceptance, versus post-norm48.62,166/172. Reverted the hypothesis. These exploratory microbench results predate the final dense-row repair.
- Width8 exploratory HTTP reached38.36 single-lane E2E tok/s but88.34% acceptance, below90%; it is not the high-acceptance winner. The original width6 attempt timed out after one warmup token and failed the60-second drain. Its FAILED artifact is retained; component shape6 success after repair alone does not qualify HTTP width6.

## Verification and limits

Default and debug-validation Rust suites:198 passed each. TS7.1/tsrs and28 TS tests pass. Offline independent/corruption tests:46 pass. C ABI ASan/UBSan:120,035 counted checks plus additional invalid-config refusals; upstream MLX is uninstrumented. Timing sanitizer and GPU total-order selection judge pass. Independent readers recompute timing, compare frozen outputs and validate every final AMST archive through the production Rust verifier; see [verification](../evidence/tonight-kernel/independent-verification.json).

Rust-analyzer was installed and run as requested. It reported one Send/Sync diagnostic in the HTTP spawn path; Cargo's compiler/all-target Clippy pass. That analyzer diagnostic remains recorded, not claimed clean. NVIDIA skill catalog was inspected; CUDA-specific skills were not applied to the Apple Metal path. Custom integer Metal kernels remain integer-only; these MLX numerical experiments are a separately labeled FP32 lane. Proposed random production sampling values remain inactive.

The 512-token native context is still a hard qualification limit. Native paged attention, TurboQuant KV, wider-context agents, semantic held-out repair, full-model integer calibration and production sampling are not qualified by this run. No production-ready or150 tok/s claim is made.

## Next gates toward150

1. Bound and isolate the compiled multi-token verifier evaluation stall; pass full committed-state/output and memory-pressure gates before measuring or promoting it.
2. Increase verified block yield at fixed quality: the final proposer ledger emits only3.04 committed tokens per counted target call. Width8 traded away acceptance. A calibrated multi-token proposer/DSpark integration under the existing automaton must improve useful block length and measured ns/token, not acceptance by suppressing coverage.
3. Profile native quantized MoE dispatch with GPU captures against the reference expert control. Sorted/fused flags did not establish a win; keep numerical and memory gates intact.
4. Requalify sustained mixed-context agent workloads, physical batching, admission fairness and cache ownership before native paging/TurboQuant or deeper context promotion. Expose each implemented control in the [command index](COMMAND_FLAGS.md), keeping planned switches explicitly unsupported.

Compiled-MTP follow-up: the explicit group4/full-state gate expired at the preregistered180-second bound without a completed report. The sampled target eval stack again enters MLX allocator/scheduler checks; reducing completion grouping did not resolve it. Eager remains the default, compiled MTP remains unqualified, and the abrupt stop is recorded in `evidence/tonight-kernel/mtp-compiled-group4/FAILED.json`. See upstream evaluation mechanics for context, not proof of cause: https://github.com/ml-explore/mlx/blob/v0.32.4/mlx/transforms.cpp .

The final shipping rebuild includes the subsequent parser-only HTTP width guard; measured binaries remain identified by each run’s command artifact, separately from the shipping binary hashes.

Final eager committed-state regression also passed widths1/2/4/8 against independent sequential target execution, immutable-parent checks and zero-byte drain, at NRMSE≤1e-5 and maximum absolute state error≤1e-3. This counting-prompt state gate is not broad acceptance qualification. Operational HTTP MTP windows are restricted to1/2/4/8 before weights load; nonpower widths remain bounded low-level experiments until their HTTP qualification is completed.
