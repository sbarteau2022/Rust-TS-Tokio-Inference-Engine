use anyhow::Result;
use rust_server::serving::{config::Config, scheduler::*};
struct Backend {
    supports: bool,
    violate: bool,
}
impl ModelBackend for Backend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: "SYNTHETIC-choice-oracle".into(),
            width: 0,
            vocabulary_size: 10,
            max_context_tokens: 64,
            workspace_bytes: 32,
            max_in_flight: 3,
            eos_tokens: vec![9],
        }
    }
    fn supports_choices(&self) -> bool {
        self.supports
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            Ok(batch
                .into_iter()
                .map(|i| StepOutput {
                    sequence: i.sequence,
                    kv: vec![],
                    state_bytes: 16,
                    next_token: i.prompt_complete.then(|| {
                        if self.violate {
                            8
                        } else {
                            i.allowed_tokens.as_ref().map_or(9, |v| v[0])
                        }
                    }),
                })
                .collect())
        })
    }
    fn release(&mut self, _: u64) -> ReleaseFuture<'_> {
        Box::pin(async { Ok(()) })
    }
}
fn engine(supports: bool, violate: bool) -> Result<Engine> {
    Engine::new(
        Config {
            version: 1,
            max_queued: 3,
            max_sequences: 3,
            max_batch_tokens: 8,
            prefill_chunk_tokens: 2,
            max_context_tokens: 64,
            max_new_tokens: 8,
            model_memory_bytes: 1024,
            max_in_flight: 3,
        },
        Storage::BackendManaged {
            fingerprint: [0; 32],
            sequence_bytes: 16,
            experimental: false,
        },
        Backend { supports, violate },
    )
}
#[tokio::test]
async fn isolated_contracts_eos_receipts_and_cancelled_rows() -> Result<()> {
    let e = engine(true, false)?;
    let a = e.submit_choices(vec![1, 2, 3], 4, vec![vec![2, 3]])?;
    let cancelled = e.submit_choices(vec![4], 4, vec![vec![6]])?;
    cancelled.cancel();
    drop(cancelled);
    let b = e.submit_choices(vec![1], 4, vec![vec![5]])?;
    let a = a.result().await?;
    let b = b.result().await?;
    assert_eq!(a.tokens, vec![2, 3, 9]);
    assert_eq!(b.tokens, vec![5, 9]);
    assert_eq!(a.finish_reason, "eos");
    assert_ne!(a.output_contract_fingerprint, b.output_contract_fingerprint);
    assert!(a.output_contract_fingerprint.is_some());
    let raw = e.submit(vec![1], 2)?.result().await?;
    assert!(raw.output_contract_fingerprint.is_none());
    let stats = e.shutdown().await?;
    assert_eq!(stats.cancelled, 1);
    assert_eq!(stats.memory_used, 0);
    assert!(!stats.admission_closed);
    Ok(())
}
#[tokio::test]
async fn unsupported_invalid_and_lying_backends_fail_closed() -> Result<()> {
    let unsupported = engine(false, false)?;
    assert!(unsupported
        .submit_choices(vec![1], 4, vec![vec![2]])
        .is_err());
    unsupported.shutdown().await?;
    let e = engine(true, true)?;
    assert!(e.submit_choices(vec![1], 2, vec![vec![2, 3]]).is_err());
    assert_eq!(e.stats().admitted, 0);
    assert!(e
        .submit_choices(vec![1], 4, vec![vec![2]])?
        .result()
        .await
        .is_err());
    assert!(e.submit(vec![1], 2).is_err());
    let stats = e.shutdown().await?;
    assert!(stats.admission_closed);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.memory_used, 0);
    Ok(())
}
