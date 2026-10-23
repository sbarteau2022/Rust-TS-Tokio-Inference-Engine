---
title: "Implementation file map and evidence index"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-23T06:10:12Z"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files:
  - path: ".gitignore"
    lines: "1-9"
    role: "config"
  - path: "Cargo.lock"
    lines: "1-1015"
    role: "config"
  - path: "Cargo.toml"
    lines: "1-73"
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
    lines: "1-218"
    role: "primary"
  - path: "examples/native_mtp_bench.rs"
    lines: "1-130"
    role: "primary"
  - path: "examples/native_mtp_decode_smoke.rs"
    lines: "1-160"
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
  - path: "judges/layer_reference.py"
    lines: "1-185"
    role: "test"
  - path: "judges/macro.py"
    lines: "1-14"
    role: "test"
  - path: "judges/model_ab.py"
    lines: "1-93"
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
    lines: "1-230"
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
    lines: "1-30"
    role: "primary"
  - path: "src/macro_bench.rs"
    lines: "1-81"
    role: "primary"
  - path: "src/main.rs"
    lines: "1-453"
    role: "primary"
  - path: "src/runtime.rs"
    lines: "1-366"
    role: "primary"
  - path: "src/sampled.rs"
    lines: "1-193"
    role: "primary"
  - path: "src/serving/choices.rs"
    lines: "1-123"
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
    lines: "1-107"
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
    lines: "1-98"
    role: "primary"
  - path: "src/serving/native_mtp_decode.rs"
    lines: "1-149"
    role: "primary"
  - path: "src/serving/native_mtp_device.rs"
    lines: "1-436"
    role: "primary"
  - path: "src/serving/native_ops.rs"
    lines: "1-273"
    role: "primary"
  - path: "src/serving/native_qwen.rs"
    lines: "1-690"
    role: "primary"
  - path: "src/serving/native_reference.rs"
    lines: "1-100"
    role: "primary"
  - path: "src/serving/native_speculative.rs"
    lines: "1-382"
    role: "primary"
  - path: "src/serving/native_state.rs"
    lines: "1-194"
    role: "primary"
  - path: "src/serving/native_tokenizer.rs"
    lines: "1-213"
    role: "primary"
  - path: "src/serving/qwen_tests.rs"
    lines: "1-566"
    role: "primary"
  - path: "src/serving/rapid.rs"
    lines: "1-265"
    role: "primary"
  - path: "src/serving/scheduler.rs"
    lines: "1-645"
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
  - path: "tests/draft_admission.rs"
    lines: "1-475"
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
    lines: "1-435"
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
related_documents: ["../README.md", "DESIGN.md", "ADVERSARIAL_REVIEW.md", "NEXT_STEPS.md"]
tags: [documentation, provenance]
---
# File map

Exact text line references for this local implementation. Binary fixtures have no text lines; null is deliberate. Generated files are compiler outputs, not handwritten implementation. Documents are draft; no git merge or git-blame provenance is claimed for this new standalone directory.

