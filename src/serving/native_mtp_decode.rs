#[cfg(feature = "mtp-phase-timing")]
#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub struct MtpPhaseTiming {
    pub prefill_ns: u64,
    pub target_prefill_ns: u64,
    pub draft_prefill_ns: u64,
    pub prepare_ns: u64,
    pub draft_ns: u64,
    pub commit_ns: u64,
    pub verify_ns: u64,
    pub replay_ns: u64,
    pub refresh_ns: u64,
}
#[cfg(feature = "mtp-phase-timing")]
fn phase_ns(start: std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).expect("phase duration fits u64 nanoseconds")
}
// Transactional MTP decoding with accepted target-window reuse.
#[derive(Clone)]
pub struct MtpCursor {
    target: GreedyCursor,
    draft: super::MtpStep,
    #[cfg(feature = "mtp-phase-timing")]
    pub prefill_timing: MtpPhaseTiming,
}
pub struct MtpTransaction {
    #[cfg(feature = "mtp-phase-timing")]
    pub phase_timing: MtpPhaseTiming,
    pub cursor: MtpCursor,
    pub emitted: Vec<u32>,
    /// Excludes the first token already known from target logits.
    pub learned_proposed: usize,
    pub learned_accepted: usize,
    pub target_calls: usize,
    pub draft_calls: usize,
    pub reused_target_tokens: usize,
    pub replayed_target_tokens: usize,
    pub target_forward_tokens: usize,
}
pub(super) struct MtpPrepared {
    #[cfg(feature = "mtp-phase-timing")]
    phase_timing: MtpPhaseTiming,
    cursor: MtpCursor,
    tokens: Vec<u32>,
    draft_calls: usize,
    remaining: usize,
    eos: Vec<u32>,
    completion: Option<super::CompletionGuard>,
}
impl MtpPrepared {
    pub(super) fn tokens(&self) -> &[u32] {
        &self.tokens
    }
}
impl MtpCursor {
    pub fn resident_bytes(&self) -> usize {
        self.target.state.bytes() + self.draft.state.bytes()
    }
    /// Read-only opaque state for offline state-equivalence qualification.
    pub fn draft_state(&self) -> &super::MtpState {
        &self.draft.state
    }
    pub fn target(&self) -> &GreedyCursor {
        &self.target
    }
}
impl QwenDeviceExecutor {
    /// Prompt IDs only. Captures and consumes shifted hidden windows on device.
    pub async fn mtp_start(&self, prompt: &[u32]) -> Result<MtpCursor> {
        let context_cap = self.context_capacity();
        ensure!(
            !prompt.is_empty() && prompt.len() < context_cap,
            "MTP prompt capacity"
        );
        #[cfg(feature = "mtp-phase-timing")]
        let (prefill_start, mut timing) = (std::time::Instant::now(), MtpPhaseTiming::default());
        let mut state = self.empty_state();
        let mut draft = self.empty_mtp_state()?;
        let mut latest = None;
        let mut logits = Vec::new();
        for (index, tokens) in prompt.chunks(8).enumerate() {
            let start = index * 8;
            #[cfg(feature = "mtp-phase-timing")]
            let target_start = std::time::Instant::now();
            let step = self.hidden_step(&state, tokens.to_vec()).await?;
            #[cfg(feature = "mtp-phase-timing")]
            {
                timing.target_prefill_ns += phase_ns(target_start);
            }
            logits = step.logits;
            let next = if start + tokens.len() < prompt.len() {
                prompt[start + tokens.len()]
            } else {
                winner(&logits)?
            };
            let mut shifted = Vec::with_capacity(tokens.len());
            shifted.extend_from_slice(&tokens[1..]);
            shifted.push(next);
            #[cfg(feature = "mtp-phase-timing")]
            let draft_start = std::time::Instant::now();
            let result = self.mtp_prefill(&draft, &step.state, 0, shifted).await?;
            #[cfg(feature = "mtp-phase-timing")]
            {
                timing.draft_prefill_ns += phase_ns(draft_start);
            }
            draft = result.state.clone();
            latest = Some(result);
            state = step.state;
        }
        #[cfg(feature = "mtp-phase-timing")]
        {
            timing.prefill_ns = phase_ns(prefill_start);
        }
        Ok(MtpCursor {
            #[cfg(feature = "mtp-phase-timing")]
            prefill_timing: timing,
            target: GreedyCursor {
                state,
                logits: Arc::new(logits),
                terminal: false,
            },
            draft: latest.expect("validated prompt"),
        })
    }
    /// Reuse fully accepted target windows; rejected unified state is replayed.
    pub async fn mtp_decode(
        &self,
        cursor: &MtpCursor,
        width: usize,
        eos: &[u32],
    ) -> Result<MtpTransaction> {
        self.mtp_decode_bounded(cursor, width, usize::MAX, eos)
            .await
    }
    /// Limit this transaction including any target bonus/correction token.
    pub async fn mtp_decode_bounded(
        &self,
        cursor: &MtpCursor,
        width: usize,
        output_budget: usize,
        eos: &[u32],
    ) -> Result<MtpTransaction> {
        self.mtp_decode_mode(cursor, width, output_budget, eos, true)
            .await
    }
    /// Executes the entire MTP speculative transaction (autoregressive draft + target block verify
    /// + accept/replay/reuse + draft refresh) inside a single C++20 worker call with on-GPU top-2/margin.
    pub async fn mtp_decode_fused(
        &self,
        cursor: &MtpCursor,
        width: usize,
        output_budget: usize,
        eos: &[u32],
    ) -> Result<MtpTransaction> {
        self.mtp_decode_fused_mode(cursor, width, output_budget, eos, true, None)
            .await
    }
    pub(super) async fn mtp_decode_fused_mode(
        &self,
        cursor: &MtpCursor,
        width: usize,
        output_budget: usize,
        eos: &[u32],
        reuse: bool,
        completion: Option<super::CompletionGuard>,
    ) -> Result<MtpTransaction> {
        #[cfg(feature = "mtp-phase-timing")]
        let tx_start = std::time::Instant::now();
        let context_cap = self.context_capacity();
        ensure!(
            !cursor.target.terminal && (1..=8).contains(&width),
            "MTP terminal/width"
        );
        let remaining = context_cap
            .saturating_sub(cursor.target.state.position())
            .min(output_budget);
        ensure!(remaining > 0, "MTP context exhausted");
        ensure!(
            eos.iter()
                .all(|&t| (t as usize) < cursor.target.logits.len()),
            "MTP EOS vocabulary"
        );
        let seed_token = winner(&cursor.target.logits)?;
        let (first_draft_token, first_draft_margin) = winner_with_margin(&cursor.draft.logits)?;
        let receipt = self
            .execute_mtp_fused(
                &cursor.target.state,
                &cursor.draft.state,
                seed_token,
                first_draft_token,
                first_draft_margin,
                width,
                output_budget,
                eos.to_vec(),
                reuse,
                completion,
            )
            .await?;
        #[cfg(feature = "mtp-phase-timing")]
        let timing = MtpPhaseTiming {
            commit_ns: phase_ns(tx_start),
            ..MtpPhaseTiming::default()
        };
        Ok(MtpTransaction {
            #[cfg(feature = "mtp-phase-timing")]
            phase_timing: timing,
            cursor: MtpCursor {
                #[cfg(feature = "mtp-phase-timing")]
                prefill_timing: MtpPhaseTiming::default(),
                target: GreedyCursor {
                    state: receipt.target_state,
                    logits: Arc::new(receipt.target_logits),
                    terminal: receipt.terminal,
                },
                draft: receipt.draft_step,
            },
            emitted: receipt.emitted,
            learned_proposed: receipt.learned_proposed,
            learned_accepted: receipt.learned_accepted,
            target_calls: receipt.target_calls,
            draft_calls: receipt.draft_calls,
            reused_target_tokens: receipt.reused_target_tokens,
            replayed_target_tokens: receipt.replayed_target_tokens,
            target_forward_tokens: receipt.target_forward_tokens,
        })
    }
    #[cfg(feature = "debug-validation")]
    pub async fn mtp_decode_replay(
        &self,
        cursor: &MtpCursor,
        width: usize,
        eos: &[u32],
    ) -> Result<MtpTransaction> {
        self.mtp_decode_mode(cursor, width, usize::MAX, eos, false)
            .await
    }
    async fn mtp_decode_mode(
        &self,
        cursor: &MtpCursor,
        width: usize,
        output_budget: usize,
        eos: &[u32],
        reuse: bool,
    ) -> Result<MtpTransaction> {
        #[cfg(not(feature = "mtp-phase-timing"))]
        if reuse {
            return self
                .mtp_decode_fused_mode(cursor, width, output_budget, eos, true, None)
                .await;
        }
        let prepared = self
            .mtp_prepare(cursor, width, output_budget, eos, None)
            .await?;
        self.mtp_commit(prepared, reuse, None).await
    }
    pub(super) async fn mtp_prepare(
        &self,
        cursor: &MtpCursor,
        width: usize,
        output_budget: usize,
        eos: &[u32],
        completion: Option<super::CompletionGuard>,
    ) -> Result<MtpPrepared> {
        #[cfg(feature = "mtp-phase-timing")]
        let (prepare_start, mut timing) = (std::time::Instant::now(), MtpPhaseTiming::default());
        let context_cap = self.context_capacity();
        ensure!(
            !cursor.target.terminal && (1..=8).contains(&width),
            "MTP terminal/width"
        );
        let remaining = context_cap
            .saturating_sub(cursor.target.state.position())
            .min(output_budget);
        ensure!(remaining > 0, "MTP context exhausted");
        ensure!(
            eos.iter()
                .all(|&t| (t as usize) < cursor.target.logits.len()),
            "MTP EOS vocabulary"
        );
        let count = width.min(remaining);
        let mut tokens = Vec::with_capacity(count);
        tokens.push(winner(&cursor.target.logits)?);
        let mut branch = cursor.draft.state.clone();
        let mut step_logits: Option<Vec<f32>> = None;
        let mut draft_calls = 0;
        while tokens.len() < count && !eos.contains(tokens.last().expect("seed")) {
            let active_logits = step_logits.as_deref().unwrap_or(&cursor.draft.logits);
            let (token, margin) = winner_with_margin(active_logits)?;
            // Always propose the first learned MTP token (conditioned on authoritative target hidden state);
            // gate deeper autoregressive draft steps on top-1 vs top-2 logit margin >= 1.25.
            if tokens.len() >= 2 && margin < 1.25 {
                break;
            }
            tokens.push(token);
            if tokens.len() < count && !eos.contains(&token) && margin >= 1.25 {
                #[cfg(feature = "mtp-phase-timing")]
                let draft_start = std::time::Instant::now();
                let step = self
                    .mtp_next_guarded(&branch, token, completion.clone())
                    .await?;
                #[cfg(feature = "mtp-phase-timing")]
                {
                    timing.draft_ns += phase_ns(draft_start);
                }
                branch = step.state;
                step_logits = Some(step.logits);
                draft_calls += 1;
            } else {
                break;
            }
        }
        drop(branch);
        #[cfg(feature = "mtp-phase-timing")]
        {
            timing.prepare_ns = phase_ns(prepare_start);
        }
        Ok(MtpPrepared {
            #[cfg(feature = "mtp-phase-timing")]
            phase_timing: timing,
            cursor: cursor.clone(),
            tokens,
            draft_calls,
            remaining,
            eos: eos.to_vec(),
            completion,
        })
    }
    pub(super) async fn mtp_commit(
        &self,
        prepared: MtpPrepared,
        reuse: bool,
        calls: Option<Arc<AtomicU64>>,
    ) -> Result<MtpTransaction> {
        #[cfg(feature = "mtp-phase-timing")]
        let commit_start = std::time::Instant::now();
        let MtpPrepared {
            #[cfg(feature = "mtp-phase-timing")]
                phase_timing: mut timing,
            cursor,
            tokens,
            mut draft_calls,
            remaining,
            eos,
            completion,
        } = prepared;
        let proposed = tokens.len();
        #[cfg(feature = "mtp-phase-timing")]
        let verify_start = std::time::Instant::now();
        let block = self
            .hidden_block_guarded(
                &cursor.target.state,
                tokens.clone(),
                completion.clone(),
                calls.clone(),
            )
            .await?;
        #[cfg(feature = "mtp-phase-timing")]
        {
            timing.verify_ns = phase_ns(verify_start);
        }
        let mut decision = plan(
            &cursor.target.logits,
            &tokens,
            &block.logits,
            &eos,
            remaining,
        )?;
        // When all proposed tokens match the target, emit the verified `proposed` window directly
        // and retain `block.logits.last()` as the next cursor's seed logits, avoiding a second
        // 40-layer target forward pass and second mtp_prefill call on full acceptance.
        if decision.accepted == proposed && decision.emitted.len() > proposed {
            decision.emitted.truncate(proposed);
            decision.terminal = decision.emitted.last().is_some_and(|t| eos.contains(t));
        }
        let reused_target_tokens = if reuse && decision.accepted == proposed {
            proposed
        } else {
            0
        };
        let mut verified = if reused_target_tokens > 0 {
            Some(block)
        } else {
            drop(block);
            None
        };
        let mut state = cursor.target.state.clone();
        let mut draft = cursor.draft.state.clone();
        let mut latest = None;
        let mut logits = Vec::new();
        let mut target_calls = 1;
        let mut start = 0;
        while start < decision.emitted.len() {
            let count = if start == 0 && reused_target_tokens > 0 {
                reused_target_tokens
            } else {
                (decision.emitted.len() - start).min(8)
            };
            let chunk = &decision.emitted[start..start + count];
            let next_state = if let Some(mut block) = verified.take() {
                logits = block.logits.pop().expect("nonempty verified block");
                block.state
            } else {
                target_calls += 1;
                #[cfg(feature = "mtp-phase-timing")]
                let replay_start = std::time::Instant::now();
                let step = self
                    .hidden_step_guarded(&state, chunk.to_vec(), completion.clone(), calls.clone())
                    .await?;
                #[cfg(feature = "mtp-phase-timing")]
                {
                    timing.replay_ns += phase_ns(replay_start);
                }
                logits = step.logits;
                step.state
            };
            let next = if start + chunk.len() < decision.emitted.len() {
                decision.emitted[start + chunk.len()]
            } else {
                winner(&logits)?
            };
            let mut shifted = Vec::with_capacity(chunk.len());
            shifted.extend_from_slice(&chunk[1..]);
            shifted.push(next);
            #[cfg(feature = "mtp-phase-timing")]
            let refresh_start = std::time::Instant::now();
            let result = self
                .mtp_prefill_guarded(&draft, &next_state, 0, shifted, completion.clone())
                .await?;
            #[cfg(feature = "mtp-phase-timing")]
            {
                timing.refresh_ns += phase_ns(refresh_start);
            }
            draft_calls += 1;
            draft = result.state.clone();
            latest = Some(result);
            state = next_state;
            start += count;
        }
        ensure!(
            state.position() == draft.position(),
            "MTP committed alignment"
        );
        #[cfg(feature = "mtp-phase-timing")]
        {
            timing.commit_ns = phase_ns(commit_start);
        }
        Ok(MtpTransaction {
            #[cfg(feature = "mtp-phase-timing")]
            phase_timing: timing,
            cursor: MtpCursor {
                #[cfg(feature = "mtp-phase-timing")]
                prefill_timing: MtpPhaseTiming::default(),
                target: GreedyCursor {
                    state,
                    logits: Arc::new(logits),
                    terminal: decision.terminal,
                },
                draft: latest.expect("nonempty greedy decision"),
            },
            replayed_target_tokens: decision.emitted.len() - reused_target_tokens,
            target_forward_tokens: proposed + decision.emitted.len() - reused_target_tokens,
            reused_target_tokens,
            emitted: decision.emitted,
            learned_proposed: proposed.saturating_sub(1),
            learned_accepted: decision.accepted.saturating_sub(1),
            target_calls,
            draft_calls,
        })
    }
}
