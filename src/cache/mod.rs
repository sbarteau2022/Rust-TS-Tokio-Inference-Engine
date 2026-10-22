//! Spec 15 controller. Opaque exact payloads; no pretend model attention or learned predictor.
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, MutexGuard},
};
static NEXT_CONTROLLER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
pub const ONE: u64 = 1 << 31;
pub const MAX_MASS: u32 = i32::MAX as u32;
pub type BlockKey = [u8; 32];
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub model: String,
    pub tokenizer: String,
    pub quantization: String,
    pub namespace: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub capacity_blocks: usize,
    pub bytes_per_block: usize,
    pub max_branches: usize,
    pub max_declared_tokens: usize,
    pub max_leases: usize,
    pub anchor_fraction_q31: u32,
    pub dead_threshold: u32,
    pub rates_q31: [u32; 3],
    pub pressure_multipliers_q31: [u64; 3],
    pub queue_depth: usize,
    pub max_wait_epochs: u64,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=4096).contains(&self.capacity_blocks)
                && (1..=1_048_576).contains(&self.bytes_per_block),
            "pool configuration"
        );
        ensure!(
            self.capacity_blocks
                .checked_mul(self.bytes_per_block)
                .is_some_and(|v| v <= 64 * 1024 * 1024),
            "pool byte budget"
        );
        ensure!(
            (1..=4096).contains(&self.max_branches)
                && (1..=1_048_576).contains(&self.max_declared_tokens)
                && (1..=64).contains(&self.max_leases),
            "metadata budget"
        );
        ensure!(
            self.anchor_fraction_q31 <= MAX_MASS && (1..=MAX_MASS).contains(&self.dead_threshold),
            "mass configuration"
        );
        ensure!(
            self.rates_q31.iter().all(|r| *r <= MAX_MASS)
                && self.rates_q31.windows(2).all(|w| w[0] <= w[1]),
            "rate classes"
        );
        ensure!(
            self.pressure_multipliers_q31[0] == ONE
                && self
                    .pressure_multipliers_q31
                    .windows(2)
                    .all(|w| w[0] <= w[1])
                && self.pressure_multipliers_q31[2] <= 8 * ONE,
            "pressure table"
        );
        ensure!(
            (1..=128).contains(&self.queue_depth)
                && (1..=1_000_000).contains(&self.max_wait_epochs),
            "queue configuration"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct BranchId(u64, u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TicketId(u64, u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Tier {
    Anchored,
    Covered,
    OffPlan,
    Dead,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Pressure {
    Normal,
    Elevated,
    Critical,
}
impl Pressure {
    fn index(self) -> usize {
        match self {
            Self::Normal => 0,
            Self::Elevated => 1,
            Self::Critical => 2,
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Demand {
    pub key: BlockKey,
    pub parent: Option<BlockKey>,
    pub tokens: Vec<u32>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Audit {
    pub epoch: u64,
    pub declared_tokens: usize,
    pub resident_declared_tokens: usize,
    pub unknown_output_estimate_bytes: u64,
    pub missing: Vec<Demand>,
    pub anchored_blocks: usize,
    pub anchor_degradations: usize,
    pub freed_blocks: usize,
    pub anchor_budget_bytes: usize,
    pub released_anchors: usize,
    pub max_released_anchor_age_epochs: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub epoch: u64,
    pub resident_blocks: usize,
    pub resident_bytes: usize,
    pub reserved_blocks: usize,
    pub open_branches: usize,
    pub active_leases: usize,
    pub active_prefills: usize,
    pub prefill_snapshot_bytes: usize,
    pub queued: usize,
    pub ready: usize,
    pub pressure_rejections: u64,
    pub queue_expirations: u64,
    pub anchor_degradations: u64,
    pub freed_blocks: u64,
    pub production_qualified: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct BlockInfo {
    pub key: BlockKey,
    pub parent: Option<BlockKey>,
    pub references: usize,
    pub locks: usize,
    pub anchored: bool,
    pub mass: u32,
    pub rate: u32,
    pub tier: Tier,
    pub last_turn_used: u64,
}
struct Block {
    demand: Demand,
    identity: Identity,
    payload: Vec<u8>,
    references: usize,
    locks: usize,
    anchored: bool,
    mass: u32,
    rate: u32,
    tier: Tier,
    last_turn_used: u64,
    anchor_since: Option<u64>,
}
struct Branch {
    parent: Option<BranchId>,
    identity: Identity,
    tokens: Vec<u32>,
    path: Vec<Demand>,
    closing: bool,
    unknown_reserved: usize,
}
struct Ticket {
    running: bool,
    branch: BranchId,
    enqueued: u64,
    ready: bool,
    reserved: usize,
}
struct State {
    config: Config,
    epoch: u64,
    next: u64,
    owner: u64,
    blocks: BTreeMap<BlockKey, Block>,
    branches: BTreeMap<BranchId, Branch>,
    tickets: BTreeMap<TicketId, Ticket>,
    active: usize,
    prefills: usize,
    snapshot_bytes: usize,
    rejected: u64,
    expired: u64,
    degraded: u64,
    freed: u64,
}
#[derive(Clone)]
pub struct Controller {
    inner: Arc<Mutex<State>>,
}
/// Locks are released on drop; frees still require an explicit quiescent boundary.
pub struct StepLease {
    controller: Controller,
    keys: Vec<BlockKey>,
}
impl Drop for StepLease {
    fn drop(&mut self) {
        let mut state = self
            .controller
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for key in &self.keys {
            let block = state
                .blocks
                .get_mut(key)
                .expect("leased block cannot be reclaimed");
            block.locks = block.locks.checked_sub(1).expect("in-use lock underflow");
        }
        state.active = state.active.checked_sub(1).expect("step lease underflow");
    }
}
struct PrefillRun {
    controller: Controller,
    ticket: TicketId,
    snapshot_bytes: usize,
}
impl Drop for PrefillRun {
    fn drop(&mut self) {
        let mut state = self
            .controller
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.prefills = state
            .prefills
            .checked_sub(1)
            .expect("prefill count underflow");
        state.snapshot_bytes = state
            .snapshot_bytes
            .checked_sub(self.snapshot_bytes)
            .expect("prefill snapshot underflow");
        if let Some(ticket) = state.tickets.get_mut(&self.ticket) {
            ticket.running = false;
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub enum Admission {
    Ready(TicketId),
    Queued(TicketId),
}
#[derive(Debug, Clone)]
pub struct PrefillRequest {
    pub branch: BranchId,
    pub tokens: Vec<u32>,
    pub missing: Vec<Demand>,
    pub resident_prefix: Vec<Vec<u8>>,
}
fn rounded(value: u128) -> u128 {
    let quotient = value >> 31;
    let remainder = value & ((ONE - 1) as u128);
    quotient
        + u128::from(
            remainder > (ONE / 2) as u128 || (remainder == (ONE / 2) as u128 && quotient & 1 == 1),
        )
}
pub fn decay(mass: u32, rate: u32) -> Result<u32> {
    ensure!(
        mass <= MAX_MASS && rate <= MAX_MASS,
        "nonnegative Q1.31 required"
    );
    Ok(
        rounded(u128::from(mass) * u128::from(ONE - u64::from(rate))).min(u128::from(MAX_MASS))
            as u32,
    )
}
fn key(identity: &Identity, parent: Option<BlockKey>, tokens: &[u32]) -> BlockKey {
    let mut digest = Sha256::new();
    digest.update(b"rust-server/spec15/v1");
    for value in [
        &identity.model,
        &identity.tokenizer,
        &identity.quantization,
        &identity.namespace,
    ] {
        digest.update((value.len() as u32).to_le_bytes());
        digest.update(value.as_bytes());
    }
    digest.update([u8::from(parent.is_some())]);
    digest.update(parent.unwrap_or([0; 32]));
    digest.update((tokens.len() as u32).to_le_bytes());
    for token in tokens {
        digest.update(token.to_le_bytes());
    }
    digest.finalize().into()
}
fn path(identity: &Identity, tokens: &[u32]) -> Vec<Demand> {
    let mut parent = None;
    tokens
        .chunks(16)
        .map(|tokens| {
            let value = key(identity, parent, tokens);
            let demand = Demand {
                key: value,
                parent,
                tokens: tokens.to_vec(),
            };
            parent = Some(value);
            demand
        })
        .collect()
}
impl State {
    fn id(&mut self) -> Result<u64> {
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("identifier exhaustion"))?;
        Ok(self.next)
    }
    fn reserved(&self) -> usize {
        self.tickets.values().map(|t| t.reserved).sum::<usize>()
            + self
                .branches
                .values()
                .map(|b| b.unknown_reserved)
                .sum::<usize>()
    }
    fn missing(&self, branch: BranchId) -> Result<Vec<Demand>> {
        let branch = self
            .branches
            .get(&branch)
            .ok_or_else(|| anyhow::anyhow!("unknown branch"))?;
        ensure!(!branch.closing, "branch closing");
        Ok(branch
            .path
            .iter()
            .filter(|d| !self.blocks.contains_key(&d.key))
            .cloned()
            .collect())
    }
    fn in_progress(&self, branch: BranchId) -> bool {
        let pending: BTreeSet<_> = self
            .tickets
            .values()
            .filter(|t| t.ready)
            .flat_map(|t| {
                self.branches[&t.branch]
                    .path
                    .iter()
                    .filter(|d| !self.blocks.contains_key(&d.key))
                    .map(|d| d.key)
            })
            .collect();
        self.branches[&branch]
            .path
            .iter()
            .any(|d| pending.contains(&d.key))
    }
    fn available(&self) -> usize {
        self.config.capacity_blocks - self.blocks.len() - self.reserved()
    }
    fn process_queue(&mut self) {
        let expired: Vec<_> = self
            .tickets
            .iter()
            .filter(|(_, t)| !t.ready && self.epoch - t.enqueued >= self.config.max_wait_epochs)
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            self.tickets.remove(&id);
            self.expired += 1;
        }
        let mut queued: Vec<_> = self
            .tickets
            .iter()
            .filter(|(_, t)| !t.ready)
            .map(|(id, t)| (self.branches[&t.branch].parent.is_none(), *id))
            .collect();
        queued.sort();
        for (_, id) in queued {
            let branch = self.tickets[&id].branch;
            let needed = self.missing(branch).expect("queued open branch").len();
            if needed > self.available() || self.in_progress(branch) {
                break;
            }
            let ticket = self.tickets.get_mut(&id).unwrap();
            ticket.ready = true;
            ticket.reserved = needed;
        }
    }
    fn boundary(&mut self) -> Result<usize> {
        ensure!(
            self.active == 0,
            "boundary requires all native step leases completed"
        );
        let closed: Vec<_> = self
            .branches
            .iter()
            .filter(|(_, b)| b.closing)
            .map(|(id, _)| *id)
            .collect();
        let mut decrements = BTreeMap::<BlockKey, usize>::new();
        for id in &closed {
            for demand in &self.branches[id].path {
                if self.blocks.contains_key(&demand.key) {
                    *decrements.entry(demand.key).or_default() += 1;
                }
            }
        }
        ensure!(
            decrements
                .iter()
                .all(|(key, n)| self.blocks[key].references >= *n),
            "cache reference underflow invariant"
        );
        for id in closed {
            let branch = self.branches.remove(&id).unwrap();
            for demand in branch.path {
                if let Some(block) = self.blocks.get_mut(&demand.key) {
                    block.references = block
                        .references
                        .checked_sub(1)
                        .expect("cache reference underflow");
                }
            }
        }
        let parents: BTreeSet<_> = self
            .blocks
            .values()
            .filter_map(|b| b.demand.parent)
            .collect();
        let dead: Vec<_> = self
            .blocks
            .iter()
            .filter(|(key, b)| {
                !parents.contains(*key)
                    && b.references == 0
                    && b.locks == 0
                    && !b.anchored
                    && b.mass < self.config.dead_threshold
            })
            .map(|(key, _)| *key)
            .collect();
        for key in &dead {
            self.blocks.remove(key);
        }
        self.freed += dead.len() as u64;
        self.process_queue();
        Ok(dead.len())
    }
}
impl Controller {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(State {
                config,
                epoch: 0,
                next: 0,
                owner: NEXT_CONTROLLER
                    .try_update(
                        std::sync::atomic::Ordering::SeqCst,
                        std::sync::atomic::Ordering::SeqCst,
                        |n| n.checked_add(1),
                    )
                    .map_err(|_| anyhow::anyhow!("controller identifier exhaustion"))?,
                blocks: BTreeMap::new(),
                branches: BTreeMap::new(),
                tickets: BTreeMap::new(),
                active: 0,
                prefills: 0,
                snapshot_bytes: 0,
                rejected: 0,
                expired: 0,
                degraded: 0,
                freed: 0,
            })),
        })
    }
    fn state(&self) -> Result<MutexGuard<'_, State>> {
        self.inner
            .lock()
            .map_err(|_| anyhow::anyhow!("cache controller poisoned"))
    }
    /// Root tokens are the whole context; child tokens append to the parent's context.
    pub fn branch(
        &self,
        parent: Option<BranchId>,
        identity: Identity,
        append: Vec<u32>,
    ) -> Result<BranchId> {
        let mut state = self.state()?;
        ensure!(append.len() <= 65536, "declared token budget");
        ensure!(
            [
                &identity.model,
                &identity.tokenizer,
                &identity.quantization,
                &identity.namespace
            ]
            .iter()
            .all(|s| !s.is_empty() && s.len() <= 256),
            "identity fields must be nonempty and bounded"
        );
        ensure!(
            state.branches.len() < state.config.max_branches,
            "branch budget"
        );
        let mut tokens = if let Some(parent) = parent {
            let branch = state
                .branches
                .get(&parent)
                .ok_or_else(|| anyhow::anyhow!("unknown parent"))?;
            ensure!(
                !branch.closing && branch.identity == identity,
                "parent closed or identity mismatch"
            );
            branch.tokens.clone()
        } else {
            Vec::new()
        };
        tokens.extend(append);
        let declared: usize = state.branches.values().map(|b| b.tokens.len()).sum();
        ensure!(
            !tokens.is_empty()
                && tokens.len() <= 65536
                && declared + tokens.len() <= state.config.max_declared_tokens,
            "declared token budget"
        );
        let path = path(&identity, &tokens);
        // A partial inherited tail is replaced, never mutated. Reject a write-fork while in use.
        if let Some(parent) = parent {
            let prior = &state.branches[&parent];
            if prior.tokens.len() % 16 != 0 && prior.tokens.len() != tokens.len() {
                let tail = prior.path.last().unwrap();
                ensure!(
                    state.blocks.get(&tail.key).is_none_or(|b| b.locks == 0),
                    "mutable tail in use"
                );
            }
        }
        for demand in &path {
            if let Some(block) = state.blocks.get(&demand.key) {
                ensure!(
                    block.identity == identity
                        && block.demand.tokens == demand.tokens
                        && block.demand.parent == demand.parent,
                    "digest identity collision"
                );
            }
        }
        let id = BranchId(state.owner, state.id()?);
        for demand in &path {
            if let Some(block) = state.blocks.get_mut(&demand.key) {
                block.references = block
                    .references
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("reference overflow"))?;
            }
        }
        state.branches.insert(
            id,
            Branch {
                parent,
                identity,
                tokens,
                path,
                closing: false,
                unknown_reserved: 0,
            },
        );
        Ok(id)
    }
    /// Reserve capacity for unknown outputs without inventing tokens or triggering prefill.
    pub fn reserve_unknown(&self, branch: BranchId, estimated_bytes: usize) -> Result<usize> {
        let mut state = self.state()?;
        let owner = state
            .branches
            .get(&branch)
            .ok_or_else(|| anyhow::anyhow!("unknown branch"))?;
        ensure!(!owner.closing, "branch closing");
        let current = owner.unknown_reserved;
        let blocks = estimated_bytes.div_ceil(state.config.bytes_per_block);
        if blocks > state.available() + current
            || (blocks > current && state.tickets.values().any(|t| !t.ready))
        {
            state.rejected += 1;
            bail!("pressure: unknown-output reservation unavailable");
        }
        state.branches.get_mut(&branch).unwrap().unknown_reserved = blocks;
        state.process_queue();
        Ok(blocks)
    }
    pub fn request_prefill(&self, branch: BranchId) -> Result<Admission> {
        let mut state = self.state()?;
        let needed = state.missing(branch)?.len();
        ensure!(
            !state.tickets.values().any(|t| t.branch == branch),
            "branch already has admission ticket"
        );
        if needed > state.config.capacity_blocks {
            state.rejected += 1;
            bail!("pressure: context exceeds pool");
        }
        // Existing queued work cannot be bypassed by a new direct admission.
        let queued = state.tickets.values().filter(|t| !t.ready).count();
        let ready = (needed == 0 || queued == 0)
            && needed <= state.available()
            && !state.in_progress(branch);
        if !ready && queued >= state.config.queue_depth {
            state.rejected += 1;
            bail!("pressure: admission queue full");
        }
        // Bound ready zero-reservation tickets as well as queued tickets.
        ensure!(
            state.tickets.len() < state.config.max_branches,
            "ticket budget"
        );
        let id = TicketId(state.owner, state.id()?);
        let epoch = state.epoch;
        state.tickets.insert(
            id,
            Ticket {
                running: false,
                branch,
                enqueued: epoch,
                ready,
                reserved: if ready { needed } else { 0 },
            },
        );
        state.process_queue();
        Ok(if state.tickets[&id].ready {
            Admission::Ready(id)
        } else {
            Admission::Queued(id)
        })
    }
    pub fn ready(&self) -> Result<Vec<TicketId>> {
        Ok(self
            .state()?
            .tickets
            .iter()
            .filter(|(_, t)| t.ready)
            .map(|(id, _)| *id)
            .collect())
    }
    pub fn cancel_prefill(&self, ticket: TicketId) -> Result<()> {
        let mut state = self.state()?;
        ensure!(
            state.tickets.remove(&ticket).is_some(),
            "unknown or expired ticket"
        );
        state.process_queue();
        Ok(())
    }
    pub fn fulfill<F>(&self, ticket: TicketId, producer: F) -> Result<()>
    where
        F: FnOnce(PrefillRequest) -> Result<Vec<Vec<u8>>>,
    {
        let (request, _running) = {
            let mut state = self.state()?;
            let item = state
                .tickets
                .get(&ticket)
                .ok_or_else(|| anyhow::anyhow!("unknown or expired ticket"))?;
            ensure!(
                item.ready && !item.running,
                "prefill queued or already executing"
            );
            ensure!(
                state.prefills < state.config.max_leases,
                "prefill worker budget"
            );
            let branch_id = item.branch;
            let branch = &state.branches[&branch_id];
            let snapshot_bytes = branch
                .path
                .iter()
                .filter(|d| state.blocks.contains_key(&d.key))
                .count()
                * state.config.bytes_per_block;
            ensure!(
                state.snapshot_bytes + snapshot_bytes
                    <= state.config.capacity_blocks * state.config.bytes_per_block,
                "prefill snapshot budget"
            );
            let request = PrefillRequest {
                branch: branch_id,
                tokens: branch.tokens.clone(),
                missing: state.missing(branch_id)?,
                resident_prefix: branch
                    .path
                    .iter()
                    .filter_map(|d| state.blocks.get(&d.key).map(|b| b.payload.clone()))
                    .collect(),
            };
            state.prefills += 1;
            state.snapshot_bytes += snapshot_bytes;
            state.tickets.get_mut(&ticket).unwrap().running = true;
            (
                request,
                PrefillRun {
                    controller: self.clone(),
                    ticket,
                    snapshot_bytes,
                },
            )
        };
        let branch_id = request.branch;
        let missing = request.missing.clone();
        let payloads = if missing.is_empty() {
            Vec::new()
        } else {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| producer(request))) {
                Ok(Ok(payloads)) => payloads,
                Ok(Err(error)) => {
                    let _ = self.cancel_prefill(ticket);
                    return Err(error);
                }
                Err(_) => {
                    let _ = self.cancel_prefill(ticket);
                    bail!("prefill producer panic isolated")
                }
            }
        };
        let result = (|| -> Result<()> {
            let mut state = self.state()?;
            let item = state
                .tickets
                .get(&ticket)
                .ok_or_else(|| anyhow::anyhow!("ticket canceled while producer ran"))?;
            ensure!(
                item.ready && item.branch == branch_id,
                "ticket state changed"
            );
            let identity = state.branches[&branch_id].identity.clone();
            ensure!(
                payloads.len() == missing.len()
                    && payloads
                        .iter()
                        .all(|p| p.len() == state.config.bytes_per_block),
                "prefill payload shape"
            );
            for (demand, payload) in missing.iter().zip(&payloads) {
                if let Some(block) = state.blocks.get(&demand.key) {
                    ensure!(
                        block.payload == *payload,
                        "conflicting exact-prefix payload"
                    );
                }
            }
            for (demand, payload) in missing.into_iter().zip(payloads) {
                if state.blocks.contains_key(&demand.key) {
                    continue;
                }
                ensure!(
                    demand.parent.is_none_or(|p| state.blocks.contains_key(&p)),
                    "missing prefix ancestor"
                );
                let references = state
                    .branches
                    .values()
                    .filter(|b| b.path.iter().any(|d| d.key == demand.key))
                    .count();
                let epoch = state.epoch;
                state.blocks.insert(
                    demand.key,
                    Block {
                        demand,
                        identity: identity.clone(),
                        payload,
                        references,
                        locks: 0,
                        anchored: false,
                        mass: MAX_MASS,
                        rate: 0,
                        tier: Tier::OffPlan,
                        last_turn_used: epoch,
                        anchor_since: None,
                    },
                );
            }
            state.tickets.remove(&ticket);
            state.process_queue();
            Ok(())
        })();
        if result.is_err() {
            let _ = self.cancel_prefill(ticket);
        }
        result
    }
    pub fn lease(&self, branch: BranchId) -> Result<StepLease> {
        let mut state = self.state()?;
        ensure!(state.active < state.config.max_leases, "step lease budget");
        ensure!(
            state.missing(branch)?.is_empty(),
            "context needs declared prefill"
        );
        let keys: Vec<_> = state.branches[&branch].path.iter().map(|d| d.key).collect();
        let epoch = state.epoch;
        for key in &keys {
            let block = state.blocks.get_mut(key).unwrap();
            block.locks += 1;
            block.last_turn_used = epoch;
        }
        state.active += 1;
        Ok(StepLease {
            controller: self.clone(),
            keys,
        })
    }
    pub fn read(&self, lease: &StepLease) -> Result<Vec<Vec<u8>>> {
        ensure!(
            Arc::ptr_eq(&self.inner, &lease.controller.inner),
            "foreign cache lease"
        );
        let state = self.state()?;
        Ok(lease
            .keys
            .iter()
            .map(|key| state.blocks[key].payload.clone())
            .collect())
    }
    pub fn close_subtree(&self, branch: BranchId) -> Result<()> {
        let mut state = self.state()?;
        ensure!(state.branches.contains_key(&branch), "unknown branch");
        let mut closing = BTreeSet::from([branch]);
        loop {
            let before = closing.len();
            for (id, b) in &state.branches {
                if b.parent.is_some_and(|p| closing.contains(&p)) {
                    closing.insert(*id);
                }
            }
            if closing.len() == before {
                break;
            }
        }
        for id in &closing {
            state.branches.get_mut(id).unwrap().closing = true;
        }
        state.tickets.retain(|_, t| !closing.contains(&t.branch));
        state.process_queue();
        Ok(())
    }
    pub fn boundary(&self) -> Result<usize> {
        self.state()?.boundary()
    }
    pub fn audit(
        &self,
        epoch: u64,
        required: &[BranchId],
        unknown_estimate_bytes: u64,
        pressure: Pressure,
    ) -> Result<Audit> {
        let mut state = self.state()?;
        ensure!(
            state.active == 0,
            "audit boundary requires completed native steps"
        );
        ensure!(
            state.epoch.checked_add(1) == Some(epoch),
            "audit epoch must advance exactly once"
        );
        let mut demands = BTreeMap::new();
        for id in required {
            let branch = state
                .branches
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("unknown plan branch"))?;
            ensure!(!branch.closing, "plan branch closing");
            for demand in &branch.path {
                demands.insert(demand.key, demand.clone());
            }
        }
        let declared_tokens = demands.values().map(|d| d.tokens.len()).sum();
        let resident_declared_tokens = demands
            .values()
            .filter(|d| state.blocks.contains_key(&d.key))
            .map(|d| d.tokens.len())
            .sum();
        let missing = demands
            .values()
            .filter(|d| !state.blocks.contains_key(&d.key))
            .cloned()
            .collect();
        state.epoch = epoch;
        let budget = (state.config.capacity_blocks as u64
            * u64::from(state.config.anchor_fraction_q31)
            / ONE) as usize;
        let mut covered: Vec<_> = demands
            .values()
            .filter(|d| state.blocks.contains_key(&d.key))
            .map(|d| d.key)
            .collect();
        if required.is_empty() {
            let roots: BTreeSet<_> = state
                .branches
                .values()
                .filter(|b| !b.closing)
                .filter_map(|b| b.path.first().map(|d| d.key))
                .collect();
            covered = roots
                .into_iter()
                .filter(|key| state.blocks.contains_key(key))
                .collect();
        }
        // Prefer shorter shared ancestry, deterministic key order for ties.
        covered.sort_by_key(|key| {
            let mut depth = 0;
            let mut parent = state.blocks[key].demand.parent;
            while let Some(p) = parent {
                depth += 1;
                parent = state.blocks[&p].demand.parent;
            }
            (depth, *key)
        });
        let anchors: BTreeSet<_> = covered.iter().take(budget).copied().collect();
        let degradations = covered.len().saturating_sub(budget);
        state.degraded += degradations as u64;
        let rates = state.config.rates_q31;
        let factor = state.config.pressure_multipliers_q31[pressure.index()];
        let threshold = state.config.dead_threshold;
        let mut released_anchors = 0;
        let mut max_released_anchor_age_epochs = 0;
        let anchor_budget_bytes = budget * state.config.bytes_per_block;
        for (key, block) in &mut state.blocks {
            let anchored = anchors.contains(key);
            if anchored && !block.anchored {
                block.anchor_since = Some(epoch);
            }
            if !anchored && block.anchored {
                released_anchors += 1;
                max_released_anchor_age_epochs = max_released_anchor_age_epochs
                    .max(epoch - block.anchor_since.take().expect("anchored epoch"));
            }
            block.anchored = anchored;
            let covered = demands.contains_key(key);
            let raw = if covered {
                rates[0]
            } else if matches!(pressure, Pressure::Critical) {
                rates[2]
            } else {
                rates[1]
            };
            block.rate = if block.anchored {
                0
            } else {
                rounded(u128::from(raw) * u128::from(factor)).min(u128::from(MAX_MASS)) as u32
            };
            block.mass = decay(block.mass, block.rate)?;
            block.tier = if block.anchored {
                Tier::Anchored
            } else if covered {
                Tier::Covered
            } else if block.mass < threshold {
                Tier::Dead
            } else {
                Tier::OffPlan
            };
        }
        let freed = state.boundary()?;
        Ok(Audit {
            epoch,
            declared_tokens,
            resident_declared_tokens,
            unknown_output_estimate_bytes: unknown_estimate_bytes,
            missing,
            anchored_blocks: anchors.len(),
            anchor_degradations: degradations,
            freed_blocks: freed,
            anchor_budget_bytes,
            released_anchors,
            max_released_anchor_age_epochs,
        })
    }
    pub fn blocks(&self) -> Result<Vec<BlockInfo>> {
        Ok(self
            .state()?
            .blocks
            .values()
            .map(|b| BlockInfo {
                key: b.demand.key,
                parent: b.demand.parent,
                references: b.references,
                locks: b.locks,
                anchored: b.anchored,
                mass: b.mass,
                rate: b.rate,
                tier: b.tier,
                last_turn_used: b.last_turn_used,
            })
            .collect())
    }
    pub fn stats(&self) -> Result<Stats> {
        let state = self.state()?;
        Ok(Stats {
            epoch: state.epoch,
            resident_blocks: state.blocks.len(),
            resident_bytes: state.blocks.len() * state.config.bytes_per_block,
            reserved_blocks: state.reserved(),
            open_branches: state.branches.values().filter(|b| !b.closing).count(),
            active_leases: state.active,
            active_prefills: state.prefills,
            prefill_snapshot_bytes: state.snapshot_bytes,
            queued: state.tickets.values().filter(|t| !t.ready).count(),
            ready: state.tickets.values().filter(|t| t.ready).count(),
            pressure_rejections: state.rejected,
            queue_expirations: state.expired,
            anchor_degradations: state.degraded,
            freed_blocks: state.freed,
            production_qualified: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_underflow_is_a_named_transactional_refusal() {
        let config: Config =
            serde_json::from_str(include_str!("../../fixtures/cache-config-bench.json")).unwrap();
        let cache = Controller::new(config).unwrap();
        let branch = cache
            .branch(
                None,
                Identity {
                    model: "test".into(),
                    tokenizer: "test".into(),
                    quantization: "test".into(),
                    namespace: "private".into(),
                },
                vec![1; 16],
            )
            .unwrap();
        let ticket = match cache.request_prefill(branch).unwrap() {
            Admission::Ready(id) => id,
            _ => unreachable!(),
        };
        cache
            .fulfill(ticket, |r| {
                Ok(r.missing.iter().map(|_| vec![0; 64]).collect())
            })
            .unwrap();
        cache.close_subtree(branch).unwrap();
        cache
            .state()
            .unwrap()
            .blocks
            .values_mut()
            .next()
            .unwrap()
            .references = 0;
        assert!(cache
            .boundary()
            .unwrap_err()
            .to_string()
            .contains("reference underflow"));
        assert_eq!(cache.state().unwrap().branches.len(), 1);
    }
}
