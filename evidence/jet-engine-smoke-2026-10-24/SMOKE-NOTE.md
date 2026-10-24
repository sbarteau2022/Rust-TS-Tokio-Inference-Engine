# Jet-engine smoke — 2026-10-24 (branch feat/jet-engine-256k-fused)

Command: ./target/release/examples/native_mtp_bench evidence/jet-engine-smoke-2026-10-24 4 eager 40 reference
Binary: target/release/examples/native_mtp_bench (built 2026-10-24 09:05 from this tree, uncommitted jet-engine work included)
Model: ~/models/fused/Qwen3.6-MoE-Combined, identity 509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b
Scope: single lane, warmed Rust API, no HTTP; tokenization and model load excluded. MTP receipt groups tokens, so receipt interval is NOT token-streaming ITL.

Result: pass=true, output_congruence=true (MTP token ids identical to baseline greedy on all 4 prompts x 2 rounds).

Baseline target-greedy decode: 25.35 / 25.68 / 25.81 / 25.71 / 25.79 / 25.83 / 25.78 / 25.73 tok/s (mean ~25.71)
MTP width-4 reuse decode: 65.65 / 65.97 / 56.48 / 42.79 / 66.10 / 65.89 / 56.55 / 43.14 tok/s (mean ~57.82)
Acceptance per round: 83/86 = 96.5% (code-repeat 23/23, code-tests 23/23, code-loop 21/23, security-review 16/17)
MLX peak: 20,654,565,312 bytes. Logical bytes at shutdown: 0.

Comparison discipline: these are Rust-API warmed-decode numbers, NOT the scorecard HTTP E2E control (36.10 tok/s single-lane) and NOT the post-first-decode lane figure (49.83 tok/s). Do not compare across scopes. 150 tok/s is NOT achieved by this smoke.
