---
title: "Native throughput and DSpark/TurboQuant completion gates"
document_type: "architecture"
version: "1.0.0"
date: "2026-10-25T01:53:09Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: "Cargo.toml"
    lines: "1-115"
    role: "config"
  - path: "README.md"
    lines: "1-1517"
    role: "supporting"
  - path: "build.rs"
    lines: "1-69"
    role: "config"
  - path: "docs/ADVERSARIAL_REVIEW.md"
    lines: "1-941"
    role: "supporting"
  - path: "docs/COMMAND_FLAGS.md"
    lines: "1-222"
    role: "supporting"
  - path: "docs/THROUGHPUT_COMPLETION.md"
    lines: "1-802"
    role: "supporting"
  - path: "evidence/throughput-batched-attention-gate/partial.json"
    lines: "1-1325"
    role: "test"
  - path: "evidence/throughput-batched-attention-gate/report.json"
    lines: "1-2494"
    role: "test"
  - path: "evidence/throughput-batched-candidate-width4/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-batched-candidate-width4/summary.json"
    lines: "1-855"
    role: "test"
  - path: "evidence/throughput-clean-batched4/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-clean-batched4/summary.json"
    lines: "1-855"
    role: "test"
  - path: "evidence/throughput-clean-old4/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-clean-old4/summary.json"
    lines: "1-854"
    role: "test"
  - path: "evidence/throughput-clean-reference4/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-clean-reference4/summary.json"
    lines: "1-855"
    role: "test"
  - path: "evidence/throughput-clean-reference8/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-clean-reference8/summary.json"
    lines: "1-855"
    role: "test"
  - path: "evidence/throughput-completion/attention-oracle.json"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/attention-oracle.stderr"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/baseline-binary.sha256"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/baseline-identity.json"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/candidate-identity.json"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/candidate-initial-binary.sha256"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/clippy.log"
    lines: "1-29"
    role: "test"
  - path: "evidence/throughput-completion/completion-status.json"
    lines: "1-113"
    role: "test"
  - path: "evidence/throughput-completion/cpp-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/cpp-final-accounting.json"
    lines: "1-16"
    role: "test"
  - path: "evidence/throughput-completion/cpp-final-identity.json"
    lines: "1-9"
    role: "test"
  - path: "evidence/throughput-completion/cpp-final-result.json"
    lines: "1-19"
    role: "test"
  - path: "evidence/throughput-completion/cpp-final-run.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/cpp-identity.json"
    lines: "1-9"
    role: "test"
  - path: "evidence/throughput-completion/cpp-independent-accounting.json"
    lines: "1-16"
    role: "test"
  - path: "evidence/throughput-completion/cpp-linkage.txt"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/cpp-plan-adversarial.json"
    lines: "1-50"
    role: "test"
  - path: "evidence/throughput-completion/cpp-plan-export.log"
    lines: "1-30"
    role: "test"
  - path: "evidence/throughput-completion/cpp-plan.txt"
    lines: "1-112"
    role: "test"
  - path: "evidence/throughput-completion/cpp-result.json"
    lines: "1-19"
    role: "test"
  - path: "evidence/throughput-completion/cpp-run.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/cpp-sanitized-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/cpp-sanitized-tests.log"
    lines: "1-5"
    role: "test"
  - path: "evidence/throughput-completion/device_abi-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/device_abi.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/dspark-artifact.json"
    lines: "1-27"
    role: "test"
  - path: "evidence/throughput-completion/dspark-ingestion.log"
    lines: "1-30"
    role: "test"
  - path: "evidence/throughput-completion/final-build.log"
    lines: "1-29"
    role: "test"
  - path: "evidence/throughput-completion/final-clippy.log"
    lines: "1-29"
    role: "test"
  - path: "evidence/throughput-completion/final-format.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/final-http.log"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/final-rust-debug.log"
    lines: "1-876"
    role: "test"
  - path: "evidence/throughput-completion/final-rust-default.log"
    lines: "1-876"
    role: "test"
  - path: "evidence/throughput-completion/http-independent-accounting.json"
    lines: "1-33"
    role: "test"
  - path: "evidence/throughput-completion/local-reference-source-hashes.json"
    lines: "1-6"
    role: "test"
  - path: "evidence/throughput-completion/offline.log"
    lines: "1-16"
    role: "test"
  - path: "evidence/throughput-completion/operator_reference-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/operator_reference.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/projection_reference-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/projection_reference.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/rust-debug.log"
    lines: "1-866"
    role: "test"
  - path: "evidence/throughput-completion/rust-default.log"
    lines: "1-866"
    role: "test"
  - path: "evidence/throughput-completion/rust-minimal.log"
    lines: "1-250"
    role: "test"
  - path: "evidence/throughput-completion/server-identity.json"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/shipping-binary.json"
    lines: "1-5"
    role: "test"
  - path: "evidence/throughput-completion/shipping-build.log"
    lines: "1-29"
    role: "test"
  - path: "evidence/throughput-completion/shipping-clippy.log"
    lines: "1-29"
    role: "test"
  - path: "evidence/throughput-completion/shipping-format.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/shipping-offline.log"
    lines: "1-16"
    role: "test"
  - path: "evidence/throughput-completion/shipping-rust-debug.log"
    lines: "1-882"
    role: "test"
  - path: "evidence/throughput-completion/shipping-rust-default.log"
    lines: "1-882"
    role: "test"
  - path: "evidence/throughput-completion/state_table-build.log"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-completion/state_table.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/throughput-clean-batched4.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/throughput-clean-old4.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/throughput-clean-reference4.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/throughput-clean-reference8.log"
    lines: "1-1"
    role: "test"
  - path: "evidence/throughput-completion/ts71.log"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/tsrs.log"
    lines: "1-4"
    role: "test"
  - path: "evidence/throughput-completion/typescript-tests.log"
    lines: "1-41"
    role: "test"
  - path: "evidence/throughput-completion/verification-summary.json"
    lines: "1-20"
    role: "test"
  - path: "evidence/throughput-final-http-reference/archive-verification.log"
    lines: "1-82"
    role: "test"
  - path: "evidence/throughput-final-http-reference/benchmark.json"
    lines: "1-40"
    role: "test"
  - path: "evidence/throughput-final-http-reference/host.txt"
    lines: "1-571"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/acceptance.json"
    lines: "1-13"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/command.json"
    lines: "1-43"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/lanes-1-partial.json"
    lines: "1-3434"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/lanes-1.json"
    lines: "1-3473"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/lanes-3-partial.json"
    lines: "1-3434"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/lanes-3.json"
    lines: "1-3473"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/open-attempts.json"
    lines: "1-1090"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/open-loop.json"
    lines: "1-1144"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request--1.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10000.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10001.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10002.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10003.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10004.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10005.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10006.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10007.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10008.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10009.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10010.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10011.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10012.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10013.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10014.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-10015.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30000.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30001.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30002.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30003.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30004.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30005.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30006.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30007.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30008.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30009.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30010.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30011.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30012.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30013.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30014.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-30015.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90000.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90001.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90002.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90003.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90004.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90005.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90006.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90007.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90008.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90009.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90010.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90011.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90012.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90013.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90014.amst"
    lines: "N/A (binary stream archive)"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/request-90015.amst"
    lines: "1-0"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/server.log"
    lines: "1-9"
    role: "test"
  - path: "evidence/throughput-final-http-reference/mtp/warmup.json"
    lines: "1-51"
    role: "test"
  - path: "evidence/throughput-final-http-reference/partial.json"
    lines: "1-6948"
    role: "test"
  - path: "evidence/throughput-final-http-reference/summary.json"
    lines: "1-8114"
    role: "test"
  - path: "evidence/throughput-fused-pressure-gate/progress.jsonl"
    lines: "1-88"
    role: "test"
  - path: "evidence/throughput-fused-pressure-gate/summary.json"
    lines: "1-114"
    role: "test"
  - path: "evidence/throughput-fused-state-gate/progress.jsonl"
    lines: "1-88"
    role: "test"
  - path: "evidence/throughput-fused-state-gate/summary.json"
    lines: "1-111"
    role: "test"
  - path: "evidence/throughput-mtp-component/summary.json"
    lines: "1-73"
    role: "test"
  - path: "evidence/throughput-resume-baseline/partial.json"
    lines: "1-834"
    role: "test"
  - path: "evidence/throughput-resume-baseline/summary.json"
    lines: "1-854"
    role: "test"
  - path: "examples/native_cpp_plan.rs"
    lines: "1-85"
    role: "test"
  - path: "examples/native_device_pressure.rs"
    lines: "1-214"
    role: "test"
  - path: "examples/native_device_reference.rs"
    lines: "1-218"
    role: "test"
  - path: "examples/native_dspark_inventory.rs"
    lines: "1-33"
    role: "test"
  - path: "examples/native_group_qualification.rs"
    lines: "1-271"
    role: "test"
  - path: "examples/native_mtp_bench.rs"
    lines: "1-223"
    role: "test"
  - path: "examples/native_mtp_decode_smoke.rs"
    lines: "1-286"
    role: "test"
  - path: "examples/native_physical_batch.rs"
    lines: "1-261"
    role: "test"
  - path: "fixtures/koopah-dspark-config.json"
    lines: "1-63"
    role: "config"
  - path: "judges/batched_reference_attention.cpp"
    lines: "1-37"
    role: "test"
  - path: "judges/native_cpp_bench.cpp"
    lines: "1-108"
    role: "test"
  - path: "judges/physical_throughput_verify.py"
    lines: "1-98"
    role: "test"
  - path: "judges/test_native_cpp_plan.py"
    lines: "1-64"
    role: "test"
  - path: "native/batched_reference_attention.h"
    lines: "1-21"
    role: "primary"
  - path: "native/mlx_bridge.h"
    lines: "1-137"
    role: "primary"
  - path: "native/qwen_device.inc"
    lines: "1-73"
    role: "primary"
  - path: "native/qwen_model.inc"
    lines: "1-390"
    role: "primary"
  - path: "native/qwen_mtp.inc"
    lines: "1-313"
    role: "primary"
  - path: "scripts/build_native_cpp_bench.sh"
    lines: "1-9"
    role: "supporting"
  - path: "src/cache/meta_automaton.rs"
    lines: "1-536"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-472"
    role: "primary"
  - path: "src/serving/mod.rs"
    lines: "1-121"
    role: "primary"
  - path: "src/serving/native_device.rs"
    lines: "1-1486"
    role: "primary"
  - path: "src/serving/native_dspark.rs"
    lines: "1-312"
    role: "primary"
  - path: "src/serving/native_http.rs"
    lines: "1-755"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-511"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-715"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-761"
    role: "primary"
  - path: "ts/http-bench-config.test.ts"
    lines: "1-62"
    role: "test"
  - path: "ts/http-bench-config.ts"
    lines: "1-37"
    role: "primary"
  - path: "ts/http-bench.ts"
    lines: "1-55"
    role: "primary"
