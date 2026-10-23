use rust_server::cache::*;
fn cache(limit: usize) -> Controller {
    let c = Controller::new(
        serde_json::from_str(include_str!("../fixtures/cache-config-bench.json")).unwrap(),
    )
    .unwrap();
    c.configure_drafts(DraftPolicy {
        max_proposers: 4,
        work_limit: limit,
        initial_mass: MAX_MASS,
        admission_mass: MAX_MASS / 8,
        reward_mass: MAX_MASS / 4,
        qualification_trials: 1,
        probe_every_epochs: 2,
    })
    .unwrap();
    c
}
fn identity() -> Identity {
    Identity {
        model: "weights@revision".into(),
        tokenizer: "tokenizer+template@revision".into(),
        quantization: "kv-f32-regular".into(),
        namespace: "private:test".into(),
    }
}
fn ready(a: DraftAdmission) -> DraftPermit {
    match a {
        DraftAdmission::Ready(p) => p,
        _ => panic!("ready required"),
    }
}
fn queued(a: DraftAdmission) -> DraftTicket {
    match a {
        DraftAdmission::Queued(t) => t,
        _ => panic!("queue required"),
    }
}
fn complete(p: DraftPermit, ns: u64, tokens: u64) {
    p.bind(&[2, 3]).unwrap();
    p.finish(DraftOutcome {
        known_prefix: 0,
        proposed: 2,
        accepted: 2,
        committed: tokens,
        target_calls: 2,
        cost_ns: ns,
        failed: false,
    })
    .unwrap();
}
#[test]
fn measured_profit_and_identity_not_acceptance_authorize_work() {
    let c = cache(1);
    let id = c.register_proposer("retrieval".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1; 17]).unwrap();
    c.record_no_draft(id, 100, 1).unwrap();
    complete(ready(c.request_draft(id, parent, 2, true).unwrap()), 201, 2);
    assert!(!c.draft_ledger(id).unwrap().profitable);
    assert!(matches!(
        c.request_draft(id, parent, 2, true).unwrap(),
        DraftAdmission::Benched
    ));
    c.audit(1, &[], 999, Pressure::Normal).unwrap();
    assert!(matches!(
        c.request_draft(id, parent, 2, true).unwrap(),
        DraftAdmission::Benched
    ));
    c.audit(2, &[], 999, Pressure::Normal).unwrap();
    let probe = ready(c.request_draft(id, parent, 2, true).unwrap());
    assert_eq!(probe.reason, DraftReason::Probe);
    complete(probe, 1, 3);
    assert!(c.draft_ledger(id).unwrap().profitable);
    let earned = ready(c.request_draft(id, parent, 2, true).unwrap());
    assert_eq!(earned.reason, DraftReason::Earned);
    complete(earned, 1, 3);
    let mut other = identity();
    other.namespace.push('2');
    let p = c.register_proposer("mtp".into(), other.clone()).unwrap();
    complete(ready(c.request_draft(p, parent, 2, true).unwrap()), 1, 3);
    assert!(!c.draft_ledger(p).unwrap().profitable); // Baseline cannot cross namespaces.
    for field in 0..4 {
        let mut i = identity();
        match field {
            0 => i.model.push('x'),
            1 => i.tokenizer.push('x'),
            2 => i.quantization.push('x'),
            _ => i.namespace.push('x'),
        };
        assert_ne!(prefix_key(&i, &[1; 17]).unwrap(), parent);
    }
    assert_ne!(prefix_key(&identity(), &[1; 16]).unwrap(), parent);
}
#[test]
fn continuation_queue_expiry_cancellation_and_epoch_barrier() {
    let c = cache(1);
    let id = c.register_proposer("p".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    let active = ready(c.request_draft(id, parent, 2, true).unwrap());
    let root = queued(c.request_draft(id, parent, 2, false).unwrap());
    let continuation = queued(c.request_draft(id, parent, 2, true).unwrap());
    assert!(c.poll_draft(root).unwrap().is_none());
    assert!(c.audit(1, &[], 0, Pressure::Normal).is_err());
    complete(active, 100, 2);
    // No baseline: queued qualification re-evaluates to benched, with explicit error.
    assert!(c.poll_draft(continuation).is_err());
    c.cancel_draft(root).unwrap();
    assert!(c.cancel_draft(root).is_err());
    c.audit(1, &[], 0, Pressure::Normal).unwrap();
    c.audit(2, &[], 0, Pressure::Normal).unwrap();
    let probe = ready(c.request_draft(id, parent, 2, true).unwrap());
    complete(probe, 100, 2);
    // Separate cold proposers test priority while qualification remains eligible.
    let a = c.register_proposer("a".into(), identity()).unwrap();
    let b = c.register_proposer("b".into(), identity()).unwrap();
    let holder = ready(c.request_draft(a, parent, 2, true).unwrap());
    let root = queued(c.request_draft(b, parent, 2, false).unwrap());
    let continuation = queued(c.request_draft(b, parent, 2, true).unwrap());
    complete(holder, 100, 2);
    assert!(c.poll_draft(root).unwrap().is_none());
    let admitted = c.poll_draft(continuation).unwrap().unwrap();
    complete(admitted, 100, 2);
    assert!(c.poll_draft(continuation).is_err());
    // Expiry only at audit epochs, not wall-clock or poll count.
    for epoch in 3..=5 {
        c.audit(epoch, &[], 0, Pressure::Normal).unwrap();
    }
    assert!(c.poll_draft(root).is_err());
    assert_eq!(c.draft_stats().unwrap().queue_expirations, 1);
}
#[test]
fn bounded_pressure_abandonment_and_exact_decay() {
    let c = cache(1);
    let id = c.register_proposer("p".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    let hold = ready(c.request_draft(id, parent, 2, true).unwrap());
    let mut queued_ids = Vec::new();
    for _ in 0..4 {
        queued_ids.push(queued(c.request_draft(id, parent, 2, true).unwrap()));
    }
    assert!(c.request_draft(id, parent, 2, true).is_err());
    assert_eq!(c.draft_stats().unwrap().pressure_rejections, 1);
    drop(hold);
    let l = c.draft_ledger(id).unwrap();
    assert_eq!(l.unfinished, 1);
    assert_eq!(l.mass, 0);
    for t in queued_ids {
        c.cancel_draft(t).unwrap();
    }
    assert_eq!(c.draft_stats().unwrap().active, 0);
    let d = cache(1);
    let p = d.register_proposer("d".into(), identity()).unwrap();
    let prior = d.draft_ledger(p).unwrap().mass;
    d.audit(1, &[], 0, Pressure::Normal).unwrap();
    assert_eq!(
        d.draft_ledger(p).unwrap().mass,
        decay(prior, 1073741824).unwrap()
    );
    assert!(d.audit(1, &[], 0, Pressure::Normal).is_err());
    assert_eq!(d.draft_stats().unwrap().epoch, 1);
    let held = ready(d.request_draft(p, parent, 2, true).unwrap());
    assert!(held.bind(&[2, 3, 4]).is_err());
    held.bind(&[2, 3]).unwrap();
    assert!(held.bind(&[2]).is_err());
    held.finish(DraftOutcome {
        known_prefix: 0,
        proposed: 2,
        accepted: 3,
        committed: 1,
        target_calls: 1,
        cost_ns: 1,
        failed: false,
    })
    .err()
    .unwrap();
    assert_eq!(d.draft_ledger(p).unwrap().unfinished, 1);
}
#[test]
fn foreign_handles_inactive_policy_and_invalid_inputs() {
    let inactive = Controller::new(
        serde_json::from_str(include_str!("../fixtures/cache-config-bench.json")).unwrap(),
    )
    .unwrap();
    assert!(inactive.register_proposer("p".into(), identity()).is_err());
    let a = cache(1);
    let b = cache(1);
    let id = a.register_proposer("p".into(), identity()).unwrap();
    assert!(b.request_draft(id, [0; 32], 2, true).is_err());
    assert!(a.request_draft(id, [0; 32], 9, true).is_err());
    assert!(a.record_no_draft(id, 0, 1).is_err());
    assert!(a.register_proposer("p".into(), identity()).is_err());
    assert!(prefix_key(&identity(), &[]).is_err());
    a.record_no_draft(id, u64::MAX, 1).unwrap();
    assert!(a.record_no_draft(id, 1, 1).is_err());
}

#[test]
fn plugins_are_arbitrated_by_cost_with_bounded_requalification() {
    let c = cache(1);
    let a = c.register_proposer("slow".into(), identity()).unwrap();
    let b = c.register_proposer("fast".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    c.record_no_draft(a, 100, 1).unwrap();
    assert_eq!(c.choose_proposer(&[b, a]).unwrap(), Some(a));
    complete(ready(c.request_draft(a, parent, 2, true).unwrap()), 150, 2);
    assert_eq!(c.choose_proposer(&[a, b]).unwrap(), Some(b));
    complete(ready(c.request_draft(b, parent, 2, true).unwrap()), 10, 2);
    assert_eq!(c.choose_proposer(&[a, b]).unwrap(), Some(b));
    assert!(c.choose_proposer(&[a, a]).is_err());
    let mut foreign = identity();
    foreign.namespace.push('x');
    let f = c.register_proposer("foreign".into(), foreign).unwrap();
    assert!(c.choose_proposer(&[a, f]).is_err());
    drop(ready(c.request_draft(b, parent, 2, true).unwrap()));
    assert_eq!(c.draft_ledger(b).unwrap().unfinished, 1);
    assert_eq!(c.choose_proposer(&[a, b]).unwrap(), Some(a));
    c.audit(1, &[], 0, Pressure::Normal).unwrap();
    c.audit(2, &[], 0, Pressure::Normal).unwrap();
    let probe = ready(c.request_draft(b, parent, 2, true).unwrap());
    assert_eq!(probe.reason, DraftReason::Probe);
    complete(probe, 10, 2);
    let l = c.draft_ledger(b).unwrap();
    assert!(l.profitable);
    assert_eq!(l.unfinished, 1);
    assert_eq!(l.window_observations, 1);
    assert_eq!(c.choose_proposer(&[a, b]).unwrap(), Some(b));
}
#[test]
fn ten_thousand_audit_cycles_preserve_bounds_and_progress() {
    let c = cache(3);
    let ids: Vec<_> = (0..4)
        .map(|i| c.register_proposer(format!("p{i}"), identity()).unwrap())
        .collect();
    let parent = prefix_key(&identity(), &[1, 2, 3]).unwrap();
    c.record_no_draft(ids[0], 1000, 1).unwrap();
    let mut completed = 0;
    let mut abandoned = 0;
    for epoch in 1..=10_000 {
        if let Some(id) = c.choose_proposer(&ids).unwrap() {
            match c.request_draft(id, parent, 2, true).unwrap() {
                DraftAdmission::Ready(p) if epoch % 13 == 0 => {
                    drop(p);
                    abandoned += 1;
                }
                DraftAdmission::Ready(p) => {
                    complete(p, 1, 2);
                    completed += 1;
                }
                _ => panic!("idle admission should run"),
            }
        }
        c.audit(
            epoch,
            &[],
            0,
            if epoch % 7 == 0 {
                Pressure::Critical
            } else {
                Pressure::Normal
            },
        )
        .unwrap();
        let s = c.draft_stats().unwrap();
        assert_eq!(s.active, 0);
        assert_eq!(s.queued, 0);
        assert!(!s.faulted);
        for &id in &ids {
            assert!(c.draft_ledger(id).unwrap().mass <= MAX_MASS);
        }
    }
    let ledgers: Vec<_> = ids.iter().map(|&id| c.draft_ledger(id).unwrap()).collect();
    assert_eq!(
        ledgers.iter().map(|l| l.observations).sum::<u64>(),
        completed
    );
    assert_eq!(ledgers.iter().map(|l| l.unfinished).sum::<u64>(), abandoned);
    assert_eq!(
        ledgers.iter().map(|l| l.committed).sum::<u64>(),
        completed * 2
    );
}

#[test]
fn prefill_and_drafts_share_the_queue_bound_without_fake_kv_residency() {
    let c = cache(1);
    let full = c.branch(None, identity(), vec![1; 128]).unwrap();
    let ticket = match c.request_prefill(full).unwrap() {
        Admission::Ready(t) => t,
        _ => panic!("empty pool"),
    };
    c.fulfill(ticket, |r| {
        Ok(r.missing.iter().map(|_| vec![0; 64]).collect())
    })
    .unwrap();
    let waiting = c.branch(None, identity(), vec![2; 16]).unwrap();
    let queued_prefill = match c.request_prefill(waiting).unwrap() {
        Admission::Queued(t) => t,
        _ => panic!("full pool"),
    };
    let id = c.register_proposer("draft".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    let active = ready(c.request_draft(id, parent, 2, true).unwrap());
    let mut tickets = Vec::new();
    for _ in 0..3 {
        tickets.push(queued(c.request_draft(id, parent, 2, true).unwrap()));
    }
    assert!(c.request_draft(id, parent, 2, true).is_err());
    let second = c.branch(None, identity(), vec![3; 16]).unwrap();
    assert!(c.request_prefill(second).is_err());
    assert_eq!(c.stats().unwrap().resident_blocks, 8);
    assert_eq!(c.draft_stats().unwrap().queued, 3);
    for ticket in tickets {
        c.cancel_draft(ticket).unwrap();
    }
    c.cancel_prefill(queued_prefill).unwrap();
    drop(active);
}

#[test]
fn overflow_is_transactional_and_false_progress_is_rejected() {
    let c = cache(1);
    let id = c.register_proposer("overflow".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    c.record_no_draft(id, u64::MAX, 1).unwrap();
    complete(
        ready(c.request_draft(id, parent, 2, true).unwrap()),
        u64::MAX,
        2,
    );
    let p = ready(c.request_draft(id, parent, 2, true).unwrap());
    p.bind(&[2, 3]).unwrap();
    assert!(p
        .finish(DraftOutcome {
            known_prefix: 0,
            proposed: 2,
            accepted: 2,
            committed: 2,
            target_calls: 2,
            cost_ns: 1,
            failed: false
        })
        .is_err());
    let l = c.draft_ledger(id).unwrap();
    assert_eq!(l.observations, 1);
    assert_eq!(l.cost_ns, u64::MAX);
    assert_eq!(l.committed, 2);
    assert_eq!(l.unfinished, 1);
    assert_eq!(c.draft_stats().unwrap().active, 0);
    let d = cache(1);
    let id = d
        .register_proposer("false-progress".into(), identity())
        .unwrap();
    let p = ready(d.request_draft(id, parent, 2, true).unwrap());
    p.bind(&[2]).unwrap();
    assert!(p
        .finish(DraftOutcome {
            known_prefix: 0,
            proposed: 2,
            accepted: 2,
            committed: 3,
            target_calls: 2,
            cost_ns: 1,
            failed: false
        })
        .is_err());
    assert_eq!(d.draft_ledger(id).unwrap().observations, 0);
}

#[test]
fn concurrent_qualification_cannot_overspend_the_trial_budget() {
    let c = cache(3);
    let id = c
        .register_proposer("bounded-cold".into(), identity())
        .unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    let first = ready(c.request_draft(id, parent, 2, true).unwrap());
    let second = queued(c.request_draft(id, parent, 2, true).unwrap());
    assert_eq!(c.draft_stats().unwrap().active, 1);
    assert_eq!(c.choose_proposer(&[id]).unwrap(), None);
    assert!(c.poll_draft(second).unwrap().is_none());
    complete(first, 10, 2);
    assert!(c.poll_draft(second).is_err()); // Baseline absent, qualification exhausted.
    assert_eq!(c.draft_ledger(id).unwrap().observations, 1);
}

#[test]
fn mtp_known_seed_is_bound_but_never_earns_learned_credit() {
    for learned in 0..=7u64 {
        for accepted in 0..=learned {
            let c = cache(1);
            let id = c.register_proposer("mtp".into(), identity()).unwrap();
            let parent = prefix_key(&identity(), &[1]).unwrap();
            let p = ready(
                c.request_draft(id, parent, learned as usize + 1, true)
                    .unwrap(),
            );
            p.bind(&vec![2; learned as usize + 1]).unwrap();
            p.finish(DraftOutcome {
                known_prefix: 1,
                proposed: learned,
                accepted,
                committed: accepted + 2,
                target_calls: 2,
                cost_ns: 100,
                failed: false,
            })
            .unwrap();
            let l = c.draft_ledger(id).unwrap();
            assert_eq!(
                (l.known_prefix, l.proposed, l.accepted, l.committed),
                (1, learned, accepted, accepted + 2)
            );
            assert_eq!(c.draft_stats().unwrap().active, 0);
        }
    }
    // Seed-only work cannot earn a speculative reward or profitable admission.
    let c = cache(1);
    let id = c.register_proposer("seed-only".into(), identity()).unwrap();
    let parent = prefix_key(&identity(), &[1]).unwrap();
    c.record_no_draft(id, 10, 1).unwrap();
    let p = ready(c.request_draft(id, parent, 1, true).unwrap());
    p.bind(&[2]).unwrap();
    p.finish(DraftOutcome {
        known_prefix: 1,
        proposed: 0,
        accepted: 0,
        committed: 1,
        target_calls: 1,
        cost_ns: 100,
        failed: false,
    })
    .unwrap();
    assert_eq!(c.draft_ledger(id).unwrap().accepted, 0);
    assert!(!c.draft_ledger(id).unwrap().profitable);
}

#[test]
fn mtp_seed_geometry_cannot_forge_progress_or_binding() {
    for (known, proposed, accepted, committed, bound) in [
        (2, 0, 0, 1, 2),
        (1, 8, 0, 1, 8),
        (1, 1, 0, 3, 2),
        (1, 1, 1, 3, 1),
        (u64::MAX, 0, 0, 0, 1),
    ] {
        let c = cache(1);
        let id = c.register_proposer("mtp-fault".into(), identity()).unwrap();
        let parent = prefix_key(&identity(), &[1]).unwrap();
        let p = ready(c.request_draft(id, parent, 8, true).unwrap());
        p.bind(&vec![2; bound]).unwrap();
        assert!(p
            .finish(DraftOutcome {
                known_prefix: known,
                proposed,
                accepted,
                committed,
                target_calls: 1,
                cost_ns: 100,
                failed: false
            })
            .is_err());
        let l = c.draft_ledger(id).unwrap();
        assert_eq!(
            (
                l.observations,
                l.known_prefix,
                l.proposed,
                l.accepted,
                l.committed
            ),
            (0, 0, 0, 0, 0)
        );
        assert_eq!(l.unfinished, 1);
    }
}
