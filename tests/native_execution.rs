#![cfg(feature = "native-mlx")]
use anyhow::Result;
use rust_server::serving::{
    native_mlx::{Backend, Data, Device, Graph},
    weights::{Manifest, Quantization},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
fn close(data: Data, expected: &[f32]) {
    let Data::F32(actual) = data else {
        panic!("F32 expected")
    };
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
        assert!((a - b).abs() < 2e-4 * (1.0 + b.abs()), "{a} != {b}");
    }
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "native-weights-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p)?;
        fs::write(
            p.join("config.json"),
            r#"{"quantization":{"bits":4,"group_size":32,"router":{"bits":8}}}"#,
        )?;
        fs::write(
            p.join("model.safetensors.index.json"),
            r#"{"weight_map":{"value":"model.safetensors"}}"#,
        )?;
        let header = br#"{"value":{"dtype":"F32","shape":[2,2],"data_offsets":[0,16]}}"#;
        let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
        bytes.extend(header);
        for v in [1f32, 2., 3., 4.] {
            bytes.extend(v.to_le_bytes());
        }
        fs::write(p.join("model.safetensors"), bytes)?;
        Ok(Self(p))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[tokio::test]
async fn persistent_weights_reuse_and_owner_isolation() -> Result<()> {
    let f = Fixture::new()?;
    let backend = Backend::with_checkpoint(Device::Gpu, f.0.clone()).await?;
    let model = backend.model().unwrap();
    assert_eq!(
        model.manifest().module_quantization("router"),
        Quantization { bits: 8, group: 32 }
    );
    let other = Backend::new(Device::Gpu)?;
    for _ in 0..8 {
        let mut g = Graph::new();
        let w = g.parameter(model, "value")?;
        let x = g.input(&[2, 2], Data::F32(vec![1., 0., 0., 1.]))?;
        let y = g.matmul(x, w)?;
        let mut wrong = Graph::new();
        let wrong_w = wrong.parameter(model, "value")?;
        assert!(other.submit(wrong, wrong_w).is_err());
        close(
            backend.submit(g, y)?.result().await?.data,
            &[1., 2., 3., 4.],
        );
    }
    assert_eq!(backend.stats().parameter_bytes, 16);
    let stats = backend.shutdown().await?;
    assert_eq!(stats.parameter_bytes, 0);
    assert_eq!(stats.resources.leases, 0);
    other.shutdown().await?;
    Ok(())
}
#[test]
fn checkpoint_mutation_and_malformed_metadata_rejected() -> Result<()> {
    let f = Fixture::new()?;
    let m = Manifest::read(&f.0)?;
    fs::write(f.0.join("new-artifact"), b"changed")?;
    assert!(m.verify_stamps().is_err());
    fs::remove_file(f.0.join("new-artifact"))?;
    fs::write(
        f.0.join("model.safetensors.index.json"),
        r#"{"weight_map":{"value":"../escape.safetensors"}}"#,
    )?;
    assert!(Manifest::read(&f.0).is_err());
    fs::write(
        f.0.join("config.json"),
        r#"{"quantization":{"bits":7,"group_size":32}}"#,
    )?;
    assert!(Manifest::read(&f.0).is_err());
    Ok(())
}
#[tokio::test]
async fn mixed_affine_widths_constant_packing_oracle() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        for bits in [2, 3, 4, 5, 6, 8] {
            for group in [32, 64, 128] {
                // All-one fields are packing-order independent, including 3/5/6-bit formats.
                let mut g = Graph::new();
                let x = g.input(&[2, group], Data::F32(vec![1.; 2 * group as usize]))?;
                let w = g.input(
                    &[3, group * bits / 32],
                    Data::U32(vec![u32::MAX; (3 * group * bits / 32) as usize]),
                )?;
                let s = g.input(&[3, 1], Data::F32(vec![0.125; 3]))?;
                let b = g.input(&[3, 1], Data::F32(vec![-0.25; 3]))?;
                let y = g.quantized_matmul(x, w, s, b, group, bits)?;
                close(
                    backend.submit(g, y)?.result().await?.data,
                    &[group as f32 * (((1 << bits) - 1) as f32 * 0.125 - 0.25); 6],
                );
            }
        }
        backend.shutdown().await?;
    }
    Ok(())
}
#[tokio::test]
async fn causal_attention_and_rope_analytic_oracles() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        let mut g = Graph::new();
        let q = g.input(&[1, 2, 2, 2], Data::F32(vec![0.; 8]))?;
        let k = g.input(&[1, 1, 2, 2], Data::F32(vec![0.; 4]))?;
        let v = g.input(&[1, 1, 2, 2], Data::F32(vec![2., 4., 6., 8.]))?;
        let y = g.attention(q, k, v, 1., true)?;
        close(
            backend.submit(g, y)?.result().await?.data,
            &[2., 4., 4., 6., 2., 4., 4., 6.],
        );
        let mut g = Graph::new();
        let x = g.input(&[1, 1, 1, 2], Data::F32(vec![1., 0.]))?;
        let y = g.rope(x, 2, 1, 10000.)?;
        close(
            backend.submit(g, y)?.result().await?.data,
            &[1f32.cos(), 1f32.sin()],
        );
        backend.shutdown().await?;
    }
    Ok(())
}
#[tokio::test]
async fn recurrent_multi_output_state_matches_scalar_update() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        for output_state in [false, true] {
            let mut g = Graph::new();
            let q = g.input(&[1, 2, 1, 2], Data::F32(vec![0.5, 0., 0., 0.5]))?;
            let k = g.input(&[1, 2, 1, 2], Data::F32(vec![1., 0., 0., 1.]))?;
            let v = g.input(&[1, 2, 2, 1], Data::F32(vec![2., 4., 6., 8.]))?;
            let decay = g.input(&[1, 2, 2], Data::F32(vec![0.5; 4]))?;
            let beta = g.input(&[1, 2, 2], Data::F32(vec![0.5; 4]))?;
            let state = g.input(&[1, 2, 1, 2], Data::F32(vec![0.; 4]))?;
            let (y, s) = g.gated_delta([q, k, v, decay, beta, state])?;
            close(
                backend
                    .submit(g, if output_state { s } else { y })?
                    .result()
                    .await?
                    .data,
                if output_state {
                    &[0.5, 3., 1., 4.]
                } else {
                    &[0.5, 1., 1.5, 2.]
                },
            );
        }
        backend.shutdown().await?;
    }
    Ok(())
}
#[tokio::test]
async fn expert_route_gather_mix_and_oob_rejection() -> Result<()> {
    for device in [Device::Cpu, Device::Gpu] {
        let backend = Backend::new(device)?;
        let mut g = Graph::new();
        let logits = g.input(&[1, 3], Data::F32(vec![-10., 0., 1.]))?;
        let (idx, scores) = g.route_topk(logits, 2)?;
        let x = g.input(&[1, 32], Data::F32(vec![1.; 32]))?;
        let w = g.input(&[3, 1, 4], Data::U32(vec![0; 12]))?;
        let scales = g.input(&[3, 1, 1], Data::F32(vec![1.; 3]))?;
        let bias = g.input(&[3, 1, 1], Data::F32(vec![0., 1., 2.]))?;
        let quant = Quantization { bits: 4, group: 32 };
        let unsafe_idx = g.input(&[1, 1], Data::U32(vec![3]))?;
        assert!(g
            .gather_quantized(x, w, scales, bias, unsafe_idx, quant)
            .is_err());
        let experts = g.gather_quantized(x, w, scales, bias, idx, quant)?;
        let y = g.mix_experts(experts, scores)?;
        close(
            backend.submit(g, y)?.result().await?.data,
            &[32. * (1. + 2. * 1f32.exp()) / (1. + 1f32.exp())],
        );
        backend.shutdown().await?;
    }
    Ok(())
}