related_issues: []
related_documents: ["NATIVE_TUNING_PLAN.md", "MTP_PHASE_COSTS.md", "COMPILED_WEIGHT_SHARING.md"]
tags: [native-mlx, throughput, speculative-decoding, dspark, turboquant]
---
# Executive summary

Continue from the local jet-engine branch, preserving the Combined checkpoint and exact greedy output contract. The user clarified the delivery gate on 2026-10-24: MTP + speculative decoding + TurboQuant must achieve at least 125 tok/s while retaining acceptance. Use matched end-to-end HTTP measurements, reporting single-stream and three-stream aggregate separately; a native API smoke is insufficient. Keep the original doubling requirement visible alongside this absolute minimum. DSpark and TurboQuant require actual device data paths and independent qualification.

# Problem statement

Main predates the shared-weight repair. The newer branch adds a fused transaction but bypasses full draft-state finite checks and duplicates verifier logits on the host. Its descriptor coordinator does not compress or execute attention over KV payloads. The cached six-layer Koopah drafter must not be confused with a five-layer generic descriptor.

# Solution overview

First reproduce the current branch. Harden fused transactions and evaluate batched FP32 reference attention to reduce multi-token verification dispatch without switching to the previously failing fused SDPA. Keep snapshot reference arithmetic untouched, preserve all learned proposals/margin policy and full validation, and measure acceptance with known seed tokens excluded.

