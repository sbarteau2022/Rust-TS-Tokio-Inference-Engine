#![cfg(feature = "native-mlx")]
use anyhow::Result;
use rust_server::serving::{
    completion_ledger::{Footprint, Ledger},
    native_mlx::{Backend, Data, Device, Graph},
};

fn product(m: i32, k: i32, n: i32) -> Result<(Graph, u32, Vec<f32>)> {
    let a: Vec<f32> = (0..m * k)
        .map(|i| ((i * 7 % 19) - 9) as f32 / 8.0)
        .collect();
    let b: Vec<f32> = (0..k * n)
        .map(|i| ((i * 3 % 17) - 8) as f32 / 8.0)
        .collect();
    let mut expected = vec![0.0; (m * n) as usize];
    for row in 0..m {
        for col in 0..n {
            for j in 0..k {
                expected[(row * n + col) as usize] +=
                    a[(row * k + j) as usize] * b[(j * n + col) as usize];
            }
        }
    }
    let mut g = Graph::new();
    let x = g.input(&[m, k], Data::F32(a))?;
    let w = g.input(&[k, n], Data::F32(b))?;
    let y = g.matmul(x, w)?;
    Ok((g, y, expected))
}
fn close(actual: Data, expected: &[f32]) {
    let Data::F32(values) = actual else {
        panic!("expected F32")
    };
    assert_eq!(values.len(), expected.len());
    for (a, b) in values.iter().zip(expected) {
        assert!((a - b).abs() <= 1e-4 * (1.0 + b.abs()), "{a} vs {b}");
    }
}
#[tokio::test]
async fn native_dense_graphs_match_independent_oracle_on_cpu_and_gpu() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        for shape in [(1, 1, 1), (3, 7, 5), (32, 64, 17), (64, 128, 64)] {
            let (g, y, oracle) = product(shape.0, shape.1, shape.2)?;
            close(backend.submit(g, y)?.result().await?.data, &oracle);
        }
        let stats = backend.shutdown().await?;
        assert_eq!(stats.completed, 4);
        assert_eq!(stats.resources.leases, 0);
        assert_eq!(stats.resources.used, Footprint::default());
    }
    Ok(())
}
#[tokio::test]
async fn raw_integer_buffers_preserve_bits_without_float_conversion() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        let mut g = Graph::new();
        let x = g.input(
            &[4],
            Data::I32(vec![i32::MIN, i32::MAX, 16777217, -16777217]),
        )?;
        let zero = g.input(&[4], Data::I32(vec![0; 4]))?;
        let y = g.add(x, zero)?;
        assert_eq!(
            backend.submit(g, y)?.result().await?.data,
            Data::I32(vec![i32::MIN, i32::MAX, 16777217, -16777217])
        );
        let mut g = Graph::new();
        let x = g.input(&[2], Data::U32(vec![u32::MAX, 0x80000001]))?;
        let one = g.input(&[2], Data::U32(vec![1, 1]))?;
        let y = g.multiply(x, one)?;
        assert_eq!(
            backend.submit(g, y)?.result().await?.data,
            Data::U32(vec![u32::MAX, 0x80000001])
        );
        backend.shutdown().await?;
    }
    Ok(())
}
#[tokio::test]
async fn packed_q4_and_rms_match_independent_scalar_oracles() -> Result<()> {
    for group in [32, 64, 128] {
        let backend = Backend::new(Device::Gpu)?;
        let k = group * 2;
        let m = 3;
        let n = 5;
        let x: Vec<f32> = (0..m * k).map(|i| (i % 9 - 4) as f32 / 8.0).collect();
        let mut packed = vec![0u32; (n * k / 8) as usize];
        for row in 0..n {
            for j in 0..k {
                packed[(row * k / 8 + j / 8) as usize] |=
                    (((row * 3 + j) % 16) as u32) << ((j % 8) * 4);
            }
        }
        let scales: Vec<f32> = (0..n * 2).map(|i| (i % 3 + 1) as f32 / 16.0).collect();
        let biases = vec![-0.25; (n * 2) as usize];
        let mut oracle = vec![0.0; (m * n) as usize];
        for row in 0..m {
            for col in 0..n {
                for j in 0..k {
                    let q = ((col * 3 + j) % 16) as f32;
                    oracle[(row * n + col) as usize] += x[(row * k + j) as usize]
                        * (q * scales[(col * 2 + j / group) as usize] - 0.25);
                }
            }
        }
        let mut g = Graph::new();
        let a = g.input(&[m, k], Data::F32(x))?;
        let w = g.input(&[n, k / 8], Data::U32(packed))?;
        let s = g.input(&[n, 2], Data::F32(scales))?;
        let b = g.input(&[n, 2], Data::F32(biases))?;
        let y = g.q4_matmul(a, w, s, b, group)?;
        let weight = g.input(&[n], Data::F32(vec![0.5; n as usize]))?;
        let y = g.rms_norm(y, weight, 1e-5)?;
        for row in oracle.chunks_mut(n as usize) {
            let rms = (row.iter().map(|v| v * v).sum::<f32>() / n as f32 + 1e-5).sqrt();
            for value in row {
                *value = *value / rms * 0.5;
            }
        }
        close(backend.submit(g, y)?.result().await?.data, &oracle);
        backend.shutdown().await?;
    }
    Ok(())
}
#[test]
fn graph_validation_rejects_adversarial_shapes_types_and_caps() -> Result<()> {
    let mut g = Graph::new();
    for shape in [vec![], vec![0], vec![-1], vec![8193], vec![1; 5]] {
        assert!(g.input(&shape, Data::F32(vec![])).is_err());
    }
    assert!(g.input(&[1], Data::F32(vec![f32::NAN])).is_err());
    assert!(g.input(&[1], Data::F32(vec![f32::INFINITY])).is_err());
    assert!(g.input(&[2], Data::F32(vec![0.0])).is_err());
    let x = g.input(&[1, 2], Data::F32(vec![1.0, 2.0]))?;
    let i = g.input(&[1, 2], Data::I32(vec![1, 2]))?;
    assert!(g.add(x, i).is_err());
    assert!(g.matmul(x, x).is_err());
    assert!(g.add(x, u32::MAX).is_err());
    assert!(g.rms_norm(x, x, 0.0).is_err());
    assert!(g.q4_matmul(x, i, x, x, 16).is_err());
    let mut current = x;
    for _ in 0..254 {
        current = g.add(current, x)?;
    }
    assert!(g.add(current, x).is_err());
    // Each broadcast-free matmul output is 64 MiB; four fit the graph tensor cap only if inputs are accounted for.
    let mut big = Graph::new();
    let a = big.input(&[8192, 1], Data::F32(vec![0.0; 8192]))?;
    let b = big.input(&[1, 8192], Data::F32(vec![0.0; 8192]))?;
    assert!(big.matmul(a, b).is_err()); // 256 MiB output plus inputs exceeds the graph cap.
    Ok(())
}
#[tokio::test]
async fn joint_native_budget_rejects_without_poisoning_then_drains_on_shutdown() -> Result<()> {
    let backend = Backend::with_limits(
        Device::Gpu,
        Footprint {
            host_bytes: 32,
            tensor_bytes: 32,
            graph_nodes: 3,
        },
    )?;
    let (g, y, _) = product(2, 2, 2)?;
    assert!(backend.submit(g, y).is_err());
    assert_eq!(backend.stats().submitted, 0);
    assert_eq!(backend.stats().resources.leases, 0);
    assert!(!backend.stats().admission_closed);
    let (g, y, oracle) = product(1, 1, 1)?;
    let receipt = backend.submit(g, y)?;
    let stats = backend.shutdown().await?;
    close(receipt.result().await?.data, &oracle);
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.failed, 0);
    assert_eq!(stats.resources.leases, 0);
    Ok(())
}
#[tokio::test]
async fn caller_cancellation_cannot_skip_native_work_and_nonfinite_result_closes_admission(
) -> Result<()> {
    let backend = Backend::new(Device::Gpu)?;
    let (g, y, _) = product(128, 128, 128)?;
    let receipt = backend.submit(g, y)?;
    drop(receipt);
    let stats = backend.shutdown().await?;
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.abandoned, 1);
    assert_eq!(stats.in_flight, 0);
    assert_eq!(stats.resources.leases, 0);
    let backend = Backend::new(Device::Gpu)?;
    let mut g = Graph::new();
    let x = g.input(&[1], Data::F32(vec![f32::MAX]))?;
    let y = g.multiply(x, x)?;
    assert!(backend.submit(g, y)?.result().await.is_err());
    let (g, y, _) = product(1, 1, 1)?;
    assert!(backend.submit(g, y).is_err());
    let stats = backend.shutdown().await?;
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.resources.leases, 0);
    Ok(())
}
#[tokio::test]
async fn native_pressure_4500_completions_bound_three_and_reconcile_windows() -> Result<()> {
    let backend = Backend::new(Device::Gpu)?;
    for window in [1000, 1500, 2000] {
        let baseline = backend.stats().completed;
        for start in (0..window).step_by(3) {
            let mut receipts = Vec::new();
            for _ in start..(start + 3).min(window) {
                let (g, y, oracle) = product(3, 7, 5)?;
                receipts.push((backend.submit(g, y)?, oracle));
            }
            for (receipt, oracle) in receipts {
                close(receipt.result().await?.data, &oracle);
            }
        }
        let stats = backend.stats();
        assert_eq!(stats.completed - baseline, window);
        assert_eq!(stats.in_flight, 0);
        assert!(stats.peak_in_flight <= 3);
        assert_eq!(stats.resources.used, Footprint::default());
    }
    let stats = backend.shutdown().await?;
    assert_eq!(stats.completed, 4500);
    assert_eq!(stats.submitted, 4500);
    assert_eq!(stats.failed, 0);
    Ok(())
}
#[test]
fn atomic_resource_ledger_rolls_back_overflow_and_pressure_across_threads() -> Result<()> {
    let limits = Footprint {
        host_bytes: 64,
        tensor_bytes: 64,
        graph_nodes: 64,
    };
    let ledger = Ledger::new(limits)?;
    let lease = ledger.reserve(Footprint {
        host_bytes: 32,
        tensor_bytes: 32,
        graph_nodes: 32,
    })?;
    for cost in [
        Footprint {
            host_bytes: 33,
            ..Footprint::default()
        },
        Footprint {
            tensor_bytes: 33,
            ..Footprint::default()
        },
        Footprint {
            graph_nodes: 33,
            ..Footprint::default()
        },
        Footprint {
            host_bytes: usize::MAX,
            ..Footprint::default()
        },
    ] {
        assert!(ledger.reserve(cost).is_err());
        assert_eq!(
            ledger.snapshot().used,
            Footprint {
                host_bytes: 32,
                tensor_bytes: 32,
                graph_nodes: 32
            }
        );
    }
    drop(lease);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let ledger = ledger.clone();
            scope.spawn(move || {
                for _ in 0..10000 {
                    if let Ok(lease) = ledger.reserve(Footprint {
                        host_bytes: 8,
                        tensor_bytes: 8,
                        graph_nodes: 8,
                    }) {
                        let snapshot = ledger.snapshot();
                        assert!(
                            snapshot.used.host_bytes <= 64
                                && snapshot.used.tensor_bytes <= 64
                                && snapshot.used.graph_nodes <= 64
                        );
                        drop(lease);
                    }
                }
            });
        }
    });
    assert_eq!(ledger.snapshot().leases, 0);
    assert_eq!(ledger.snapshot().used, Footprint::default());
    Ok(())
}
