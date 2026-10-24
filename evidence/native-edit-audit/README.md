# User-edit audit — 2026-10-23

Model: Qwen3.6-MoE-Combined; identity is recorded in every target report.

Design: docs/NATIVE_TUNING_PLAN.md, user-edit audit preregistration. Keep strict NRMSE <= 1e-5 and max absolute error <= 1e-3, full state checks, immutable parents and exact frozen greedy tokens. Do not promote an optimization merely because argmax stays unchanged.

Commands (run sequentially, canonical repository root):

```sh
cargo build --release --bin rust-server --example native_group_qualification --example native_mtp_decode_smoke
./target/release/examples/native_group_qualification evidence/native-edit-audit/repaired 40 fused off compiled-decode fused off
./target/release/examples/native_group_qualification evidence/native-edit-audit/attention-reference 40 fused off compiled-decode reference off
./target/release/examples/native_mtp_decode_smoke evidence/native-edit-audit/mtp
python3 judges/edit_audit_report.py evidence/native-edit-audit/attention-reference/report.json
python3 -m unittest discover -s judges -p test_edit_audit_report.py
```

`initial` retains the pre-repair exploratory candidate. `repaired` restores checks and truthful execution controls but enables fused attention; 63/63 numerical observations fail. `attention-reference` changes only the attention selection and passes 63/63 plus 256 frozen tokens. The attention selection is thus an isolated numerical-sensitivity source in this configuration. No claim is made about the precise internal SDPA mechanism causing the difference.

Target smoke scope: four prompts, one native lane, 64 generated tokens each, 63 timed post-first decode steps each. Rates are pooled by total steps / total seconds; TTFT is the arithmetic mean over four prompts. No p95, HTTP transport, concurrent-stream, continuous-batch or learned-acceptance claim follows from this runner. Initial binary was not hashed before replacement. Hashes in qualification.json identify the rebuilt audit binaries, with the intervening source distinction stated there.

Adversarial audit repaired: removed state validation; timing-dependent evaluation boundaries; MTP CLI/metadata disagreement; missing cache bounds/accounting; non-independent cached-constant snapshot baseline. Native context expansion remains experimental; scheduler and IR still admit at most 512 tokens. Fused attention remains opt-in and unqualified. Operator-stage profiling remains a serialized diagnostic and cannot support production throughput claims.

## MTP disposition

Compiled-default MTP was terminated after approximately four minutes without a completed report (exit 143). The stack sample is retained as `mtp-compiled-stall.sample.txt`; it locates the worker in hidden-step final MLX evaluation but does not establish the root cause. `open_mtp` now defaults to eager; `open_mtp_configured` retains explicit compilation selection.

The rebuilt eager run `mtp-eager/summary.json` passed widths 1/2/4/8, immutable-parent and committed-state checks. Learned acceptance was 32/32 on one counting prompt, four transactions per width. Width 1 proposed no learned tokens. This is not a representative acceptance benchmark, performance result or serving qualification. Debug replay/reference comparison was not enabled for this live smoke; the general debug-validation regression suite passed separately.

## Serialized operator attribution

```sh
./target/release/examples/native_group_qualification evidence/native-edit-audit/operator-stages 40 fused off eager reference stages
python3 judges/edit_audit_report.py evidence/native-edit-audit/operator-stages/report.json
```

All 63 numerical comparisons and 256 frozen tokens pass. The offline reader checks all 19 exact stage multiplicities for prefill and the 252 decode forwards, positive construction/wait intervals, unchanged reference tokens and zero residents at drain. Mutation tests reject missing counts and timing intervals. The largest isolated stage is MoE up/gate projection, followed by down projection. Forced per-stage evaluation and completion change execution; these waits are diagnostic, cannot be added to predict normal latency and are not production throughput claims.

## Physical-batch regression

```sh
cargo build --release --example native_physical_batch
./target/release/examples/native_physical_batch evidence/native-edit-audit/physical 40 fused compiled-decode
```

All 96 rows across B1/B2/B3 pass full-logit/state and token comparisons, with zero residents/bytes at drain. This gate does not establish concurrent HTTP performance. Final regression logs are retained in `logs/`: 193 default + 193 debug-validation Rust tests, 26 TypeScript tests, 41 offline tests, TS 7.1/tsrs, Clippy, ABI and operator timing sanitizers. The final source delta after full debug regression only restores the default MTP opener to eager; that opener was then exercised by the passing real-model eager smoke, library tests and all-target Clippy.
