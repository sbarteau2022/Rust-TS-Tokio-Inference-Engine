//! MetaAutomaton 256K Paged KV, Cold Prefix Disaggregation & TurboQuant/MTP Coordinator.
//!
//! Manages 256-token KV pages up to full 256K (`262,144`) context on Apple Unified Memory:
//! - Non-TurboQ Regular KV (`RegularBf16` = 5.00 GiB at 256K, `RegularFp32` = 10.00 GiB at 256K)
//! - TurboQuant (`TurboQuantK8V4` = 1.875 GiB at 256K, `TurboQuantV4` = 1.25 GiB at 256K)
//!   with protected unquantized BF16 sink (128 tokens) + recent window (1,024 tokens)
//! - Disaggregated MTP / DSpark sidecar state (1,024-token sliding ring buffer <= 4 MiB)
//! - Predictive prefix offloading (`HotGpu` <-> `WarmUnified` <-> `ColdTurboQ`) off the hot path
//! - O(1) zero-copy speculative tail rewind on partial draft rejection (`refcount == 1`)

use super::{BlockKey, Controller, Identity};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub const MAX_CONTEXT_256K: usize = 262_144;
pub const PAGE_TOKENS: usize = 256;
pub const MAX_PAGES_256K: usize = MAX_CONTEXT_256K / PAGE_TOKENS; // 1,024 pages
pub const FULL_ATTN_LAYERS: usize = 10;
pub const LINEAR_ATTN_LAYERS: usize = 30;
pub const KV_HEADS: usize = 2;
pub const HEAD_DIM: usize = 256;
pub const MTP_SLIDING_WINDOW_TOKENS: usize = 1024;
pub const SINK_PROTECTED_TOKENS: usize = 128;
pub const RECENT_PROTECTED_TOKENS: usize = 1024;

/// Constant O(1) state footprint for the 30 linear-attention (GDN + conv1d) layers in FP32:
/// 30 * (3 * 8192 + 32 * 128 * 128) * 4 = 65,863,680 bytes (62.81 MiB).
pub const LINEAR_STATE_BYTES_FP32: usize = LINEAR_ATTN_LAYERS * (3 * 8192 + 32 * 128 * 128) * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KvMode {
    /// Non-TurboQ BF16 KV cache: 20,480 bytes/tok -> 5.00 GiB at 256K (5.12 MiB per 256-tok page).
    RegularBf16,
    /// Non-TurboQ FP32 KV cache: 40,960 bytes/tok -> 10.00 GiB at 256K (10.24 MiB per 256-tok page).
    RegularFp32,
    /// Asymmetric TurboQuant K8V4: 8-bit keys + 4-bit values -> 1.875 GiB at 256K.
    TurboQuantK8V4,
    /// Symmetric TurboQuant V4 + QJL residual signs -> 1.25 GiB at 256K.
    TurboQuantV4,
}

impl KvMode {
    /// Bytes per 256-token page across all 10 full-attention layers when stored in this format.
    pub const fn page_bytes(self) -> usize {
        let elements_per_page = FULL_ATTN_LAYERS * 2 * KV_HEADS * PAGE_TOKENS * HEAD_DIM; // 2,621,440
        match self {
            Self::RegularBf16 => elements_per_page * 2, // 5,242,880 B (5.00 MiB)
            Self::RegularFp32 => elements_per_page * 4, // 10,485,760 B (10.00 MiB)
            Self::TurboQuantK8V4 => (elements_per_page * 3) / 4, // 1,966,080 B (1.875 MiB)
            Self::TurboQuantV4 => elements_per_page / 2, // 1,310,720 B (1.25 MiB)
        }
    }