# Implementation details

Candidate attention computes all query rows with explicit causal masking and FP32 elementwise products/reductions, retaining the reference reduction axes. Immutable parents, all 80 state arrays, completion-before-publication and bounded residency remain mandatory. Failed candidates stay unpromoted. Fused transaction results must retire both new handles on publication failure and validate target/draft payloads under the same selected validation posture. The controller path passes its already-bound immutable proposal tokens to a native prepared-transaction entry point; it never drafts again after binding. Attempted target calls are returned even on failure so the existing atomic counter and completion-owned admission ledger remain accountable. Device row selection preserves full rejected-suffix finite validation while exporting only row IDs and final-row logits.

# Validation and adversarial review

Require unchanged frozen tokens, full logits/state NRMSE <=1e-5 and max absolute <=1e-3, widths 1/2/4/8, rejection/replay and immutable parent checks, cancellation, pressure, context and zero drain. Run Rust default/debug/minimal suites, TypeScript and offline readers, native judges and Clippy. Only then compare alternate-order HTTP 1/3-lane windows with TTFT/ITL, memory, token congruence and learned acceptance. Preserve raw commands, model and binary hashes. Never count descriptor byte estimates as compression evidence.

# Remaining integration work

DSpark requires eight target-layer taps, matching native six-layer backbone/Markov/confidence math and a separately identified proposer. TurboQuant requires actual typed K/V payload encoding and attention consumption; recurrent/convolution state stays separate and lossy attention must pass declared quality gates. No seamless integration or doubled-throughput claim follows from flags, bookkeeping, or unit tests.

