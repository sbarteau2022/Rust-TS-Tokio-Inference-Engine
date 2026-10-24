//! Spec 15's optional draft workload. Shares Controller ownership, epochs and decay.
use super::*;

#[derive(Clone, Debug)]
pub struct DraftPolicy {
    pub max_proposers: usize,
    pub work_limit: usize,
    pub initial_mass: u32,
    pub admission_mass: u32,
    pub reward_mass: u32,
    pub qualification_trials: u64,
    pub probe_every_epochs: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ProposerId(u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct DraftTicket(u64, u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DraftReason {
    Qualify,
    Earned,
    Probe,
}
pub enum DraftAdmission {
    Ready(DraftPermit),
    Queued(DraftTicket),
    Benched,
}
#[derive(Clone, Copy, Debug)]
pub struct DraftOutcome {
    /// Known target prefix; excluded from learned proposal/reward counts.
    pub known_prefix: u64,
    pub proposed: u64,
    pub accepted: u64,
    /// Final cursor handed to the response channel, not end-user delivery proof.
    pub committed: u64,
    /// Target commands admitted to the native worker, including failed attempts.
    pub target_calls: u64,
    /// End-to-end proposal + verify + replay cost, integer monotonic nanoseconds.
    pub cost_ns: u64,
    pub failed: bool,
}
/// Identity-scoped no-draft observations, separate from learned proposer commits.
#[derive(Clone, Default, Debug, Serialize, PartialEq, Eq)]
pub struct DraftBaseline {
    pub cost_ns: u64,
    pub committed: u64,
}
#[derive(Clone, Default, Debug, Serialize)]
pub struct DraftLedger {
    pub known_prefix: u64,
    pub observations: u64,
    pub proposed: u64,
    pub accepted: u64,
    pub committed: u64,
    pub target_calls: u64,
    pub cost_ns: u64,
    pub failures: u64,
    pub unfinished: u64,
    pub mass: u32,
    pub profitable: bool,
    pub window_cost_ns: u64,
    pub window_committed: u64,
    pub window_observations: u64,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct DraftStats {
    pub epoch: u64,
    pub registered: usize,
    pub active: usize,
    pub queued: usize,
    pub pressure_rejections: u64,
    pub queue_expirations: u64,
    pub benched: u64,
    pub cancelled: u64,
    pub faulted: bool,
}
#[derive(Clone)]
struct Proposer {
    name: String,
    identity: Identity,
    ledger: DraftLedger,
    last_probe: u64,
    recent_acceptance: bool,
}
struct Pending {
    proposer: ProposerId,
    parent: BlockKey,
    maximum: usize,
    continuation: bool,
    enqueued: u64,
    reason: DraftReason,
    running: bool,
    key: Option<BlockKey>,
    bound: Option<usize>,
}
pub(super) struct Drafts {
    policy: DraftPolicy,
    proposers: BTreeMap<ProposerId, Proposer>,
    pending: BTreeMap<DraftTicket, Pending>,
    baselines: BTreeMap<BlockKey, (u64, u64)>, // ns, committed tokens; exact identity scope
    rejected: u64,
    expired: u64,
    benched: u64,
    cancelled: u64,
    faulted: bool,
}
/// Single-use lease. Retain through native completion, independently of the waiter.
pub struct DraftPermit {
    controller: Controller,
    ticket: Option<DraftTicket>,
    pub reason: DraftReason,
}
fn identity_valid(identity: &Identity) -> Result<()> {
    ensure!(
        [
            &identity.model,
            &identity.tokenizer,
            &identity.quantization,
            &identity.namespace
        ]
        .iter()
        .all(|s| !s.is_empty() && s.len() <= 256),
        "draft identity bounds"
    );
    Ok(())
}
/// Same Spec 15 radix key, including partial tail, as the declared prefix cache.
pub fn prefix_key(identity: &Identity, tokens: &[u32]) -> Result<BlockKey> {
    identity_valid(identity)?;
    ensure!(
        !tokens.is_empty() && tokens.len() <= 65536,
        "draft prefix bounds"
    );
    Ok(path(identity, tokens).last().expect("nonempty prefix").key)
}
impl Drafts {
    pub(super) fn active(&self) -> usize {
        self.pending.values().filter(|t| t.running).count()
    }
    pub(super) fn queued(&self) -> usize {
        self.pending.len() - self.active()
    }
    fn profitable(&self, p: &Proposer) -> bool {
        let l = &p.ledger;
        self.baselines
            .get(&key(&p.identity, None, &[]))
            .is_some_and(|&(ns, tokens)| {
                l.window_committed > 0
                    && u128::from(l.window_cost_ns) * u128::from(tokens)
                        < u128::from(ns) * u128::from(l.window_committed)
            })
    }
    fn reason(
        &self,
        id: ProposerId,
        epoch: u64,
        exclude: Option<DraftTicket>,
    ) -> Option<DraftReason> {
        if self.faulted {
            return None;
        }
        let p = &self.proposers[&id];
        if p.ledger.mass >= self.policy.admission_mass {
            if p.ledger.window_observations < self.policy.qualification_trials {
                return Some(DraftReason::Qualify);
            }
            if self.profitable(p) {
                return Some(DraftReason::Earned);
            }
        }
        if epoch.saturating_sub(p.last_probe) >= self.policy.probe_every_epochs
            && !self
                .pending
                .iter()
                .any(|(ticket, t)| Some(*ticket) != exclude && t.proposer == id)
        {
            Some(DraftReason::Probe)
        } else {
            None
        }
    }
    fn qualification_available(&self, id: ProposerId) -> bool {
        let reserved = self
            .pending
            .values()
            .filter(|t| t.proposer == id && t.running && t.reason == DraftReason::Qualify)
            .count() as u64;
        self.proposers[&id]
            .ledger
            .window_observations
            .saturating_add(reserved)
            < self.policy.qualification_trials
    }
    fn start(&mut self, id: DraftTicket, epoch: u64) -> DraftReason {
        let ticket = self.pending.get_mut(&id).expect("ready ticket");
        ticket.running = true;
        if ticket.reason == DraftReason::Probe {
            self.proposers.get_mut(&ticket.proposer).unwrap().last_probe = epoch;
        }
        ticket.reason
    }
    pub(super) fn expire(&mut self, epoch: u64, wait: u64) {
        let ids: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, t)| !t.running && epoch - t.enqueued >= wait)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.pending.remove(&id);
            match self.expired.checked_add(1) {
                Some(v) => self.expired = v,
                None => self.faulted = true,
            }
        }
    }
    fn first(&self) -> Option<DraftTicket> {
        self.pending
            .iter()
            .filter(|(_, t)| !t.running)
            .min_by_key(|(id, t)| (!t.continuation, **id))
            .map(|(id, _)| *id)
    }
    pub(super) fn audit(&mut self, pressure: Pressure, config: &Config) -> Result<()> {
        for p in self.proposers.values_mut() {
            let raw = if matches!(pressure, Pressure::Critical) {
                config.rates_q31[2]
            } else if p.recent_acceptance {
                config.rates_q31[0]
            } else {
                config.rates_q31[1]
            };
            let rate = rounded(
                u128::from(raw) * u128::from(config.pressure_multipliers_q31[pressure.index()]),
            )
            .min(u128::from(MAX_MASS)) as u32;
            p.ledger.mass = decay(p.ledger.mass, rate)?;
            p.recent_acceptance = false;
        }
        Ok(())
    }
}
impl Controller {
    pub fn configure_drafts(&self, policy: DraftPolicy) -> Result<()> {
        ensure!(
            (1..=64).contains(&policy.max_proposers)
                && (1..=3).contains(&policy.work_limit)
                && policy.initial_mass <= MAX_MASS
                && policy.admission_mass > 0
                && policy.admission_mass <= policy.initial_mass
                && policy.reward_mass <= MAX_MASS
                && (1..=1_000_000).contains(&policy.qualification_trials)
                && (1..=1_000_000).contains(&policy.probe_every_epochs),
            "draft policy bounds"
        );
        let mut s = self.state()?;
        ensure!(s.drafts.is_none(), "draft policy already configured");
        s.drafts = Some(Drafts {
            policy,
            proposers: BTreeMap::new(),
            pending: BTreeMap::new(),
            baselines: BTreeMap::new(),
            rejected: 0,
            expired: 0,
            benched: 0,
            cancelled: 0,
            faulted: false,
        });
        Ok(())
    }
    pub fn register_proposer(&self, name: String, identity: Identity) -> Result<ProposerId> {
        identity_valid(&identity)?;
        ensure!(
            !name.is_empty() && name.len() <= 128,
            "proposer name bounds"
        );
        let mut s = self.state()?;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        ensure!(
            d.proposers.len() < d.policy.max_proposers
                && !d
                    .proposers
                    .values()
                    .any(|p| p.name == name && p.identity == identity),
            "proposer duplicate/budget"
        );
        let id = ProposerId(s.owner, s.id()?);
        let d = s.drafts.as_mut().unwrap();
        let mass = d.policy.initial_mass;
        d.proposers.insert(
            id,
            Proposer {
                name,
                identity,
                ledger: DraftLedger {
                    mass,
                    ..DraftLedger::default()
                },
                last_probe: 0,
                recent_acceptance: false,
            },
        );
        Ok(id)
    }
    pub fn proposer_identity(&self, id: ProposerId) -> Result<Identity> {
        Ok(self
            .state()?
            .drafts
            .as_ref()
            .context("draft policy inactive")?
            .proposers
            .get(&id)
            .context("foreign proposer")?
            .identity
            .clone())
    }
    pub fn record_no_draft(&self, id: ProposerId, cost_ns: u64, committed: u64) -> Result<()> {
        ensure!(cost_ns > 0 && committed > 0, "baseline counters");
        let mut s = self.state()?;
        let d = s.drafts.as_mut().context("draft policy inactive")?;
        let p = d.proposers.get(&id).context("foreign proposer")?;
        let k = key(&p.identity, None, &[]);
        let (ns, tokens) = d.baselines.get(&k).copied().unwrap_or_default();
        let next = (
            ns.checked_add(cost_ns).context("baseline cost overflow")?,
            tokens
                .checked_add(committed)
                .context("baseline token overflow")?,
        );
        d.baselines.insert(k, next);
        Ok(())
    }
    /// Deterministic arbitration. Cold/probe opportunities precede earned work;
    /// then compare measured ns/token exactly, tie-breaking by registration ID.
    /// Selection is advisory; request_draft atomically rechecks admission.
    pub fn choose_proposer(&self, candidates: &[ProposerId]) -> Result<Option<ProposerId>> {
        let s = self.state()?;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        ensure!(
            !candidates.is_empty() && candidates.len() <= d.policy.max_proposers,
            "proposer selection bounds"
        );
        let first = d
            .proposers
            .get(&candidates[0])
            .context("foreign proposer")?;
        let mut seen = BTreeSet::new();
        let mut best: Option<(ProposerId, DraftReason)> = None;
        for &id in candidates {
            let p = d.proposers.get(&id).context("foreign proposer")?;
            ensure!(
                p.identity == first.identity && seen.insert(id),
                "mixed/duplicate candidates"
            );
            let Some(reason) = d.reason(id, s.epoch, None) else {
                continue;
            };
            if reason == DraftReason::Qualify && !d.qualification_available(id) {
                continue;
            }
            let priority = |r| match r {
                DraftReason::Qualify => 0,
                DraftReason::Probe => 1,
                DraftReason::Earned => 2,
            };
            let replace = best.is_none_or(|(old, old_reason)| {
                let a = &p.ledger;
                let b = &d.proposers[&old].ledger;
                let order = priority(reason).cmp(&priority(old_reason));
                let cost = (u128::from(a.window_cost_ns) * u128::from(b.window_committed))
                    .cmp(&(u128::from(b.window_cost_ns) * u128::from(a.window_committed)));
                order.is_lt() || (order.is_eq() && (cost.is_lt() || (cost.is_eq() && id < old)))
            });
            if replace {
                best = Some((id, reason));
            }
        }
        Ok(best.map(|(id, _)| id))
    }
    pub fn request_draft(
        &self,
        id: ProposerId,
        parent: BlockKey,
        maximum: usize,
        continuation: bool,
    ) -> Result<DraftAdmission> {
        ensure!((1..=8).contains(&maximum), "draft length budget");
        let mut s = self.state()?;
        let epoch = s.epoch;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        ensure!(d.proposers.contains_key(&id), "foreign proposer");
        ensure!(!d.faulted, "draft accounting fault latched");
        let Some(reason) = d.reason(id, epoch, None) else {
            let d = s.drafts.as_mut().unwrap();
            d.benched = d.benched.checked_add(1).context("bench counter overflow")?;
            return Ok(DraftAdmission::Benched);
        };
        let queued = s.tickets.values().filter(|t| !t.ready).count() + d.queued();
        let ready = d.active() < d.policy.work_limit
            && d.queued() == 0
            && (reason != DraftReason::Qualify || d.qualification_available(id));
        if !ready && queued >= s.config.queue_depth {
            let d = s.drafts.as_mut().unwrap();
            d.rejected = d
                .rejected
                .checked_add(1)
                .context("pressure counter overflow")?;
            bail!("pressure: draft admission queue full");
        }
        let ticket = DraftTicket(s.owner, s.id()?);
        let d = s.drafts.as_mut().unwrap();
        d.pending.insert(
            ticket,
            Pending {
                proposer: id,
                parent,
                maximum,
                continuation,
                enqueued: epoch,
                reason,
                running: false,
                key: None,
                bound: None,
            },
        );
        if ready {
            let reason = d.start(ticket, epoch);
            Ok(DraftAdmission::Ready(DraftPermit {
                controller: self.clone(),
                ticket: Some(ticket),
                reason,
            }))
        } else {
            Ok(DraftAdmission::Queued(ticket))
        }
    }
    pub fn poll_draft(&self, ticket: DraftTicket) -> Result<Option<DraftPermit>> {
        let mut s = self.state()?;
        let epoch = s.epoch;
        let d = s.drafts.as_mut().context("draft policy inactive")?;
        ensure!(
            d.pending.get(&ticket).is_some_and(|t| !t.running),
            "unknown/expired/running draft ticket"
        );
        ensure!(!d.faulted, "draft accounting fault latched");
        let id = d.pending[&ticket].proposer;
        let Some(reason) = d.reason(id, epoch, Some(ticket)) else {
            d.pending.remove(&ticket);
            d.benched = d.benched.checked_add(1).context("bench counter overflow")?;
            bail!("queued proposer now benched");
        };
        if d.active() >= d.policy.work_limit
            || d.first() != Some(ticket)
            || (reason == DraftReason::Qualify && !d.qualification_available(id))
        {
            return Ok(None);
        }
        d.pending.get_mut(&ticket).unwrap().reason = reason;
        let reason = d.start(ticket, epoch);
        Ok(Some(DraftPermit {
            controller: self.clone(),
            ticket: Some(ticket),
            reason,
        }))
    }
    pub fn cancel_draft(&self, ticket: DraftTicket) -> Result<()> {
        let mut s = self.state()?;
        let d = s.drafts.as_mut().context("draft policy inactive")?;
        ensure!(
            d.pending.get(&ticket).is_some_and(|t| !t.running),
            "unknown/running draft ticket"
        );
        d.pending.remove(&ticket);
        d.cancelled = d
            .cancelled
            .checked_add(1)
            .context("cancel counter overflow")?;
        Ok(())
    }
    pub fn draft_baseline(&self, id: ProposerId) -> Result<DraftBaseline> {
        let s = self.state()?;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        let p = d.proposers.get(&id).context("foreign proposer")?;
        let (cost_ns, committed) = d
            .baselines
            .get(&key(&p.identity, None, &[]))
            .copied()
            .unwrap_or_default();
        Ok(DraftBaseline { cost_ns, committed })
    }
    pub fn draft_ledger(&self, id: ProposerId) -> Result<DraftLedger> {
        let s = self.state()?;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        let p = d.proposers.get(&id).context("foreign proposer")?;
        let mut l = p.ledger.clone();
        l.profitable = d.profitable(p);
        Ok(l)
    }
    pub fn draft_stats(&self) -> Result<DraftStats> {
        let s = self.state()?;
        let d = s.drafts.as_ref().context("draft policy inactive")?;
        Ok(DraftStats {
            epoch: s.epoch,
            registered: d.proposers.len(),
            active: d.active(),
            queued: d.queued(),
            pressure_rejections: d.rejected,
            queue_expirations: d.expired,
            benched: d.benched,
            cancelled: d.cancelled,
            faulted: d.faulted,
        })
    }
}
impl DraftPermit {
    pub fn bind(&self, tokens: &[u32]) -> Result<BlockKey> {
        let mut s = self.controller.state()?;
        let d = s.drafts.as_mut().context("draft policy inactive")?;
        let t = d
            .pending
            .get_mut(&self.ticket.context("consumed draft permit")?)
            .context("missing draft permit")?;
        ensure!(
            t.running && t.key.is_none() && tokens.len() <= t.maximum,
            "draft binding bounds/reuse"
        );
        let k = key(&d.proposers[&t.proposer].identity, Some(t.parent), tokens);
        t.key = Some(k);
        t.bound = Some(tokens.len());
        Ok(k)
    }
    pub fn finish(mut self, outcome: DraftOutcome) -> Result<()> {
        ensure!(
            outcome.known_prefix <= 1
                && outcome.proposed <= 8 - outcome.known_prefix
                && outcome.accepted <= outcome.proposed
                && outcome.committed <= outcome.known_prefix + outcome.accepted + 1
                && outcome.target_calls <= 2
                && outcome.cost_ns > 0
                && (outcome.failed || outcome.target_calls > 0)
                && (!outcome.failed || (outcome.committed == 0 && outcome.accepted == 0)),
            "draft outcome geometry"
        );
        let mut s = self.controller.state()?;
        let d = s.drafts.as_mut().context("draft policy inactive")?;
        let id = self.ticket.context("consumed draft permit")?;
        let t = d.pending.get(&id).context("missing draft permit")?;
        ensure!(
            t.running
                && t.key.is_some()
                && t.bound == Some((outcome.known_prefix + outcome.proposed) as usize),
            "unbound draft outcome"
        );
        let p = d.proposers.get(&t.proposer).unwrap();
        let mut l = p.ledger.clone();
        macro_rules! add {
            ($field:ident,$value:expr) => {
                l.$field = l
                    .$field
                    .checked_add($value)
                    .context(concat!("draft ledger overflow: ", stringify!($field)))?;
            };
        }
        add!(observations, 1);
        add!(window_observations, 1);
        add!(window_cost_ns, outcome.cost_ns);
        add!(window_committed, outcome.committed);
        add!(known_prefix, outcome.known_prefix);
        add!(proposed, outcome.proposed);
        add!(accepted, outcome.accepted);
        add!(committed, outcome.committed);
        add!(target_calls, outcome.target_calls);
        add!(cost_ns, outcome.cost_ns);
        add!(failures, u64::from(outcome.failed));
        let reward = (u64::from(d.policy.reward_mass) * outcome.accepted)
            .checked_div(outcome.proposed)
            .unwrap_or(0);
        l.mass = (u64::from(l.mass) + reward).min(u64::from(MAX_MASS)) as u32;
        if outcome.failed {
            l.mass = 0;
        }
        let proposer = t.proposer;
        let p = d.proposers.get_mut(&proposer).unwrap();
        p.ledger = l;
        p.recent_acceptance |= outcome.accepted > 0;
        d.pending.remove(&id);
        self.ticket = None;
        Ok(())
    }
}
impl Drop for DraftPermit {
    fn drop(&mut self) {
        let Some(id) = self.ticket.take() else {
            return;
        };
        let mut s = self
            .controller
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(d) = s.drafts.as_mut() {
            if let Some(t) = d.pending.remove(&id) {
                let l = &mut d.proposers.get_mut(&t.proposer).unwrap().ledger;
                match l.unfinished.checked_add(1) {
                    Some(v) => l.unfinished = v,
                    None => d.faulted = true,
                };
                l.mass = 0;
                l.window_cost_ns = 0;
                l.window_committed = 0;
                l.window_observations = 0;
            }
        }
    }
}