#[tokio::test]
async fn state_batches_preserve_rows_through_reorder_remove_and_failed_work() -> Result<()> {
    use rust_server::serving::native_state::RecurrentBank;
    let backend = Backend::new(Device::Gpu)?;
    let bank = RecurrentBank::new(&backend, [1, 1, 1], 3)?;
    for id in [10, 20, 30] {
        bank.insert(id)?;
    }
    assert!(bank.insert(40).is_err());
    let y = bank
        .execute(
            &backend,
            &[20, 10],
            1,
            1,
            vec![1.; 2],
            vec![1.; 2],
            vec![4., 2.],
            vec![1.; 2],
            vec![0.5; 2],
        )
        .await?;
    assert_eq!(y, vec![2., 1.]);
    assert_eq!(bank.snapshot(10)?, vec![1.]);
    assert_eq!(bank.snapshot(20)?, vec![2.]);
    // Row order changes. Existing state decays and evolves independently.
    let y = bank
        .execute(
            &backend,
            &[10, 20],
            1,
            1,
            vec![1.; 2],
            vec![1.; 2],
            vec![3., 8.],
            vec![0.5; 2],
            vec![0.5; 2],
        )
        .await?;
    assert_eq!(y, vec![1.75, 4.5]);
    assert!(bank
        .execute(
            &backend,
            &[10, 10],
            1,
            1,
            vec![1.; 2],
            vec![1.; 2],
            vec![1.; 2],
            vec![1.; 2],
            vec![1.; 2]
        )
        .await
        .is_err());
    assert!(bank
        .execute(
            &backend,
            &[10],
            1,
            1,
            vec![f32::NAN],
            vec![1.],
            vec![1.],
            vec![1.],
            vec![1.]
        )
        .await
        .is_err());
    assert_eq!(bank.snapshot(10)?, vec![1.75]);
    assert_eq!(bank.remove(20)?, vec![4.5]);
    bank.insert(40)?;
    assert_eq!(bank.snapshot(40)?, vec![0.]);
    backend.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn mixed_bitstream_nonconstant_values_match_scalar_decode() -> Result<()> {
    let backend = Backend::new(Device::Gpu)?;
    for bits in [2, 3, 4, 5, 6, 8] {
        for group in [32, 64, 128] {
            let k = group * 2;
            let mut packed = vec![0u32; (3 * k * bits / 32) as usize];
            let x: Vec<f32> = (0..2 * k).map(|j| (j % 11 - 5) as f32 / 8.).collect();
            let mut expected = vec![0.; 6];
            for row in 0..3 {
                for j in 0..k {
                    let q = ((j * 7 + row * 3) % (1 << bits)) as u32;
                    let bit = (row * k + j) * bits;
                    let word = (bit / 32) as usize;
                    let shift = bit % 32;
                    packed[word] |= q << shift;
                    if shift + bits > 32 {
                        packed[word + 1] |= q >> (32 - shift);
                    }
                    for batch in 0..2 {
                        expected[(batch * 3 + row) as usize] +=
                            x[(batch * k + j) as usize] * (q as f32 * 0.125 - 0.25);
                    }
                }
            }
            let mut g = Graph::new();
            let a = g.input(&[2, k], Data::F32(x))?;
            let w = g.input(&[3, k * bits / 32], Data::U32(packed))?;
            let scales = g.input(&[3, 2], Data::F32(vec![0.125; 6]))?;
            let biases = g.input(&[3, 2], Data::F32(vec![-0.25; 6]))?;
            let y = g.quantized_matmul(a, w, scales, biases, group, bits)?;
            close(backend.submit(g, y)?.result().await?.data, &expected);
        }
    }
    backend.shutdown().await?;
    Ok(())
}
#[test]
fn metadata_duplicates_overlap_and_symlink_fail_closed() -> Result<()> {
    let f = Fixture::new()?;
    fs::write(
        f.0.join("config.json"),
        r#"{"quantization":{"bits":4,"bits":8,"group_size":32}}"#,
    )?;
    assert!(Manifest::read(&f.0).is_err());
    fs::write(
        f.0.join("config.json"),
        r#"{"quantization":{"bits":4,"group_size":32}}"#,
    )?;
    let header = br#"{"value":{"dtype":"F32","dtype":"I32","shape":[2,2],"data_offsets":[0,16]}}"#;
    let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
    bytes.extend(header);
    bytes.extend([0u8; 16]);
    fs::write(f.0.join("model.safetensors"), bytes)?;
    assert!(Manifest::read(&f.0).is_err());
    std::os::unix::fs::symlink("config.json", f.0.join("link"))?;
    assert!(Manifest::read(&f.0).is_err());
    Ok(())
}
#[tokio::test]
async fn recurrent_chunking_equals_full_sequence_and_cancellation_rolls_back() -> Result<()> {
    use rust_server::serving::native_state::RecurrentBank;
    let backend = Backend::new(Device::Gpu)?;
    let full = RecurrentBank::new(&backend, [2, 1, 2], 1)?;
    full.insert(1)?;
    let chunk = RecurrentBank::new(&backend, [2, 1, 2], 1)?;
    chunk.insert(1)?;
    let q = vec![0.5, 0., 0., 0.5];
    let k = vec![1., 0., 0., 1.];
    let v = vec![2., 4., 6., 8.];
    let expected = full
        .execute(
            &backend,
            &[1],
            2,
            1,
            q.clone(),
            k.clone(),
            v.clone(),
            vec![0.5; 4],
            vec![0.5; 4],
        )
        .await?;
    let mut actual = Vec::new();
    for t in 0..2 {
        actual.extend(
            chunk
                .execute(
                    &backend,
                    &[1],
                    1,
                    1,
                    q[t * 2..t * 2 + 2].to_vec(),
                    k[t * 2..t * 2 + 2].to_vec(),
                    v[t * 2..t * 2 + 2].to_vec(),
                    vec![0.5; 2],
                    vec![0.5; 2],
                )
                .await?,
        );
    }
    assert_eq!(actual, expected);
    assert_eq!(chunk.snapshot(1)?, full.snapshot(1)?);
    let before = chunk.snapshot(1)?;
    let mut future = Box::pin(chunk.execute(
        &backend,
        &[1],
        1,
        1,
        vec![0.5, 0.5],
        vec![1., 0.],
        vec![2., 3.],
        vec![0.5; 2],
        vec![0.5; 2],
    ));
    let waker = std::task::Waker::noop();
    let mut context = std::task::Context::from_waker(waker);
    use std::future::Future;
    let result = future.as_mut().poll(&mut context);
    if result.is_pending() {
        drop(future);
        assert_eq!(chunk.snapshot(1)?, before);
        chunk.remove(1)?;
    } else {
        drop(future);
    }
    let stats = backend.shutdown().await?;
    assert_eq!(stats.resources.leases, 0);
    Ok(())
}
#[tokio::test]
async fn changed_checkpoint_closes_admission_before_parameter_evaluation() -> Result<()> {
    let f = Fixture::new()?;
    let backend = Backend::with_checkpoint(Device::Gpu, f.0.clone()).await?;
    let mut graph = Graph::new();
    let output = graph.parameter(backend.model().unwrap(), "value")?;
    fs::write(f.0.join("unexpected"), b"mutation")?;
    assert!(backend.submit(graph, output)?.result().await.is_err());
    assert!(backend.stats().admission_closed);
    let stats = backend.shutdown().await?;
    assert_eq!(stats.resources.leases, 0);
    assert_eq!(stats.parameter_bytes, 0);
    Ok(())
}
#[tokio::test]
async fn recurrent_pressure_matches_independent_scalar_state_per_sequence() -> Result<()> {
    use rust_server::serving::native_state::RecurrentBank;
    let backend = Backend::new(Device::Gpu)?;
    let bank = RecurrentBank::new(&backend, [2, 2, 3], 3)?;
    for id in 0..3 {
        bank.insert(id)?;
    }
    let mut states = vec![vec![0f32; 12]; 3];
    for round in 0..100 {
        let ids: Vec<u64> = (0..3).map(|i| ((i + round) % 3) as u64).collect();
        let q = [0.25, -0.125, 0.375];
        let k = [0.5, 0.25, -0.25];
        let mut values = Vec::new();
        let mut oracle = Vec::new();
        for id in &ids {
            let row: Vec<f32> = (0..4)
                .map(|j| ((*id as usize * 7 + round + j) % 13) as f32 / 16.)
                .collect();
            values.extend_from_slice(&row);
            let state = &mut states[*id as usize];
            for head in 0..2 {
                for value in 0..2 {
                    let offset = (head * 2 + value) * 3;
                    for d in 0..3 {
                        state[offset + d] *= 0.875;
                    }
                    let memory = (0..3).map(|d| state[offset + d] * k[d]).sum::<f32>();
                    let delta = (row[head * 2 + value] - memory) * 0.625;
                    for d in 0..3 {
                        state[offset + d] += k[d] * delta;
                    }
                    oracle.push((0..3).map(|d| state[offset + d] * q[d]).sum::<f32>());
                }
            }
        }
        let output = bank
            .execute(
                &backend,
                &ids,
                1,
                1,
                q.repeat(3),
                k.repeat(3),
                values,
                vec![0.875; 6],
                vec![0.625; 6],
            )
            .await?;
        close(Data::F32(output), &oracle);
    }
    for (id, state) in states.iter().enumerate() {
        close(Data::F32(bank.snapshot(id as u64)?), state);
    }
    backend.shutdown().await?;
    Ok(())
}