## User-selected Koopah artifact

Use the cached `Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK` snapshot
`815f706fe138402c9e446b2e2787da23ff7ba9e7`. The local path is
`/Users/stewartbarteau/.cache/huggingface/hub/models--Koopah--Qwen3.6-35B-A3B-NVFP4-DSPARK/snapshots/815f706fe138402c9e446b2e2787da23ff7ba9e7`.
The user explicitly selected this artifact on 2026-10-24. This selection does not
establish compatibility with the Combined target's trained representation.

Its config specifies six layers, hidden size 2048, MLP size 6144, 32 query heads,
eight KV heads, head dimension 128, checkpoint block field eight, vocabulary 248320,
RMS epsilon 1e-6 and RoPE theta 10000000. Required target-layer identifiers are
`[1,6,11,16,22,27,32,37]`; establish the reference wrapper's indexing convention
before wiring target taps. Use the checkpoint's own BF16 embedding and output
weights, rank-256 Markov correction and confidence head with Markov inputs.
Do not substitute target quantized embeddings or the single-layer MTP sidecar.
The installed reference normalizes checkpoint block eight into eight proposals
plus the anchor (verification width nine), defaults runtime width to eight,
uses `sample_from_anchor=true`, and chooses adaptive block policy for this
architecture. These semantics must be explicit; the raw block field is not
the verifier width. The current native target ABI accepts at most eight input
rows per call, so width nine cannot be silently passed through it.

