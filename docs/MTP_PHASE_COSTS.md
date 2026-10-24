---
title: "Native MTP phase costs and packed-projection discriminator"
document_type: "research"
version: "1.0.0"
date: "2026-10-24"
status: "experimental"
---

# Native MTP phase costs

**Decision: retain the existing eager/reference-expert production defaults. No throughput improvement or150 tok/s qualification is established.** The opt-in `mtp-phase-timing` Cargo feature measures existing awaited Rust operations without changing model arithmetic. Disabled builds contain no phase clocks/receipt fields. Design and controls were preregistered in [NATIVE_TUNING_PLAN.md](NATIVE_TUNING_PLAN.md); controls are indexed in [COMMAND_FLAGS.md](COMMAND_FLAGS.md#rust-boundary-mtp-phase-diagnostics).

Exact workhorse: `~/models/fused/Qwen3.6-MoE-Combined`, manifest identity `509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b`. Native Rust/C++ MLX GPU execution, FP32 activations, full finite validation, eager/group40/fused recurrence/reference attention, width4 MTP, regular fixed-capacity KV. The stock MLX numerical path remains distinct from integer-only custom Metal kernels. No Python model worker or JSON model transport.

## Measurements and limits

Two alternating rounds across four frozen prompts,32 generated tokens per row,256 target-only and256 MTP tokens per run. Loading, warmup, tokenization and HTTP are excluded. These are warmed scalar Rust API measurements, not agent quality, continuous tensor batching or end-to-end serving qualification.

| Run | Feature | Expert | MTP decode tok/s | Peak MLX bytes |
|---|---|---|---:|---:|
| Original control | off | reference |49.5154| not recorded |
| Initial timed | on | reference |47.4520| not recorded |
| Clean reference repeat | on | reference |48.0418|20,654,647,232|
| Packed up/gate candidate | on | fused |47.6250|33,211,056,360|

All runs preserve exact frozen outputs and166/172 learned acceptance (**96.51%**, excluding known target seeds). The clean instrumented pair shows the candidate taking0.8753% longer for decode, with60.79% more peak MLX memory. That is insufficient evidence of a benefit; no matching HTTP promotion was attempted. This is one ordered shared-host comparison, not a statistical backend ranking. The initial timed run overlapped CPU CI near its tail and is retained as a confounded observation. Clean reference elapsed decode is3.0672% above the earlier untimed control; time order/host drift prevent attributing this solely to instrumentation overhead.

The clean reference repeat partitions successful prepare+commit envelopes as follows:

| Awaited phase | Time ns | Fraction |
|---|---:|---:|
| Autoregressive draft calls |388,017,833|7.28%|
| Target block verification |3,774,282,460|70.86%|
| Rejected-window target replay |169,151,416|3.18%|
| Committed draft-state refresh |340,064,916|6.38%|
| Host/unclassified envelope residue |655,234,632|12.30%|

Total transaction envelope5,326,751,257ns; outer decode5,328,687,625ns. Prefill is separate:3,685,952,041ns (target3,408,215,541; draft269,505,916; residue8,230,584).256 committed tokens /90 counted target calls =2.8444;246 target-window tokens reused and10 replayed. These are successful-call counters, not complete GPU dispatch counts.

**Await spans include native graph construction, queueing, execution, completion and exports; they are not GPU timestamps.** Removing the current draft-call cost alone cannot close the gap to150. A better trusted proposer could still help by widening accepted blocks and amortizing verification, so this result does not rule out proposer improvements. The immediate discriminator is the multi-token verifier's real dispatch/dataflow cost, followed by state-safe wider verification.

## Qualification and adversarial checks

The fused candidate passed all1–8-token block versus serial logit/hidden/full-state tolerances, immutable-parent/branch tests, cancellation, bounded allocation/pressure, the full512-token native context and zero-state drain. This establishes component equivalence within the existing tolerance, not a speed win or broad model qualification.

`judges/mtp_phase_report.py` independently reconciles every receipt, emitted token, learned proposal/acceptance, target calls/forward tokens, accepted-window reuse and rejection replay. It rejects dropped receipts, counter corruption, negative/NaN/bool/oversized durations, phase/envelope overlap, configuration/instrumentation mismatches, and recorded workloads as throughput controls. Frozen token checks and>=90% learned acceptance remain mandatory. New example arguments reject invalid values before output creation/model loading. Production configuration, default validation and proposed sampling rates remain unchanged.

## Reproduction

Run from the repository root; every output directory must be new. Preserve each binary before rebuilding another feature set.

```sh
cargo build --release --example native_mtp_bench
cp target/release/examples/native_mtp_bench artifacts/mtp-phase-control
artifacts/mtp-phase-control NEW_CONTROL 4 eager 40 reference
cargo build --release --features mtp-phase-timing --example native_mtp_bench
cp target/release/examples/native_mtp_bench artifacts/mtp-phase-candidate
artifacts/mtp-phase-candidate NEW_REFERENCE 4 eager 40 reference
artifacts/mtp-phase-candidate NEW_FUSED 4 eager 40 fused
cargo run --release --example native_mtp_smoke -- NEW_COMPONENT eager 40 fused fused
python3 -m unittest discover -s judges -p 'test_*.py'
```

Independent reader: `validate(control, timed, frozen)` checks instrumentation; `validate(reference_timed, fused_timed, frozen, candidate=True)` checks the packed candidate. Raw reports and independent reconciliations are in [evidence/mtp-phase-costs](../evidence/mtp-phase-costs/). These offline Python readers do not execute model inference.

## Next gates

1. Obtain valid warmed GPU dispatch tables before claiming an individual kernel bottleneck. Preserve failed traces and bound recorder lifetimes. Captured workloads cannot qualify throughput.
2. Optimize measured multi-token verification costs under unchanged frozen-output/state gates. Keep the losing packed projection optional; do not promote it.
3. Only after a native discriminator wins, rerun identical1/3-stream HTTP cells and open arrivals,1,000/1,500/2,000-token windows, TTFT/ITL, accepted/proposed counts and zero-drain accounting.
4. Wider proposers/MTP or DFlash-style plugins need measured net benefit under the same automaton admission/rollback discipline. Paging/radix and prefill disaggregation remain separate gated work, not supported speed claims.

The last verified HTTP figures remain37.05 single-stream /37.00 aggregate three-stream generated tok/s,96.42% cumulative learned acceptance; see [COMPILED_WEIGHT_SHARING.md](COMPILED_WEIGHT_SHARING.md). This run adds attribution and rejects a candidate; it does not change those figures.
