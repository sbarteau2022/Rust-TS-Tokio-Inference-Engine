# Diagnostic sequence (2026-10-22)

All real-model runs use the designated Combined workhorse. Intermediate logs
record development experiments; only the final reduction implementation and its
final reports are bound to the final source/binary manifests.

1. `initial-trace.log`: original fused attention, first differing layer 3 mixer.
2. `attention-oracle.log`: original fused attention vs scalar FP64 on actual QKV.
3. `explicit-matmul-trace.log`, `explicit-matmul-qualification.log`: explicit
   matrix operations reproduce the failure; rejected implementation.
4. `row-dispatch-trace.log`: single-query SDPA fixes the long-context experiment,
   but `row-dispatch-synthetic-failure.log` disproves general correctness at short
   context. Rejected implementation.
5. `reduction-trace.log`: final FP32 elementwise/reduction attention. The scalar
   FP64 oracle gate is active for every traced attention call. Tracing is opt-in;
   production calls perform no CPU oracle or trace snapshots.
6. `final-extended-qualification.log`, `final-contract-qualification.log`: frozen
   model gates using the final release binary. Full-model independent reference
   and broad quality qualification remain absent even when local gates pass.

The scalar harness checks analytic uniform, nonuniform, causal, cached, GQA,
extreme-logit and malformed/nonfinite cases under ASan/UBSan. The FFI harness
checks 7,000 malformed descriptors including null/oversized trace capacities;
prebuilt MLX itself is not sanitizer-instrumented. Leaks are not claimed tested.