Before admission, bind target identity, complete draft weight/config identity,
tap convention, arithmetic policy and cache codec/version into proposer identity.
The context cache must advance only with committed target tokens; speculative
proposal KV must remain temporary. Cancellation, rejection and replay must retain
immutable parent states and retire every temporary allocation. Markov proposal
generation and confidence decisions require their own offline reference gate.
Only the exact target verifier can commit accepted text.

TurboQuant must expose real encoded K/V storage and decode/attention consumption,
including codec metadata, scales, error bounds and ownership accounting. Report
physical payload bytes separately from logical admission charges. Recurrent
state is outside the codec. Qualify the combined configuration at contexts that
actually activate compression; a short context entirely in a protected tail is
insufficient evidence. Keep lossy-quality qualification separate from exact
target-versus-speculative congruence under the same cache configuration.

# Risks and rollback

Batched reductions can alter floating-point results and increase transient memory. Preserve the snapshot reference and fail the existing gate on drift; do not widen tolerances. Native API and short-context performance cannot establish long-context or production qualification.

# Diagnostic execution boundary

The `mtp-phase-timing` feature retains the separate Rust prepare/verify/refresh path for phase attribution. Default builds use the native fused transaction; a timing-feature run is not a timing-only intervention on that path. Benchmark reports identify this execution difference explicitly. Compare default binaries for throughput qualification.

## Two kernel proof routes

The user identified a separate C++ fork after removing TS/Tokio. Compare that fork with this repository's native C++ kernel path using the same checkpoint, prompt IDs, output budget, cache configuration and acceptance denominator. Identify the actual fork and commit before interpreting its measurements. Direct C++ timings and Rust API timings can isolate wrapper overhead; HTTP measurements separately qualify serving. A faster wrapper cannot establish a kernel improvement.

## Standalone C++ control protocol

Add a direct C++ executable over the existing native ABI to isolate the kernel
from Rust/Tokio scheduling without claiming it is the user's separate fork.
An offline Rust exporter validates the target manifest, MTP sidecar and frozen
reference identity, tokenizes the same four prompts, and emits a bounded quoted
text plan containing shard paths, quantization overrides, EOS and exact input/
reference IDs. No Rust runtime, TS, Python model worker or HTTP server executes
inside the C++ measurement process. Native session geometry and full validation
match eager/group40/width4 Rust controls. Warm both paths, alternate target/MTP
order over two rounds, verify every output ID, release every state, and require
zero residents/bytes before reporting success. Report pooled decode throughput,
acceptance excluding known seeds, native API scope and lack of DSpark/TurboQuant
execution explicitly. This isolates wrapper costs; the separate C++ fork still
requires its actual source/commit before a kernel comparison is possible.

# Measured result and necessity

The 125 tok/s combined gate is **not met**. The direct C++ control measured
52.56 and 52.74 warmed MTP decode tok/s in two runs, with 166/172 learned
proposals accepted (96.51%) and 512 frozen output tokens checked per run.
Its target-only control measured 23.89 and 23.91 tok/s. It executes the existing
kernel, not the unidentified separate fork. These short native calls exclude
model loading, tokenization and HTTP; they do not qualify the combined path.

The final current-path HTTP control delivered 2,000 exact frozen closed-loop
tokens: 35.18 end-to-end tok/s on one stream and 34.37 aggregate over three
streams. Its cumulative learned acceptance was 1486/1540 (96.49%), excluding
known seeds. Open arrivals produced eight successful requests and eight bounded
503 refusals; independent archive/ledger accounting and zero drain passed.

The isolated Rust API controls measured 48.07 MTP tok/s with reference attention,
42.73 with batched reference attention, and 48.63 with the preserved pre-change
binary. Width eight measured 52.71 tok/s but acceptance fell to 178/200 (89%).
These runs are sequential observations on a shared host, not confidence intervals
or a randomized promotion study. Preserve width four/reference defaults. Neither
wider proposals nor this batched-attention implementation demonstrates the
required improvement. Full fused-state validation and publication rollback are
correctness repairs, regardless of their performance cost.