    pub const fn uses_turboq(self) -> bool {
        matches!(self, Self::TurboQuantK8V4 | Self::TurboQuantV4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidecarKind {
    /// Built-in 1-layer Qwen3.6 MTP head (reads target FP32 hidden state, 1024-tok sliding KV).
    MtpSingleLayer,
    /// Disaggregated DSpark / DFlash 5-layer block-diffusion drafter + MTP verifier pipeline.
    DSparkCrossAttention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageResidency {
    /// Active on the Metal command buffer hot path (sink page 0 + recent sliding window + prefetched pages).
    HotGpu,
    /// Pushed off the hot command encoder into pinned macOS unified memory (zero-copy accessible on demand).
    WarmUnifiedOffHotPath,
    /// Compressed with TurboQuant (k8v4/v4) in cold prefix storage; protected sink/tail stay unquantized.
    ColdTurboQuant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvPageDescriptor {
    pub page_index: usize,
    pub start_token: usize,
    pub valid_tokens: usize,
    pub residency: PageResidency,
    pub mode: KvMode,
    pub byte_size: usize,
    pub prefix_digest: Option<BlockKey>,
    pub pinned_sink: bool,
    pub last_access_step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAutomatonConfig {
    pub max_context_tokens: usize,
    pub kv_mode: KvMode,
    pub sidecar: SidecarKind,
    /// Number of recent 256-token pages kept on the hot GPU path (default 4 = 1,024 tokens).
    pub hot_tail_pages: usize,
    /// Maximum unified memory budget for the 256K KV arena + linear state + MTP sidecar.
    pub memory_budget_bytes: usize,
    /// Minimum top-1 vs top-2 logit margin for multi-step draft continuation.
    pub draft_margin_threshold: f32,
}

impl Default for MetaAutomatonConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: MAX_CONTEXT_256K,
            kv_mode: KvMode::RegularBf16,
            sidecar: SidecarKind::MtpSingleLayer,
            hot_tail_pages: 4, // 1,024 hot recent tokens + Page 0 sink
            memory_budget_bytes: 12 * 1024 * 1024 * 1024, // 12 GiB default arena cap
            draft_margin_threshold: 1.25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaAutomatonTelemetry {
    pub position_tokens: usize,
    pub allocated_pages: usize,
    pub hot_pages: usize,
    pub warm_offloaded_pages: usize,
    pub cold_turboq_pages: usize,
    pub target_kv_bytes: usize,
    pub linear_state_bytes: usize,
    pub sidecar_kv_bytes: usize,
    pub total_state_bytes: usize,
    pub speculative_commits: u64,
    pub speculative_rewinds: u64,
    pub prefix_demotions: u64,
    pub predictive_prefetches: u64,
}

/// Manages 256K context paging, prefix disaggregation off the hot path, TurboQ hybrid windows,
/// and O(1) speculative draft rollback without CoW full-context buffer cloning.
pub struct MetaAutomaton {
    config: MetaAutomatonConfig,
    controller: Option<Controller>,
    identity: Identity,
    pages: Vec<KvPageDescriptor>,
    position: usize,
    step_counter: u64,
    speculative_commits: u64,
    speculative_rewinds: u64,
    prefix_demotions: u64,
    predictive_prefetches: u64,
}

impl MetaAutomaton {
    pub fn new(
        config: MetaAutomatonConfig,
        controller: Option<Controller>,
        identity: Identity,
    ) -> Result<Self> {
        ensure!(
            (512..=MAX_CONTEXT_256K).contains(&config.max_context_tokens)
                && config.max_context_tokens.is_multiple_of(PAGE_TOKENS),
            "MetaAutomaton max_context_tokens must be a multiple of 256 in 512..=262144"
        );
        ensure!(
            (1..=64).contains(&config.hot_tail_pages),
            "MetaAutomaton hot_tail_pages must be in 1..=64"
        );
        ensure!(
            config.draft_margin_threshold.is_finite() && config.draft_margin_threshold >= 0.0,
            "MetaAutomaton draft_margin_threshold must be finite nonnegative"
        );
        ensure!(
            config.memory_budget_bytes >= LINEAR_STATE_BYTES_FP32 + 16 * 1024 * 1024,
            "MetaAutomaton memory_budget_bytes too small for hybrid state"
        );
        Ok(Self {
            config,
            controller,
            identity,
            pages: Vec::new(),
            position: 0,
            step_counter: 0,
            speculative_commits: 0,
            speculative_rewinds: 0,
            prefix_demotions: 0,
            predictive_prefetches: 0,
        })
    }

    pub fn config(&self) -> &MetaAutomatonConfig {
        &self.config
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn pages(&self) -> &[KvPageDescriptor] {
        &self.pages
    }

    /// Sidecar (MTP / DSpark) KV bytes are bounded by a 1,024-token sliding window (<= 4 MiB),
    /// decoupled from the 256K target prefix.
    pub fn sidecar_kv_bytes(&self) -> usize {
        let window_tokens = self.position.clamp(1, MTP_SLIDING_WINDOW_TOKENS);
        let active_pages = window_tokens.div_ceil(PAGE_TOKENS);
        let bytes_per_tok = match self.config.sidecar {
            SidecarKind::MtpSingleLayer => 2 * KV_HEADS * HEAD_DIM * 2, // BF16: 2,048 B/tok
            SidecarKind::DSparkCrossAttention => 5 * 2 * KV_HEADS * HEAD_DIM * 2,
        };
        active_pages * PAGE_TOKENS * bytes_per_tok
    }

    pub fn target_kv_bytes(&self) -> usize {
        self.pages.iter().map(|p| p.byte_size).sum()
    }

    pub fn total_state_bytes(&self) -> usize {
        LINEAR_STATE_BYTES_FP32 + self.target_kv_bytes() + self.sidecar_kv_bytes()
    }

    /// Appends `tokens` to the 256-token paged KV table, allocating new 256-token pages on demand
    /// and pushing cold prefix pages off the hot GPU path while preserving sink + hot tail residency.
    pub fn append_tokens(&mut self, tokens: &[u32]) -> Result<MetaAutomatonTelemetry> {
        ensure!(!tokens.is_empty(), "cannot append empty token slice");
        ensure!(
            self.position + tokens.len() <= self.config.max_context_tokens,
            "context capacity exceeded ({} + {} > {})",
            self.position,
            tokens.len(),
            self.config.max_context_tokens
        );
        self.step_counter += 1;
        let mut remaining = tokens;
        while !remaining.is_empty() {
            let page_idx = self.position / PAGE_TOKENS;
            let offset_in_page = self.position % PAGE_TOKENS;
            let space = PAGE_TOKENS - offset_in_page;
            let take = remaining.len().min(space);

            if page_idx == self.pages.len() {
                // Check budget before allocating a new 256-token page.
                let new_page_bytes = KvMode::RegularBf16.page_bytes();
                ensure!(
                    self.total_state_bytes() + new_page_bytes <= self.config.memory_budget_bytes,
                    "MetaAutomaton 256K memory budget exhausted"
                );
                self.pages.push(KvPageDescriptor {
                    page_index: page_idx,
                    start_token: page_idx * PAGE_TOKENS,
                    valid_tokens: 0,
                    residency: PageResidency::HotGpu,
                    mode: if self.config.kv_mode == KvMode::RegularFp32 {
                        KvMode::RegularFp32
                    } else {
                        // New active tail pages always start in unquantized BF16 so MTP/DSpark
                        // draft verification experiences zero quantization jitter on recent tokens.
                        KvMode::RegularBf16
                    },
                    byte_size: if self.config.kv_mode == KvMode::RegularFp32 {
                        KvMode::RegularFp32.page_bytes()
                    } else {
                        KvMode::RegularBf16.page_bytes()
                    },
                    prefix_digest: None,
                    pinned_sink: page_idx == 0,
                    last_access_step: self.step_counter,
                });
            }

            let page = &mut self.pages[page_idx];
            page.valid_tokens += take;
            page.last_access_step = self.step_counter;
            if page.valid_tokens == PAGE_TOKENS && page.prefix_digest.is_none() {
                let slice_tokens = &remaining[..take];
                if let Ok(digest) = super::prefix_key(&self.identity, slice_tokens) {
                    page.prefix_digest = Some(digest);
                }
            }

            self.position += take;
            remaining = &remaining[take..];
        }

        self.rebalance_prefix_tiers();
        Ok(self.telemetry())
    }

    /// Pushes completed prefix pages outside `[Page 0 (sink)] U [last hot_tail_pages]` off the hot path:
    /// - In Non-TurboQ mode: transitions middle pages to `WarmUnifiedOffHotPath`.
    /// - In TurboQuant (`k8v4`/`v4`) mode: compresses middle pages outside the 1,024-token recent window
    ///   to `ColdTurboQuant`, shrinking page footprint from 5.00 MiB -> 1.875 MiB (`k8v4`) or 1.25 MiB (`v4`).
    pub fn rebalance_prefix_tiers(&mut self) {
        let total_pages = self.pages.len();
        if total_pages <= self.config.hot_tail_pages + 1 {
            return;
        }
        let hot_tail_start = total_pages.saturating_sub(self.config.hot_tail_pages);
        for (idx, page) in self.pages.iter_mut().enumerate() {
            if idx == 0 || page.pinned_sink || idx >= hot_tail_start {
                page.residency = PageResidency::HotGpu;
                continue;
            }
            if page.valid_tokens < PAGE_TOKENS {
                continue;
            }
            if self.config.kv_mode.uses_turboq() {
                if page.residency != PageResidency::ColdTurboQuant {
                    page.residency = PageResidency::ColdTurboQuant;
                    page.mode = self.config.kv_mode;
                    page.byte_size = self.config.kv_mode.page_bytes();
                    self.prefix_demotions += 1;
                }
            } else if page.residency == PageResidency::HotGpu {
                page.residency = PageResidency::WarmUnifiedOffHotPath;
                self.prefix_demotions += 1;
            }
        }
    }

    /// Predictively promotes specific offloaded prefix pages back to `HotGpu` before attention reads them
    /// (e.g. for sparse retrieval or chunked Split-KV attention at 256K).
    pub fn prefetch_prefix_pages(&mut self, page_indices: &[usize]) -> Result<usize> {
        self.step_counter += 1;
        let mut promoted = 0;
        for &idx in page_indices {
            ensure!(idx < self.pages.len(), "prefetch page index out of bounds");
            let page = &mut self.pages[idx];
            page.last_access_step = self.step_counter;
            if page.residency != PageResidency::HotGpu {
                page.residency = PageResidency::HotGpu;
                promoted += 1;
                self.predictive_prefetches += 1;
            }
        }
        Ok(promoted)
    }

    /// Executes an O(1) zero-copy speculative transaction update on the paged KV arena:
    /// - First provisionally appends `proposed` draft tokens.
    /// - If `accepted < proposed`, rewinds the tail write cursor back to `accepted + 1` (the target
    ///   correction token) in O(1) without copying or reallocating prefix pages!
    /// - If `accepted == proposed` and `bonus_token.is_some()`, appends the bonus token.
    pub fn commit_or_rewind_speculative(
        &mut self,
        proposed_tokens: &[u32],
        accepted_count: usize,
        correction_or_bonus: Option<u32>,
    ) -> Result<MetaAutomatonTelemetry> {
        ensure!(
            !proposed_tokens.is_empty()
                && proposed_tokens.len() <= 8
                && accepted_count <= proposed_tokens.len(),
            "invalid speculative proposal/acceptance bounds"
        );
        let base_pos = self.position;
        self.append_tokens(proposed_tokens)?;
        if accepted_count < proposed_tokens.len() {
            // Partial rejection: rewind provisional draft tail in O(1) and write correction token.
            let target_pos = base_pos + accepted_count;
            self.rewind_to(target_pos)?;
            self.speculative_rewinds += 1;
            if let Some(tok) = correction_or_bonus {
                self.append_tokens(&[tok])?;
            }
        } else if let Some(bonus) = correction_or_bonus {
            if self.position < self.config.max_context_tokens {
                self.append_tokens(&[bonus])?;
            }
        }
        self.speculative_commits += 1;
        Ok(self.telemetry())
    }

    /// Rewinds the KV page table to `target_position` in O(1) by truncating only the uncommitted tail.
    pub fn rewind_to(&mut self, target_position: usize) -> Result<()> {
        ensure!(
            target_position <= self.position,
            "cannot rewind forward ({} > {})",
            target_position,
            self.position
        );
        if target_position == 0 {
            self.pages.clear();
            self.position = 0;
            return Ok(());
        }
        let keep_pages = target_position.div_ceil(PAGE_TOKENS);
        self.pages.truncate(keep_pages);
        if let Some(last) = self.pages.last_mut() {
            let rem = target_position % PAGE_TOKENS;
            last.valid_tokens = if rem == 0 { PAGE_TOKENS } else { rem };
            last.residency = PageResidency::HotGpu;
            if last.valid_tokens < PAGE_TOKENS {
                last.prefix_digest = None;
            }
        }
        self.position = target_position;
        Ok(())
    }

    pub fn controller(&self) -> Option<&Controller> {
        self.controller.as_ref()
    }

    pub fn telemetry(&self) -> MetaAutomatonTelemetry {
        let mut hot = 0;
        let mut warm = 0;
        let mut cold = 0;
        for p in &self.pages {
            match p.residency {
                PageResidency::HotGpu => hot += 1,
                PageResidency::WarmUnifiedOffHotPath => warm += 1,
                PageResidency::ColdTurboQuant => cold += 1,
            }
        }
        let target_kv = self.target_kv_bytes();
        let sidecar_kv = self.sidecar_kv_bytes();
        MetaAutomatonTelemetry {
            position_tokens: self.position,
            allocated_pages: self.pages.len(),
            hot_pages: hot,
            warm_offloaded_pages: warm,
            cold_turboq_pages: cold,
            target_kv_bytes: target_kv,
            linear_state_bytes: LINEAR_STATE_BYTES_FP32,
            sidecar_kv_bytes: sidecar_kv,
            total_state_bytes: LINEAR_STATE_BYTES_FP32 + target_kv + sidecar_kv,
            speculative_commits: self.speculative_commits,
            speculative_rewinds: self.speculative_rewinds,
            prefix_demotions: self.prefix_demotions,
            predictive_prefetches: self.predictive_prefetches,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_identity() -> Identity {
        Identity {
            model: "Qwen3.6-35B-A3B-MoE-Combined".into(),
            tokenizer: "qwen3_6_tokenizer".into(),
            quantization: "mixed-q4-q5-q6-q8".into(),
            namespace: "meta-automaton-256k".into(),
        }
    }

    #[test]
    fn full_256k_non_turboq_bf16_stays_within_5gib_kv_and_offloads_prefix_pages() {
        let cfg = MetaAutomatonConfig {
            max_context_tokens: MAX_CONTEXT_256K,
            kv_mode: KvMode::RegularBf16,
            sidecar: SidecarKind::MtpSingleLayer,
            hot_tail_pages: 4, // 1,024 hot tail tokens + 256 sink tokens
            memory_budget_bytes: 8 * 1024 * 1024 * 1024, // 8 GiB budget
            draft_margin_threshold: 1.25,
        };
        let mut meta = MetaAutomaton::new(cfg, None, test_identity()).unwrap();
        let chunk = vec![42u32; 4096];
        for _ in 0..(MAX_CONTEXT_256K / 4096) {
            meta.append_tokens(&chunk).unwrap();
        }
        let t = meta.telemetry();
        assert_eq!(t.position_tokens, 262_144);
        assert_eq!(t.allocated_pages, 1024);
        // 1 sink page + 4 hot tail pages = 5 hot pages; 1019 warm offloaded pages!
        assert_eq!(t.hot_pages, 5);
        assert_eq!(t.warm_offloaded_pages, 1019);
        assert_eq!(t.cold_turboq_pages, 0);
        // Exact 5.00 GiB (5,368,709,120 bytes) for 256K non-TurboQ BF16 KV across 10 layers:
        assert_eq!(t.target_kv_bytes, 5 * 1024 * 1024 * 1024);
        // Exact 62.81 MiB (65,863,680 bytes) for 30 linear-attention layers:
        assert_eq!(t.linear_state_bytes, 65_863_680);
        // MTP sliding-window KV capped at 1,024 tokens (2.0 MiB):
        assert_eq!(t.sidecar_kv_bytes, 2 * 1024 * 1024);

        // Predictive prefix prefetch brings offloaded pages back to HotGpu on demand:
        let promoted = meta.prefetch_prefix_pages(&[10, 20, 500]).unwrap();
        assert_eq!(promoted, 3);
        assert_eq!(meta.telemetry().hot_pages, 8);
    }

    #[test]
    fn turboquant_k8v4_and_v4_with_mtp_and_dspark_sidecar_at_256k() {
        for (mode, sidecar, expected_cold_page_bytes) in [
            (
                KvMode::TurboQuantK8V4,
                SidecarKind::MtpSingleLayer,
                1_966_080usize,
            ),
            (
                KvMode::TurboQuantV4,
                SidecarKind::DSparkCrossAttention,
                1_310_720usize,
            ),
        ] {
            let cfg = MetaAutomatonConfig {
                max_context_tokens: MAX_CONTEXT_256K,
                kv_mode: mode,
                sidecar,
                hot_tail_pages: 4,
                memory_budget_bytes: 4 * 1024 * 1024 * 1024, // Fits easily in < 2.1 GiB!
                draft_margin_threshold: 1.25,
            };
            let mut meta = MetaAutomaton::new(cfg, None, test_identity()).unwrap();
            let chunk = vec![7u32; 8192];
            for _ in 0..(MAX_CONTEXT_256K / 8192) {
                meta.append_tokens(&chunk).unwrap();
            }
            let t = meta.telemetry();
            assert_eq!(t.position_tokens, 262_144);
            assert_eq!(t.hot_pages, 5); // Sink page 0 + 4 recent tail pages stay unquantized BF16
            assert_eq!(t.cold_turboq_pages, 1019);
            let expected_target_bytes =
                5 * KvMode::RegularBf16.page_bytes() + 1019 * expected_cold_page_bytes;
            assert_eq!(t.target_kv_bytes, expected_target_bytes);
            assert!(t.total_state_bytes < 2_200_000_000);
        }
    }

    #[test]
    fn speculative_commit_and_rewind_is_zero_copy_and_exact() {
        let mut meta =
            MetaAutomaton::new(MetaAutomatonConfig::default(), None, test_identity()).unwrap();
        meta.append_tokens(&vec![1u32; 254]).unwrap();
        assert_eq!(meta.position(), 254);
        assert_eq!(meta.pages().len(), 1);

        // Propose 6 tokens crossing the 256-token page boundary (positions 254..260),
        // accept only 1, and emit 1 correction token -> final position must rewind to 256 (1 page)!
        let t = meta
            .commit_or_rewind_speculative(&[10, 11, 12, 13, 14, 15], 1, Some(99))
            .unwrap();
        assert_eq!(t.position_tokens, 256);
        assert_eq!(t.allocated_pages, 1);
        assert_eq!(t.speculative_commits, 1);
        assert_eq!(t.speculative_rewinds, 1);
    }
}
