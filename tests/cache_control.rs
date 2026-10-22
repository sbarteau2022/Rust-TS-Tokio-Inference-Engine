use rust_server::{
    cache::*,
    execute, ir,
    runtime::{ChainJob, Runtime},
    sampled::{Selection, ValidationConfig},
};
fn config() -> Config {
    serde_json::from_str(include_str!("../fixtures/cache-config-bench.json")).unwrap()
}
fn identity() -> Identity {
    Identity {
        model: "test-weights@sha256:fixture".into(),
        tokenizer: "test-tokenizer-template-v1".into(),
        quantization: "exact-test-payload-v1".into(),
        namespace: "private-test".into(),
    }
}
fn ticket(admission: Admission) -> TicketId {
    match admission {
        Admission::Ready(t) | Admission::Queued(t) => t,
    }
}
fn fill(cache: &Controller, branch: BranchId) {
    let id = ticket(cache.request_prefill(branch).unwrap());
    cache
        .fulfill(id, |request| {
            Ok(request.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
        })
        .unwrap();
}
#[test]
fn exact_prefix_identity_is_path_and_namespace_dependent() {
    let cache = Controller::new(config()).unwrap();
    let a = cache.branch(None, identity(), (0..32).collect()).unwrap();
    fill(&cache, a);
    let b = cache.branch(None, identity(), (0..32).collect()).unwrap();
    let id = ticket(cache.request_prefill(b).unwrap());
    cache
        .fulfill(id, |_| panic!("resident prefix must not be pref illed"))
        .unwrap();
    assert_eq!(cache.stats().unwrap().resident_blocks, 2);
    assert!(cache.blocks().unwrap().iter().all(|b| b.references == 2));
    for field in 0..4 {
        let mut other = identity();
        match field {
            0 => other.model.push('x'),
            1 => other.tokenizer.push('x'),
            2 => other.quantization.push('x'),
            _ => other.namespace.push('x'),
        }
        let branch = cache.branch(None, other, (0..32).collect()).unwrap();
        let audit = cache
            .audit(field + 1, &[branch], 123, Pressure::Normal)
            .unwrap();
        assert_eq!(audit.declared_tokens, 32);
        assert_eq!(audit.resident_declared_tokens, 0);
        assert_eq!(audit.unknown_output_estimate_bytes, 123);
    }
    let divergent = cache.branch(None, identity(), (16..48).collect()).unwrap();
    let audit = cache.audit(5, &[divergent], 0, Pressure::Normal).unwrap();
    assert_eq!(audit.resident_declared_tokens, 0);
}
#[test]
fn partial_tail_fork_is_immutable_and_locks_bar_write_forks() {
    let cache = Controller::new(config()).unwrap();
    let parent = cache.branch(None, identity(), vec![7; 5]).unwrap();
    fill(&cache, parent);
    let lease = cache.lease(parent).unwrap();
    let before = cache.read(&lease).unwrap();
    assert!(cache.branch(Some(parent), identity(), vec![8]).is_err());
    assert!(cache.boundary().is_err());
    drop(lease);
    let child = cache.branch(Some(parent), identity(), vec![8; 11]).unwrap();
    fill(&cache, child);
    assert_eq!(cache.stats().unwrap().resident_blocks, 2);
    let lease = cache.lease(parent).unwrap();
    assert_eq!(cache.read(&lease).unwrap(), before);
    let other = Controller::new(config()).unwrap();
    assert!(other.read(&lease).is_err());
    assert!(other.lease(parent).is_err());
}
#[test]
fn decay_ties_thresholds_and_one_level_per_boundary() {
    assert_eq!(decay(1, 1 << 30).unwrap(), 0);
    assert_eq!(decay(3, 1 << 30).unwrap(), 2);
    assert_eq!(decay(MAX_MASS, 0).unwrap(), MAX_MASS);
    assert!(decay(u32::MAX, 0).is_err());
    let mut cfg = config();
    cfg.anchor_fraction_q31 = 0;
    let cache = Controller::new(cfg).unwrap();
    let root = cache.branch(None, identity(), (0..48).collect()).unwrap();
    fill(&cache, root);
    cache.close_subtree(root).unwrap();
    assert_eq!(cache.stats().unwrap().resident_blocks, 3);
    assert_eq!(
        cache
            .audit(1, &[], 0, Pressure::Critical)
            .unwrap()
            .freed_blocks,
        1
    );
    assert_eq!(cache.stats().unwrap().resident_blocks, 2);
    assert_eq!(cache.boundary().unwrap(), 1);
    assert_eq!(cache.boundary().unwrap(), 1);
    assert!(cache.audit(1, &[], 0, Pressure::Critical).is_err());
    let mut cfg = config();
    cfg.anchor_fraction_q31 = 0;
    cfg.dead_threshold = 1;
    let cache = Controller::new(cfg).unwrap();
    let root = cache.branch(None, identity(), vec![2; 16]).unwrap();
    fill(&cache, root);
    cache.close_subtree(root).unwrap();
    assert_eq!(
        cache
            .audit(1, &[], 0, Pressure::Critical)
            .unwrap()
            .freed_blocks,
        0
    );
    assert_eq!(cache.blocks().unwrap()[0].mass, 1);
    assert_eq!(
        cache
            .audit(2, &[], 0, Pressure::Critical)
            .unwrap()
            .freed_blocks,
        1
    );
}
#[test]
fn anchors_degrade_visibly_and_holds_survive_pressure() {
    let mut cfg = config();
    cfg.capacity_blocks = 2;
    let cache = Controller::new(cfg).unwrap();
    let branch = cache.branch(None, identity(), (0..32).collect()).unwrap();
    fill(&cache, branch);
    let audit = cache.audit(1, &[branch], 999, Pressure::Critical).unwrap();
    assert_eq!(audit.anchored_blocks, 1);
    assert_eq!(audit.anchor_degradations, 1);
    assert_eq!(audit.resident_declared_tokens, 32);
    assert!(cache
        .blocks()
        .unwrap()
        .iter()
        .any(|b| b.anchored && b.rate == 0 && b.mass == MAX_MASS));
    cache.audit(2, &[], 0, Pressure::Critical).unwrap();
    assert_eq!(cache.stats().unwrap().resident_blocks, 2);
    cache.close_subtree(branch).unwrap();
    cache.audit(3, &[], 0, Pressure::Critical).unwrap();
    assert_eq!(cache.stats().unwrap().resident_blocks, 1);
}
#[test]
fn continuation_queue_priority_expiry_and_no_resident_reprefill() {
    let mut cfg = config();
    cfg.capacity_blocks = 2;
    cfg.anchor_fraction_q31 = 0;
    let cache = Controller::new(cfg).unwrap();
    let a = cache.branch(None, identity(), vec![1; 16]).unwrap();
    fill(&cache, a);
    let b = cache.branch(None, identity(), vec![2; 16]).unwrap();
    fill(&cache, b);
    let fresh = cache.branch(None, identity(), vec![3; 16]).unwrap();
    let new_ticket = ticket(cache.request_prefill(fresh).unwrap());
    let child = cache.branch(Some(a), identity(), vec![4; 16]).unwrap();
    let continuation = ticket(cache.request_prefill(child).unwrap());
    cache.close_subtree(b).unwrap();
    cache.audit(1, &[], 0, Pressure::Critical).unwrap();
    assert!(cache.ready().unwrap().contains(&continuation));
    assert!(!cache.ready().unwrap().contains(&new_ticket));
    cache
        .fulfill(continuation, |r| {
            Ok(r.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
        })
        .unwrap();
    cache.audit(2, &[], 0, Pressure::Critical).unwrap();
    cache.audit(3, &[], 0, Pressure::Critical).unwrap();
    assert_eq!(cache.stats().unwrap().queue_expirations, 1);
    assert!(cache.cancel_prefill(new_ticket).is_err());
}
#[test]
fn prefix_inflight_is_deduplicated_and_producer_errors_release_reservations() {
    let cache = Controller::new(config()).unwrap();
    let a = cache.branch(None, identity(), vec![4; 16]).unwrap();
    let b = cache.branch(None, identity(), vec![4; 16]).unwrap();
    let first = ticket(cache.request_prefill(a).unwrap());
    assert!(matches!(
        cache.request_prefill(b).unwrap(),
        Admission::Queued(_)
    ));
    cache
        .fulfill(first, |r| {
            Ok(r.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
        })
        .unwrap();
    let reused = cache.ready().unwrap()[0];
    cache
        .fulfill(reused, |_| panic!("duplicate prefill"))
        .unwrap();
    assert_eq!(cache.stats().unwrap().resident_blocks, 1);
    assert_eq!(cache.blocks().unwrap()[0].references, 2);
    let c = cache.branch(None, identity(), vec![9; 16]).unwrap();
    let id = ticket(cache.request_prefill(c).unwrap());
    assert!(cache
        .fulfill(id, |_| anyhow::bail!("injected producer failure"))
        .is_err());
    assert_eq!(cache.stats().unwrap().reserved_blocks, 0);
    let id = ticket(cache.request_prefill(c).unwrap());
    assert!(cache.fulfill(id, |_| Ok(vec![vec![0; 63]])).is_err());
    assert_eq!(cache.stats().unwrap().resident_blocks, 1);
    let id = ticket(cache.request_prefill(c).unwrap());
    assert!(cache
        .fulfill(id, |_| panic!("injected producer panic"))
        .is_err());
    assert_eq!(cache.stats().unwrap().reserved_blocks, 0);
}
#[test]
fn unknown_reservations_are_not_tokens_and_queue_is_bounded() {
    let mut cfg = config();
    cfg.capacity_blocks = 2;
    cfg.queue_depth = 1;
    let cache = Controller::new(cfg).unwrap();
    let a = cache.branch(None, identity(), vec![1; 16]).unwrap();
    fill(&cache, a);
    assert_eq!(cache.reserve_unknown(a, 1).unwrap(), 1);
    let b = cache.branch(None, identity(), vec![2; 16]).unwrap();
    assert!(matches!(
        cache.request_prefill(b).unwrap(),
        Admission::Queued(_)
    ));
    let c = cache.branch(None, identity(), vec![3; 16]).unwrap();
    assert!(cache.request_prefill(c).is_err());
    let audit = cache.audit(1, &[a], 64, Pressure::Normal).unwrap();
    assert_eq!(audit.declared_tokens, 16);
    assert_eq!(audit.resident_declared_tokens, 16);
    cache.reserve_unknown(a, 0).unwrap();
    assert_eq!(cache.ready().unwrap().len(), 1);
    assert_eq!(cache.stats().unwrap().pressure_rejections, 1);
}
#[test]
fn random_pressure_operations_preserve_bounds_and_parentage() {
    let mut cfg = config();
    cfg.max_declared_tokens = 65536;
    cfg.max_branches = 128;
    let cache = Controller::new(cfg).unwrap();
    let mut branches = Vec::new();
    let mut epoch = 0;
    let mut seed = 17u64;
    for _ in 0..10000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        match seed % 5 {
            0 => {
                if let Ok(b) = cache.branch(None, identity(), vec![(seed >> 32) as u32; 16]) {
                    branches.push(b);
                }
            }
            1 => {
                if let Some(b) = branches.last() {
                    if let Ok(Admission::Ready(t)) = cache.request_prefill(*b) {
                        let _ = cache.fulfill(t, |r| {
                            Ok(r.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
                        });
                    }
                }
            }
            2 => {
                if let Some(b) = branches.pop() {
                    let _ = cache.close_subtree(b);
                }
            }
            _ => {
                epoch += 1;
                cache.audit(epoch, &[], 0, Pressure::Critical).unwrap();
            }
        }
        let stats = cache.stats().unwrap();
        assert!(stats.resident_blocks + stats.reserved_blocks <= 8);
        assert!(stats.queued <= 4);
        let blocks = cache.blocks().unwrap();
        for block in &blocks {
            if let Some(parent) = block.parent {
                assert!(blocks.iter().any(|b| b.key == parent));
            }
        }
    }
}
#[tokio::test]
async fn cached_gpu_receipts_release_leases_on_native_completion() {
    let cache = Controller::new(config()).unwrap();
    let branch = cache.branch(None, identity(), vec![1; 16]).unwrap();
    fill(&cache, branch);
    let runtime = Runtime::new(8).unwrap();
    let trace = include_bytes!("../fixtures/elementwise.amvk").to_vec();
    let plan = ir::lower_trace(&trace).unwrap();
    let receipt = runtime
        .submit_cached(
            ChainJob {
                traces: vec![trace],
                inputs: execute::seeded_inputs(&plan, 42),
            },
            cache.lease(branch).unwrap(),
        )
        .unwrap()
        .await
        .unwrap();
    assert!(receipt.failure.is_none());
    assert_eq!(receipt.validation_policy, "DETERMINISTIC-MODE");
    assert_eq!(cache.stats().unwrap().active_leases, 0);
    let sampling = ValidationConfig {
        selection: Selection::DynamicDraft,
        every: 100,
    };
    assert!(Runtime::with_sampling(8, 3, None, sampling).is_err());
    let runtime = Runtime::with_sampling(
        8,
        3,
        None,
        ValidationConfig {
            selection: Selection::DeterministicBench,
            every: 1,
        },
    )
    .unwrap();
    let trace = include_bytes!("../fixtures/elementwise.amvk").to_vec();
    assert!(runtime
        .submit_cached(
            ChainJob {
                traces: vec![trace],
                inputs: execute::seeded_inputs(&plan, 42)
            },
            cache.lease(branch).unwrap()
        )
        .unwrap()
        .await
        .unwrap()
        .failure
        .is_none());
    let stats = runtime.flush_validation().await.unwrap();
    assert_eq!(stats.passed, 1);
    assert!(!stats.production_sampling_claims);
}

#[test]
fn invalid_configuration_and_unknown_fields_fail_closed() {
    let mut bad = Vec::new();
    let mut c = config();
    c.capacity_blocks = 0;
    bad.push(c);
    let mut c = config();
    c.bytes_per_block = usize::MAX;
    bad.push(c);
    let mut c = config();
    c.max_leases = 0;
    bad.push(c);
    let mut c = config();
    c.anchor_fraction_q31 = u32::MAX;
    bad.push(c);
    let mut c = config();
    c.rates_q31 = [1, 0, 1];
    bad.push(c);
    let mut c = config();
    c.pressure_multipliers_q31 = [ONE, 0, ONE];
    bad.push(c);
    let mut c = config();
    c.max_wait_epochs = 0;
    bad.push(c);
    let mut c = config();
    c.queue_depth = 0;
    bad.push(c);
    for cfg in bad {
        assert!(Controller::new(cfg).is_err());
    }
    let mut value = serde_json::to_value(config()).unwrap();
    value["silent_typo"] = true.into();
    assert!(serde_json::from_value::<Config>(value).is_err());
    let cache = Controller::new(config()).unwrap();
    let mut id = identity();
    id.namespace.clear();
    assert!(cache.branch(None, id, vec![1]).is_err());
}
#[test]
fn epochs_and_fixed_point_match_independent_division_oracle() {
    let mut seed = 99u64;
    for _ in 0..10000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let mass = (seed >> 33) as u32;
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rate = (seed >> 33) as u32;
        let product = i128::from(mass) * (i128::from(ONE) - i128::from(rate));
        let divisor = i128::from(ONE);
        let q = product / divisor;
        let r = product % divisor;
        let expected = q + i128::from(2 * r > divisor || (2 * r == divisor && q % 2 == 1));
        assert_eq!(decay(mass, rate).unwrap(), expected as u32);
    }
}
#[test]
fn duplicate_prefill_execution_and_close_during_callback_are_safe() {
    let cache = Controller::new(config()).unwrap();
    let a = cache.branch(None, identity(), vec![2; 16]).unwrap();
    let id = ticket(cache.request_prefill(a).unwrap());
    cache
        .fulfill(id, |r| {
            assert!(cache.fulfill(id, |_| panic!("duplicate callback")).is_err());
            Ok(r.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
        })
        .unwrap();
    let b = cache.branch(Some(a), identity(), vec![3; 16]).unwrap();
    let id = ticket(cache.request_prefill(b).unwrap());
    assert!(cache
        .fulfill(id, |r| {
            cache.close_subtree(b)?;
            cache.boundary()?;
            Ok(r.missing.iter().map(|d| vec![d.key[0]; 64]).collect())
        })
        .is_err());
    assert_eq!(cache.stats().unwrap().active_prefills, 0);
    assert_eq!(cache.stats().unwrap().prefill_snapshot_bytes, 0);
    assert_eq!(cache.stats().unwrap().reserved_blocks, 0);
    assert!(cache.lease(a).is_ok());
}
