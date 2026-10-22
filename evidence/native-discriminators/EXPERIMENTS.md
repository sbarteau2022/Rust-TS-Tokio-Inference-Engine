---
title: "Frozen-failure discriminator findings"
document_type: "implementation"
version: "1.0.0"
date: "2026-10-22"
implementation_date: null
author: "Codex for Stewart Barteau"
reviewers: []
status: "draft"
related_files: []
related_issues: []
related_documents: ["../../docs/NATIVE_DISCRIMINATORS.md"]
tags: [mlx, reference, ablation]
---
# Actual experiment order and scope

1. Verify native settings: greedy, speculation off, prefix reuse off, FP32 regular
KV. Absent features are not counted as ablations. Native KV-only 16-bit control
is unavailable; changing activation precision would confound it. Stock's fresh
checkpoint-dtype cache is BF16 KV, widened profile is FP32 KV (recorded dtypes).
2. Stock mlx-lm 0.31.3 / MLX 0.32.4 reference on both frozen prompts. Original
mixed directory produces garbage because glob-loading MTP shifts sanitized norms;
retain it as rejected reference. Accepted reference uses unchanged stock code on
symlinks to exact indexed main shards and the same boundary artifacts. No weights
modified, no Rapid/DSpark/DFlash imported. All 166 package Python files match
installed distribution RECORD. The tokenizer emits a regex warning; it is not
silently patched. Exact rendered template and token IDs match native on every
original/probe item in both stock profiles, limiting this parity claim to them.
3. Four separately labeled interpretation probes through native and both stock
profiles; expected answers do not enter model execution. No original fixture is
rewritten. The two user-proposed core questions agree across lanes: same dropped
future not polled, owned fields dropped. JoinHandle detachment also agrees.
Async cleanup is profile-sensitive: stock checkpoint dtype says No.; stock F32
and native F32 say Yes. The lowercase-only yes/no margin and raw capitalized
winner are reported separately: do not mistake that conditional margin for the
model's full answer-class probability.
4. The real-prompt embedding control builds in an isolated temporary source copy,
changes only the widening line, then runs the same native diagnostic. Canonical
source is never mutated. Logprobs use stable FP64 logsumexp over actual FP32 logits.
Source patch, hashes, binary hashes, output and successful drain are retained.

# Findings

| Lane | Cancellation raw answer | no minus yes logits | Borrowing raw answer |
| --- | --- | --- | --- |
| Native corrected F32 | No | 0.107872009 | No. |
| Native uncorrected F32 | No | 0.108324051 | No. |
| Stock checkpoint dtypes | No | 0.125000000 | No. |
| Stock widened F32 | No | 0.082023621 | No. |

Cancellation uppercase No-minus-Yes is 0.612108230 natively and 0.607517242 in
stock F32. Borrowing no-minus-yes is 7.089965820 natively and 7.088315964 in stock
F32; this is consistent with a correct semantic answer plus capitalization and
punctuation noncompliance. Exact output-choice compliance remains a separately
declared API profile; the original raw exactness failure stays failed.

Embedding correction moves cancellation's lowercase margin 0.000452042 toward
yes, with unchanged winner. Borrowing delta is -0.000455857, also unchanged.
This demonstrates a real full-prompt numerical effect. Its size does not establish
a dose-response cure, nor does reference agreement prove all graph routes or
states equivalent. These observations support model/profile behavior for these
items rather than speculative, prefix-cache or quantized-KV sensitivity in this
native lane; those serving features are absent here.

# Reproduction and integrity

`stock_compare.py` runs only as an explicitly requested external reference test,
never a Python worker or JSON transport in the Rust server. Set REFERENCE_MODEL
to the recorded index-defined view, REFERENCE_TAG=indexed-stock, and optionally
REFERENCE_PROBES=1 / REFERENCE_TAG=indexed-probes. Read indexed-stock-view.json for
exact symlink targets. `embedding_ab.py` clones the current source/build cache in
a temporary directory; APFS copy-on-write cache cloning is optional and full
compilation works if cloning fails. Both original model questions are compiled
from unchanged fixtures into examples/native_discriminators.rs. Two scalar
margin tests cover finite/bounds rejection and common-offset invariance.
`comparison.json` contains exact candidates, logits/logprobs, generated IDs,
cache dtypes and the profile-sensitive follow-up. Source/binary/provenance
artifacts are fingerprinted; no independent audit or production signoff claimed.


# Final-source verification

116 default / 116 debug / 87 minimal Rust tests, two native margin tests, five TS,
93 scalar projection/embedding and 43 operator cases, Clippy/format and existing
Metal/compiler/capture gates pass. The final Rust release smoke returns READY
and EOS on Combined. Unknown/duplicate example flags reject before model loading.
Original main fixtures and native forward code are unchanged in this phase;
full-model production quality remains unqualified. Reports are artifact JSON,
not a server runtime protocol. The prior phase's malformed native-ABI sanitizer
coverage still applies because this phase changes no native ABI or forward code.
