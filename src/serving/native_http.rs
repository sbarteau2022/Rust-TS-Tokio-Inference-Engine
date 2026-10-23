//! Persistent designated-model loopback server; policy remains explicit and experimental.
use super::{
    http::{self, TextCodec, TextOutput},
    native_text_stream::NativeTextStream,
    native_tokenizer::NativeTokenizer,
};
use anyhow::{ensure, Result};
use std::{net::SocketAddr, sync::Arc};
pub struct Codec {
    tokenizer: NativeTokenizer,
    identity: String,
}
impl Codec {
    pub fn new(tokenizer: NativeTokenizer, identity: String) -> Self {
        Self {
            tokenizer,
            identity,
        }
    }
}
impl TextOutput for NativeTextStream<'_> {
    fn push(&mut self, id: u32) -> Result<Option<String>> {
        self.push(id)
    }
    fn finish(&mut self) -> Result<String> {
        self.finish()
    }
    fn text(&self) -> &str {
        self.text()
    }
}
impl TextCodec for Codec {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn prepare(&self, text: &str, max: usize) -> Result<Vec<u32>> {
        let ids = self.tokenizer.chat(None, text)?;
        ensure!(ids.len() + max <= 512, "HTTP context cap");
        Ok(ids)
    }
    fn stream(&self) -> Result<Box<dyn TextOutput + '_>> {
        Ok(Box::new(self.tokenizer.text_stream()?))
    }
    fn decode(&self, ids: &[u32]) -> Result<String> {
        self.tokenizer.decode(ids)
    }
}
pub async fn serve(
    address: SocketAddr,
    body_timeout_ms: u64,
    max_connections: usize,
    flags: &[String],
) -> Result<()> {
    use super::{
        config::Config,
        native_mlx::Device,
        native_mtp_cli::Options,
        native_qwen::{QwenDeviceExecutor, ScheduledMtp},
        scheduler::Engine,
        weights::Manifest,
    };
    ensure!(
        address.ip().is_loopback()
            && address.port() != 0
            && (1..=30000).contains(&body_timeout_ms)
            && (1..=64).contains(&max_connections),
        "explicit loopback address/nonzero port and HTTP limits required"
    );
    let options = Options::parse(flags)?;
    let controller = options.controller()?;
    let manifest = Manifest::read(std::path::Path::new(
        include_str!("../../fixtures/native-model.path").trim(),
    ))?;
    let tokenizer = NativeTokenizer::load(&manifest)?;
    let executor =
        Arc::new(QwenDeviceExecutor::open_mtp(Device::Gpu, 64, 1024 * 1024 * 1024).await?);
    let backend = ScheduledMtp::new(
        controller.clone(),
        executor.clone(),
        tokenizer.eos(),
        options.width,
        "local-native-http".into(),
    )?
    .with_exclusive_metadata_audits();
    let profile=format!("EXPERIMENTAL-NATIVE-HTTP width={} stream_capacity={} policy={:?} context=511 in_flight=3 body_timeout_ms={body_timeout_ms} max_connections={max_connections} physical_tensor_batching=false production_qualified=false audit_clock=validated-batches",options.width,options.capacity,options.policy);
    let engine = Arc::new(Engine::new(
        Config {
            version: 1,
            max_queued: 3,
            max_sequences: 3,
            max_batch_tokens: 24,
            prefill_chunk_tokens: 8,
            max_context_tokens: 511,
            max_new_tokens: 128,
            model_memory_bytes: 1024 * 1024 * 1024,
            max_in_flight: 3,
        },
        backend.storage(),
        backend,
    )?);
    let codec = Arc::new(Codec::new(tokenizer, executor.identity().into()));
    let server = http::Server::bind(
        address,
        engine.clone(),
        codec,
        http::Config {
            max_connections,
            body_timeout_ms,
            shutdown_grace_ms: 2000,
            stream_capacity: options.capacity,
            profile,
        },
    )
    .await?;
    println!(
        "READY http://{} identity={}",
        server.address,
        executor.identity()
    );
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {r=tokio::signal::ctrl_c()=>r?,_=terminate.recv()=>{}}
    let transport = server.shutdown().await;
    let scheduler = Arc::try_unwrap(engine)
        .map_err(|_| anyhow::anyhow!("HTTP engine retained"))?
        .shutdown()
        .await;
    let native = Arc::try_unwrap(executor)
        .map_err(|_| anyhow::anyhow!("HTTP executor retained"))?
        .shutdown()
        .await;
    let transport = transport?;
    let scheduler = scheduler?;
    let native = native?;
    ensure!(
        transport.active_producers == 0
            && transport.active_connections == 0
            && scheduler.memory_used == 0
            && native.logical_bytes == 0,
        "HTTP shutdown drain"
    );
    println!(
        "DRAIN http={transport:?} scheduler={scheduler:?} native_bytes={} audit_epochs={}",
        native.logical_bytes,
        controller.stats()?.epoch
    );
    Ok(())
}
