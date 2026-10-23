// Transactional MTP decoding with accepted target-window reuse.
#[derive(Clone)]
pub struct MtpCursor {
    target: GreedyCursor,
    draft: super::MtpStep,
}
pub struct MtpTransaction {
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
    pub fn target(&self) -> &GreedyCursor {
        &self.target
    }
}
impl QwenDeviceExecutor {
    /// Prompt IDs only. Captures and consumes shifted hidden windows on device.
    pub async fn mtp_start(&self, prompt: &[u32]) -> Result<MtpCursor> {
        ensure!(
            !prompt.is_empty() && prompt.len() < 512,
            "MTP prompt capacity"
        );
        let mut state = self.empty_state();
        let mut draft = self.empty_mtp_state()?;
        let mut latest = None;
        let mut logits = Vec::new();
        for (index, tokens) in prompt.chunks(8).enumerate() {
            let start = index * 8;
            let mut block = self.hidden_block(&state, tokens.to_vec()).await?;
            logits = block.logits.pop().expect("nonempty hidden block");
            let next = if start + tokens.len() < prompt.len() {
                prompt[start + tokens.len()]
            } else {
                winner(&logits)?
            };
            let mut shifted = Vec::with_capacity(tokens.len());
            shifted.extend_from_slice(&tokens[1..]);
            shifted.push(next);
            let result = self.mtp_prefill(&draft, &block.state, 0, shifted).await?;
            draft = result.state.clone();
            latest = Some(result);
            state = block.state;
        }
        Ok(MtpCursor {
            target: GreedyCursor {
                state,
                logits: Arc::new(logits),
                terminal: false,
            },
            draft: latest.expect("validated prompt"),
        })
    }
    /// Reuse fully accepted target windows; rejected unified state is replayed.
    /// This experimental profile makes no profitability or scheduler claim.
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
        ensure!(
            !cursor.target.terminal && (1..=8).contains(&width),
            "MTP terminal/width"
        );
        let remaining = 512usize
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
        let mut draft_logits = cursor.draft.logits.clone();
        let mut draft_calls = 0;
        while tokens.len() < count && !eos.contains(tokens.last().expect("seed")) {
            let token = winner(&draft_logits)?;
            tokens.push(token);
            if tokens.len() < count && !eos.contains(&token) {
                let step = self
                    .mtp_next_guarded(&branch, token, completion.clone())
                    .await?;
                branch = step.state;
                draft_logits = step.logits;
                draft_calls += 1;
            }
        }
        drop(branch);
        Ok(MtpPrepared {
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
        let MtpPrepared {
            cursor,
            tokens,
            mut draft_calls,
            remaining,
            eos,
            completion,
        } = prepared;
        let proposed = tokens.len();
        let block = self
            .hidden_block_guarded(
                &cursor.target.state,
                tokens.clone(),
                completion.clone(),
                calls.clone(),
            )
            .await?;
        let decision = plan(
            &cursor.target.logits,
            &tokens,
            &block.logits,
            &eos,
            remaining,
        )?;
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
            let mut block = if let Some(block) = verified.take() {
                block
            } else {
                target_calls += 1;
                self.hidden_block_guarded(&state, chunk.to_vec(), completion.clone(), calls.clone())
                    .await?
            };
            logits = block.logits.pop().expect("nonempty replay");
            let next = if start + chunk.len() < decision.emitted.len() {
                decision.emitted[start + chunk.len()]
            } else {
                winner(&logits)?
            };
            let mut shifted = Vec::with_capacity(chunk.len());
            shifted.extend_from_slice(&chunk[1..]);
            shifted.push(next);
            let result = self
                .mtp_prefill_guarded(&draft, &block.state, 0, shifted, completion.clone())
                .await?;
            draft_calls += 1;
            draft = result.state.clone();
            latest = Some(result);
            state = block.state;
            start += count;
        }
        ensure!(
            state.position() == draft.position(),
            "MTP committed alignment"
        );
        Ok(MtpTransaction {
            cursor: MtpCursor {
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