Raw receipt index: `evidence/throughput-completion/completion-status.json`.
The direct C++ report is `cpp-final-result.json`; its independent accounting and
binary/source/plan hashes are adjacent. HTTP archives are in
`evidence/throughput-final-http-reference/`. Drafter ingestion produced full
weight SHA-256 `ab0beb963036e558c7378b55e7188c800455ef3a493f3d9edd3f6b9a0ab4e0e2`;
this establishes the inspected bytes, not native model compatibility.

# Languages and configuration

| Language | Component | Reason |
| --- | --- | --- |
| C++20 | Native transaction, attention experiment, direct benchmark | Operate on existing MLX device arrays and isolate kernel cost. |
| Rust | Executor ownership, controller binding, strict drafter ingestion, offline plan export | Preserve admission lifetimes and checked model metadata. |
| TypeScript | Existing HTTP driver and selector validation | Carry an explicit attention mode into archived stream receipts. |
| Python | Offline receipt accounting and adversarial CLI checks | Validate artifacts independently; no model worker or serving transport. |
| POSIX shell | Direct C++ build entry point | Reproduce an executable without a Rust runtime. |

This implementation was not directly grounded in published literature. It applies
existing reference arithmetic, transactional state ownership and strict ingestion
checks. The local DSpark implementation/config and TurboQuant source are recorded
by path/hash in `local-reference-source-hashes.json`; their contents are inspection
references, not native execution claims. The only new inference selector is the
opt-in bounded `batched-reference` attention mode; production defaults remain
reference attention, width four and full validation. Diagnostic phase-timing
builds use the separately labelled Rust path.

# Testing evidence and reproduction

The full Rust default and debug-validation suites, minimal-feature tests,
TypeScript tests/type checks, offline archive readers, C++ ABI/operator/state
judges, component/state parity, pressure and cancellation gates are archived in
`evidence/throughput-completion/`. The new drafter validator checks malformed
geometry, missing/extra tensors, ranges, duplicate JSON keys and incompatible
config/tap/anchor semantics. The direct C++ CLI tests malformed counts, paths,
vocabulary, trailing data, oversized plans and preservation of existing reports.
Sanitizer coverage of the bridge does not instrument the upstream MLX dylib.

Reproduce core gates from the repository root:

```sh
cargo test --locked --release --all-targets
cargo test --locked --release --all-targets --features debug-validation
cargo test --locked --release --no-default-features --lib --tests
cargo clippy --locked --all-targets --features debug-validation,mtp-phase-timing -- -D warnings
npm test
npm run typecheck
npm run typecheck:ts71
python3 -m unittest discover -s judges -p 'test_*.py'
cargo run --locked --release --example native_cpp_plan -- NEW_PLAN.txt
scripts/build_native_cpp_bench.sh /tmp/native-cpp-control
/tmp/native-cpp-control NEW_PLAN.txt NEW_REPORT.json
NATIVE_CPP_BENCH=/tmp/native-cpp-control NATIVE_CPP_PLAN=NEW_PLAN.txt python3 -m unittest discover -s judges -p 'test_native_cpp_plan.py'
```

HTTP command and binary identity are preserved in each run's `command.json` and
`benchmark.json`. Do not run competing model or compilation workloads during
performance measurements. The optional CLI integration tests require the two
explicit environment variables above; ordinary offline discovery skips them.

# Deployment and rollout

No deployment, merge or production promotion is performed. The work is local on
`codex/throughput-dspark-turboquant`; document status remains draft. The opt-in
attention experiment can be disabled with `--attention-kernel reference`.
Keep the captured old binary for bounded comparisons; it retains the inherited
fused-validation defects and is not a correctness rollback recommendation.

# Next stage and learnings

