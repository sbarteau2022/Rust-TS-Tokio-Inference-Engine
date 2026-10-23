// Conservative transactional MTP decoding; replay costs are explicit.
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
}
impl MtpCursor {
    pub fn target(&self) -> &GreedyCursor { &self.target }
}
impl QwenDeviceExecutor {
    /// Prompt IDs only. Captures and consumes shifted hidden windows on device.
    pub async fn mtp_start(&self, prompt: &[u32]) -> Result<MtpCursor> {
        ensure!(!prompt.is_empty() && prompt.len() < 512, "MTP prompt capacity");
        let mut state = self.empty_state();
        let mut draft = self.empty_mtp_state()?;
        let mut latest = None;
        let mut logits = Vec::new();
        for (index, tokens) in prompt.chunks(8).enumerate() {
            let start = index * 8;
            let block = self.hidden_block(&state, tokens.to_vec()).await?;
            logits = block.logits.last().expect("nonempty hidden block").clone();
            let next = if start + tokens.len() < prompt.len() {
                prompt[start + tokens.len()]
            } else { winner(&logits)? };
            let mut shifted = tokens[1..].to_vec();
            shifted.push(next);
            let result = self.mtp_prefill(&draft, &block.state, 0, shifted).await?;
            draft = result.state.clone();
            latest = Some(result);
            state = block.state;
        }
        Ok(MtpCursor {
            target: GreedyCursor { state, logits: Arc::new(logits), terminal: false },
            draft: latest.expect("validated prompt"),
        })
    }
    /// Exact greedy verification followed by authoritative hidden-state replay.
    /// Always replay from the immutable committed parent, including full acceptance.
    /// This correctness profile makes no profitability or scheduler claim.
    pub async fn mtp_decode(&self, cursor: &MtpCursor, width: usize, eos: &[u32]) -> Result<MtpTransaction> {
        ensure!(!cursor.target.terminal && (1..=8).contains(&width), "MTP terminal/width");
        let remaining = 512usize.saturating_sub(cursor.target.state.position());
        ensure!(remaining > 0, "MTP context exhausted");
        ensure!(eos.iter().all(|&t| (t as usize) < cursor.target.logits.len()), "MTP EOS vocabulary");
        let count = width.min(remaining);
        let mut tokens = vec![winner(&cursor.target.logits)?];
        let mut branch = cursor.draft.state.clone();
        let mut draft_logits = cursor.draft.logits.clone();
        let mut draft_calls = 0;
        while tokens.len() < count && !eos.contains(tokens.last().expect("seed")) {
            let token = winner(&draft_logits)?;
            tokens.push(token);
            if tokens.len() < count && !eos.contains(&token) {
                let step = self.mtp_next(&branch, token).await?;
                branch = step.state;
                draft_logits = step.logits;
                draft_calls += 1;
            }
        }
        drop(branch);
        let proposed = tokens.len();
        let block = self.verify_block_guarded(&cursor.target.state, tokens.clone(), None, None).await?;
        let decision = plan(&cursor.target.logits, &tokens, &block.logits, eos, remaining)?;
        drop(block);
        let mut state = cursor.target.state.clone();
        let mut draft = cursor.draft.state.clone();
        let mut latest = None;
        let mut logits = Vec::new();
        let mut target_calls = 1;
        for (index, chunk) in decision.emitted.chunks(8).enumerate() {
            let start = index * 8;
            let block = self.hidden_block(&state, chunk.to_vec()).await?;
            target_calls += 1;
            logits = block.logits.last().expect("nonempty replay").clone();
            let next = if start + chunk.len() < decision.emitted.len() {
                decision.emitted[start + chunk.len()]
            } else { winner(&logits)? };
            let mut shifted = chunk[1..].to_vec();
            shifted.push(next);
            let result = self.mtp_prefill(&draft, &block.state, 0, shifted).await?;
            draft_calls += 1;
            draft = result.state.clone();
            latest = Some(result);
            state = block.state;
        }
        ensure!(state.position() == draft.position(), "MTP committed alignment");
        Ok(MtpTransaction {
            cursor: MtpCursor {
                target: GreedyCursor { state, logits: Arc::new(logits), terminal: decision.terminal },
                draft: latest.expect("nonempty greedy decision"),
            },
            emitted: decision.emitted,
            learned_proposed: proposed.saturating_sub(1),
            learned_accepted: decision.accepted.saturating_sub(1),
            target_calls,
            draft_calls,
        })
    }
}
