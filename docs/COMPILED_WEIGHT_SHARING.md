# Compiled MTP: shared immutable parameter payloads

2026-10-24. Design and gates: [native tuning plan](NATIVE_TUNING_PLAN.md). This is a native C++ MLX ownership repair under the Rust executor, not a new numerical kernel or proposer.

## Reproduction and mechanism

The original phase-journal run completed width1 decode/reference checks, then stopped in the second prompt's target hidden prefill. The180-second bound expired; native sampling reported55 GB physical footprint. It was not a proven mutex deadlock. MLX's [compile_dfs](https://github.com/ml-explore/mlx/blob/v0.32.4/mlx/compile.cpp) copies primitive-bearing graph nodes. Capturing unevaluated safetensors Load nodes into distinct compiled shape caches can therefore retain separate parameter payloads. Already-materialized arrays have no load primitive and are shared.

A real lazy-safetensors GPU fixture reproduced67,108,864 active bytes for four compiled shapes versus16,777,216 bytes after materializing the original parameter. Both paths produced exactly equal outputs, including repeated shapes. The fixture also checks absent/malformed prefixes, failed evaluation without marking ready, and the64-prefix bound. ASan/UBSan instrument the judge/helper; upstream MLX is uninstrumented. Leak detection is disabled for framework process lifetime allocations.

## Implementation and state boundaries

Before tracing, evaluate original immutable parameters under the layer prefix once per weight owner. The prefix includes a trailing separator in matching, so layer1 does not include layer10. Mark readiness only after evaluation and completion succeed; never capture request tensors in the ready set. Reuse the same arrays across callable shape caches. Preparation occurs outside tracing; callable caches are destroyed before the arrays. Eager execution, mathematical operators, precision, validation, memory limits and defaults are unchanged. No new production flag is required.

## Full-model gates

Compiled grouping4 and40 both pass widths1/2/4/8 against sequential target execution, immutable-parent checks and zero-byte drain. Unchanged committed-state limits: NRMSE1e-5 and max absolute error1e-3. Total cold runs took76.76 and75.21 seconds, including model loading. Recorded allocator peaks across starts:20.904 and20.905 GB; active bytes remain~20.130 GB while target cache entries grow80→200. These are allocator bytes, not an OS footprint measurement.

The separate compiled/group40 component gate passes all counts1..8, full512-position KV/hidden/logit comparisons, cancellation, pressure and branch immutability at unchanged1e-5 tolerance. Component success is not broad agent or throughput qualification.

Native examples perform tensor-state comparisons in process. The offline reader checks eager token/counter equality, phase completion/order, clocks and allocation bounds; it does not recompute state errors from serialized full tensors.

Raw journals, state reports, numerical reports, independent verification and source/binary identities: [evidence](../evidence/compiled-isolation/provenance.json).

## Reproduce the isolated regression

```sh
clang++ -std=c++20 -fsanitize=address,undefined -fno-omit-frame-pointer \
  -Inative -Ivendor/mlx/include judges/compiled_constants.cpp \
  -Lvendor/mlx/lib -lmlx -Wl,-rpath,"$PWD/vendor/mlx/lib" \
  -o artifacts/compiled-constants-asan
ASAN_OPTIONS=detect_leaks=0 ./artifacts/compiled-constants-asan /tmp/NEW_UNIQUE_FIXTURE_DIRECTORY
python3 judges/compiled_isolation_report.py evidence/compiled-isolation
python3 -m unittest discover -s judges -p 'test_*.py'
```

The fixture refuses an existing directory and removes only its own generated fixture after success. Model benchmarks require the pinned Combined workhorse and must run without another owned inference benchmark.

## Alternatives and the next decision

First measure compiled width4 against the eager width4 control at identical FP32/full-validation settings, fixed prompts,1/3 streams and open admission pressure. Preserve exact output, learned acceptance and full delivery accounting. Improved memory ownership alone is not a decode speedup.

If useful committed tokens per total draft+verify nanosecond remains low, the next proposer comparison can use compatible DSpark2/DFlash artifacts under the same automaton and atomic commit/rollback verifier. The workhorse directory currently contains its MTP sidecar; a separate compatible DFlash checkpoint has not been established by this audit. No substitution is implicit. Draft disaggregation needs measured draft/target overlap and resource contention on the shared Apple GPU before adding scheduling complexity. Radix/prefix reuse and paged/offloaded KV need mixed-context prefill/memory workloads; the current512-token qualification cannot establish their value. The [upstream FlashAttention3 Hopper implementation](https://github.com/Dao-AILab/flash-attention/tree/main/hopper) is not a direct Metal backend; existing Metal SDPA must pass the strict numerical gate before an attention-path promotion. None of these planned paths is advertised as implemented.

## First repaired HTTP run

Explicit compiled/group40/width4, FP32/full finite checks, reference attention/expert, fused recurrence.4000 closed exact-reference tokens plus warmup/open traffic passed independent archive parsing, metadata, clock arithmetic and full delivery accounting. Learned acceptance2807/2909=96.49%;4169 proposer commits +95 baseline commits=4264 delivered.73 successful requests,8 bounded503s, zero failed/cancelled/deadline/producer errors and zero residual native bytes.

| Streams | Aggregate E2E tok/s | Mean TTFT ms | Post-first decode tok/s per lane | Callback ITL p95 ms |
|---|---:|---:|---:|---:|
|1|36.29|499.81|50.35|67.84|
|3|35.31|1183.23|15.36|142.49|

This remains scalar MTP execution under continuous scheduler arbitration; synchronized-wave clients do not establish physical tensor batching. Cumulative acceptance includes warmup and open traffic, not per-cell quality. Against the previous eager control36.10/36.22 tok/s, this is no material single-stream improvement and a lower aggregate result; neither is a statistically controlled ranking. A fresh eager control follows to check host/time-order confounds.150 tok/s remains unmet. The useful block yield is3.06 committed tokens per counted target call, so high acceptance alone is not sufficient.

## Fresh eager control and decision

The subsequent eager run uses the same rebuilt binary, workhorse, flags except graph mode, prompts,2000-token cells and open pressure. Single-stream E2E37.05 tok/s, TTFT488.11 ms, post-first decode51.35 tok/s, callback ITL p9566.43 ms. Three-stream aggregate37.00 tok/s, mean TTFT1096.33 ms, decode16.04 tok/s per lane, ITL p95158.92 ms. Acceptance2799/2903=96.42%. Both runs pass independent4000-token closed-output checks,73 completed requests,8 bounded overload refusals and zero native/scheduler residue. This is one ordered compiled→eager pair on a shared host, not a confidence interval.

Decision: retain eager as default. Shared-constant preparation is retained for compiled reliability; the tested compiled configuration is not promoted as a speed win.198 default +198 debug Rust tests,28 TS tests/TS7.1/tsrs,49 offline/corruption tests and all-target debug Clippy pass. The standalone shared-constant judge passes ASan/UBSan (upstream MLX uninstrumented). The next performance discriminator is total proposer/verify/replay/prefill cost and useful block yield, before selecting a compatible parallel proposer or cache work.150 tok/s remains unmet.