| File | Exact lines | Role |
| --- | --- | --- |
| `.gitignore` | 1–9 | config |
| `Cargo.lock` | 1–1015 | config |
| `Cargo.toml` | 1–73 | config |
| `backend/rapid_worker.py` | 1–165 | primary |
| `build.rs` | 1–63 | primary |
| `examples/native_automaton_smoke.rs` | 1–394 | primary |
| `examples/native_device_pressure.rs` | 1–184 | primary |
| `examples/native_device_reference.rs` | 1–166 | primary |
| `examples/native_discriminators.rs` | 1–145 | primary |
| `examples/native_hidden_smoke.rs` | 1–170 | primary |
| `examples/native_mtp_automaton_smoke.rs` | 1–218 | primary |
| `examples/native_mtp_bench.rs` | 1–130 | primary |
| `examples/native_mtp_decode_smoke.rs` | 1–160 | primary |
| `examples/native_mtp_smoke.rs` | 1–186 | primary |
| `examples/native_speculative_smoke.rs` | 1–370 | primary |
| `examples/native_suite_reference.rs` | 1–198 | primary |
| `fixtures/atan2.amvk` | Binary / no text lines | test |
| `fixtures/cache-config-bench.json` | 1–13 | test |
| `fixtures/cos.amvk` | Binary / no text lines | test |
| `fixtures/elementwise.amvk` | Binary / no text lines | test |
| `fixtures/native-choice-heldout.tsv` | 1–4 | test |
| `fixtures/native-domain-heldout.tsv` | 1–24 | test |
| `fixtures/native-heldout.tsv` | 1–16 | test |
| `fixtures/native-interpretation-probes.tsv` | 1–4 | test |
| `fixtures/native-model.path` | 1–1 | test |
| `fixtures/native-speculative-prompts.tsv` | 1–4 | test |
| `fixtures/qwen-prompts-long.json` | 1–4 | test |
| `fixtures/qwen-prompts.json` | 1–6 | test |
| `fixtures/qwen-serving-long.json` | 1–21 | test |
| `fixtures/qwen-serving.json` | 1–21 | test |
| `fixtures/rectangular.amvk` | Binary / no text lines | test |
| `fixtures/sin.amvk` | Binary / no text lines | test |
| `fixtures/square.amvk` | Binary / no text lines | test |
| `fixtures/tanh.amvk` | Binary / no text lines | test |
| `judges/attention_identical_inputs.py` | 1–72 | test |
| `judges/attention_profile.py` | 1–44 | test |
| `judges/benchmark.py` | 1–51 | test |
| `judges/cordic.py` | 1–30 | test |
| `judges/dense_precision.cpp` | 1–21 | test |
| `judges/device_abi.cpp` | 1–49 | test |
| `judges/device_smoke_report.py` | 1–38 | test |
| `judges/generate_cordic.py` | 1–44 | test |
| `judges/layer_reference.py` | 1–185 | test |
| `judges/macro.py` | 1–14 | test |
| `judges/model_ab.py` | 1–93 | test |
| `judges/operator_reference.cpp` | 1–29 | test |
| `judges/projection_reference.cpp` | 1–53 | test |
| `judges/resources.py` | 1–35 | test |
| `judges/run.py` | 1–230 | test |
| `judges/speculative_report.py` | 1–74 | test |
| `judges/state_table.cpp` | 1–21 | test |
| `judges/suite_reference.py` | 1–248 | test |
| `judges/test_attention_profile.py` | 1–51 | test |
| `judges/test_device_smoke_report.py` | 1–26 | test |
| `judges/test_layer_reference.py` | 1–56 | test |
| `judges/test_speculative_report.py` | 1–34 | test |
| `judges/test_suite_reference.py` | 1–107 | test |
| `native/attention_reference.h` | 1–21 | primary |
| `native/mlx_bridge.cpp` | 1–178 | primary |
| `native/mlx_bridge.h` | 1–112 | primary |
| `native/operator_probe.inc` | 1–28 | primary |
| `native/operator_reference.h` | 1–58 | primary |
| `native/projection_probe.inc` | 1–29 | primary |
| `native/projection_reference.h` | 1–30 | primary |
| `native/qwen_device.inc` | 1–59 | primary |
| `native/qwen_model.inc` | 1–173 | primary |
| `native/qwen_mtp.inc` | 1–55 | primary |
| `native/state_table.h` | 1–38 | primary |
| `native/telemetry.cpp` | 1–23 | primary |
| `package-lock.json` | 1–1072 | config |
| `package.json` | 1–21 | config |
| `requirements-bench.txt` | 1–2 | config |
| `run.sh` | 1–5 | primary |
| `schema/trace.fbs` | 1–12 | primary |
| `src/backend.rs` | 1–614 | primary |
| `src/cache/drafts.rs` | 1–599 | primary |
| `src/cache/mod.rs` | 1–1078 | primary |
| `src/chain.rs` | 1–50 | primary |
| `src/completion.rs` | 1–218 | primary |
| `src/cordic.rs` | 1–204 | primary |
| `src/cordic_tables.rs` | 1–102 | primary |
| `src/execute.rs` | 1–122 | primary |
| `src/gpu.rs` | 1–440 | primary |
| `src/ir.rs` | 1–646 | primary |
| `src/lib.rs` | 1–30 | primary |
| `src/macro_bench.rs` | 1–81 | primary |
| `src/main.rs` | 1–453 | primary |
| `src/runtime.rs` | 1–366 | primary |
| `src/sampled.rs` | 1–193 | primary |
| `src/serving/choices.rs` | 1–123 | primary |
| `src/serving/completion_ledger.rs` | 1–100 | primary |
| `src/serving/config.rs` | 1–51 | primary |
| `src/serving/kv.rs` | 1–280 | primary |
| `src/serving/mod.rs` | 1–107 | primary |
| `src/serving/model_ir.rs` | 1–244 | primary |
| `src/serving/native_automaton.rs` | 1–328 | primary |
| `src/serving/native_boundary.rs` | 1–193 | primary |
| `src/serving/native_device.rs` | 1–809 | primary |
| `src/serving/native_device_scheduler.rs` | 1–178 | primary |
| `src/serving/native_device_smoke.rs` | 1–162 | primary |
| `src/serving/native_diagnose.rs` | 1–104 | primary |
| `src/serving/native_extended.rs` | 1–214 | primary |
| `src/serving/native_ingestion.rs` | 1–272 | primary |
| `src/serving/native_layer_reference.rs` | 1–93 | primary |
| `src/serving/native_mlx.rs` | 1–950 | primary |
| `src/serving/native_mtp.rs` | 1–468 | primary |
| `src/serving/native_mtp_automaton.rs` | 1–98 | primary |
| `src/serving/native_mtp_decode.rs` | 1–149 | primary |
| `src/serving/native_mtp_device.rs` | 1–436 | primary |
| `src/serving/native_ops.rs` | 1–273 | primary |
| `src/serving/native_qwen.rs` | 1–690 | primary |
| `src/serving/native_reference.rs` | 1–100 | primary |
| `src/serving/native_speculative.rs` | 1–382 | primary |
| `src/serving/native_state.rs` | 1–194 | primary |
| `src/serving/native_tokenizer.rs` | 1–213 | primary |
| `src/serving/qwen_tests.rs` | 1–566 | primary |
| `src/serving/rapid.rs` | 1–265 | primary |
| `src/serving/scheduler.rs` | 1–645 | primary |
| `src/serving/weights.rs` | 1–395 | primary |
| `src/telemetry.rs` | 1–26 | primary |
| `src/trace_generated.rs` | 1–1523 | primary |
| `src/wire.rs` | 1–124 | primary |
| `tests/adversarial.rs` | 1–456 | test |
| `tests/async_completion.rs` | 1–192 | test |
| `tests/cache_control.rs` | 1–419 | test |
| `tests/cordic.rs` | 1–264 | test |
| `tests/draft_admission.rs` | 1–475 | test |
| `tests/fixed_point.rs` | 1–146 | test |
| `tests/grid.rs` | 1–41 | test |
| `tests/native_execution.rs` | 1–486 | test |
| `tests/native_mlx.rs` | 1–297 | test |
| `tests/output_choices.rs` | 1–110 | test |
| `tests/production_pipeline.rs` | 1–315 | test |
| `tests/rapid_rpc.rs` | 1–76 | test |
| `tests/resources.rs` | 1–341 | test |
| `tests/serving.rs` | 1–435 | test |
| `ts/generated/trace.ts` | 1–5 | primary |
| `ts/generated/wire/add.ts` | 1–68 | primary |
| `ts/generated/wire/cordic.ts` | 1–78 | primary |
| `ts/generated/wire/dot.ts` | 1–68 | primary |
| `ts/generated/wire/load.ts` | 1–81 | primary |
| `ts/generated/wire/op.ts` | 1–61 | primary |
| `ts/generated/wire/operation.ts` | 1–54 | primary |
| `ts/generated/wire/relu.ts` | 1–58 | primary |
| `ts/generated/wire/store.ts` | 1–61 | primary |
| `ts/generated/wire/trace.ts` | 1–91 | primary |
| `ts/generated/wire.ts` | 1–13 | primary |
| `ts/smoke.ts` | 1–1 | primary |
| `ts/trace.test.ts` | 1–45 | test |
| `ts/trace.ts` | 1–73 | primary |
| `tsconfig.json` | 1–1 | config |
| `README.md` | 1–1080 | primary |
| `docs/ACCELERATOR_ROADMAP.md` | 1–516 | primary |
| `docs/ADVERSARIAL_REVIEW.md` | 1–706 | primary |
| `docs/ASYNC_COMPLETION.md` | 1–549 | primary |
| `docs/DESIGN.md` | 1–503 | primary |
| `docs/DEVICE_STATE_IMPLEMENTATION.md` | 1–641 | primary |
| `docs/DEVICE_STATE_OWNERSHIP.md` | 1–640 | primary |
| `docs/FILE_MAP.md` | 1–664 | primary |
| `docs/FIXED_POINT.md` | 1–573 | primary |
| `docs/Failures.md` | 1–501 | primary |
| `docs/GRID_THROUGHPUT.md` | 1–532 | primary |
| `docs/MODEL_ADVERSARIAL_REVIEW.md` | 1–514 | primary |
| `docs/MODEL_INTEGRATION.md` | 1–621 | primary |
| `docs/NATIVE_DISCRIMINATORS.md` | 1–560 | primary |
| `docs/NATIVE_LAYER_REFERENCE.md` | 1–648 | primary |
| `docs/NATIVE_MLX_BACKEND.md` | 1–586 | primary |
| `docs/NATIVE_QWEN_GENERATION.md` | 1–869 | primary |
| `docs/NATIVE_SPECULATIVE_VERIFIER.md` | 1–587 | primary |
| `docs/NATIVE_SUITE_REFERENCE.md` | 1–600 | primary |
| `docs/NATIVE_TUNING_PLAN.md` | 1–622 | primary |
| `docs/NATIVE_WEIGHTS_EXECUTION.md` | 1–526 | primary |
| `docs/NEXT_STEPS.md` | 1–755 | primary |
| `docs/PRODUCTION_PIPELINE.md` | 1–533 | primary |
| `docs/RESOURCE_BACKEND_DESIGN.md` | 1–513 | primary |
| `docs/RESOURCE_IMPLEMENTATION.md` | 1–570 | primary |
| `docs/SAMPLING_POLICY_STATUS.md` | 1–494 | primary |
| `docs/SERVING_ARCHITECTURE.md` | 1–638 | primary |
| `docs/SOURCE_SPEC.md` | 1–689 | primary |
| `docs/SPEC15_DRAFT_ADMISSION.md` | 1–544 | primary |
| `docs/SPEC15_IMPLEMENTATION.md` | 1–568 | primary |
| `docs/SPEC15_SOURCE.md` | 1–726 | primary |
| `docs/SPECULATIVE_RESEARCH_2026_10.md` | 1–593 | primary |

# Evidence

`evidence/judge-summary.json` is the final ordered run summary. `evidence/source-sha256.json` binds tested application/configuration/test/fixture source. `evidence/evidence-sha256.json` indexes every retained evidence file except itself. README benchmark values are copied from the final comparison JSON files. Earlier step logs are historical; the complete judge run and final hashes are authoritative for the final source.

# Changelog

1.0.0 — 2026-10-21 — Codex — Exact file line references and binary-fixture exception recorded.