This implementation sets up reproducible kernel-versus-wrapper comparisons and
checked drafter ingestion, but does not complete the 125 tok/s combined goal.
Locate the actual C++ fork and commit, inspect its design and kernels, and run the
same bounded frozen plan there. Next qualify actual target taps and native
DSpark backbone/Markov/confidence output, then real TurboQuant payload and
attention execution, before combined acceptance/quality/performance promotion.

The native C++ result remains near the Rust API result, so removing the wrapper
alone does not demonstrate the required throughput. Proposal width cannot be
promoted independently of acceptance. Raw checkpoint block fields can differ
from normalized verification widths; cached filenames and descriptor labels
cannot establish execution compatibility or actual compression.

# Changelog

- 2026-10-25: Added fused transaction hardening, prepared controller commit,
  bounded attention experiment, strict Koopah ingestion, independent C++
  control, adversarial checks and unqualified completion scorecard.

# Files modified with line references

These spans identify current source/config/documentation files. Raw test artifacts are individually indexed in metadata; binary archives have no source-line span.

| File | Current line span | Role |
| --- | --- | --- |
| `Cargo.toml` | 1-115 | config |
| `README.md` | 1-1515 | supporting |
| `build.rs` | 1-69 | config |
| `docs/ADVERSARIAL_REVIEW.md` | 1-941 | supporting |
| `docs/COMMAND_FLAGS.md` | 1-222 | supporting |
| `docs/THROUGHPUT_COMPLETION.md` | 1-802 | supporting |
| `examples/native_cpp_plan.rs` | 1-85 | test |
| `examples/native_device_pressure.rs` | 1-214 | test |
| `examples/native_device_reference.rs` | 1-218 | test |
| `examples/native_dspark_inventory.rs` | 1-33 | test |
| `examples/native_group_qualification.rs` | 1-271 | test |
| `examples/native_mtp_bench.rs` | 1-223 | test |
| `examples/native_mtp_decode_smoke.rs` | 1-286 | test |
| `examples/native_physical_batch.rs` | 1-261 | test |
| `fixtures/koopah-dspark-config.json` | 1-63 | config |
| `judges/batched_reference_attention.cpp` | 1-37 | test |
| `judges/native_cpp_bench.cpp` | 1-108 | test |
| `judges/physical_throughput_verify.py` | 1-98 | test |
| `judges/test_native_cpp_plan.py` | 1-64 | test |
| `native/batched_reference_attention.h` | 1-21 | primary |
| `native/mlx_bridge.h` | 1-137 | primary |
| `native/qwen_device.inc` | 1-73 | primary |
| `native/qwen_model.inc` | 1-390 | primary |
| `native/qwen_mtp.inc` | 1-313 | primary |
| `scripts/build_native_cpp_bench.sh` | 1-9 | supporting |
| `src/cache/meta_automaton.rs` | 1-536 | primary |
| `src/main.rs` | 1-472 | primary |
| `src/serving/mod.rs` | 1-121 | primary |
| `src/serving/native_device.rs` | 1-1486 | primary |
| `src/serving/native_dspark.rs` | 1-312 | primary |
| `src/serving/native_http.rs` | 1-755 | primary |
| `src/serving/native_mtp_decode.rs` | 1-511 | primary |
| `src/serving/native_mtp_device.rs` | 1-715 | primary |
| `src/serving/native_qwen.rs` | 1-761 | primary |
| `ts/http-bench-config.test.ts` | 1-62 | test |
| `ts/http-bench-config.ts` | 1-37 | primary |
| `ts/http-bench.ts` | 1-55 | primary |

## Preservation checkpoint —2026-10-25

This original checkout is preserved separately on `codex/throughput-dspark-turboquant`, including its local experimental source and evidence. The later user-selected328a6bb C++/DSpark worktree is published on `codex/cpp-dspark-kernel-proof`; use that branch's root handoff.md to continue. These are separate experimental snapshots, not merged implementations or production promotions. Earlier working-tree/measurement descriptions above are historical captures.
