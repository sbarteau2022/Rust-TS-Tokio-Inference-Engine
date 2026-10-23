use anyhow::{ensure, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, Request, StatusCode};
use hyper_util::rt::TokioIo;
use rust_server::serving::{
    config::Config,
    http::{self, TextCodec, TextOutput},
    scheduler::*,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Notify, Semaphore},
};
struct Codec;
struct Text(String);
impl TextOutput for Text {
    fn push(&mut self, id: u32) -> Result<Option<String>> {
        let s = format!("T{id}");
        self.0.push_str(&s);
        Ok(Some(s))
    }
    fn finish(&mut self) -> Result<String> {
        Ok(String::new())
    }
    fn text(&self) -> &str {
        &self.0
    }
}
impl TextCodec for Codec {
    fn identity(&self) -> &str {
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }
    fn prepare(&self, text: &str, _: usize) -> Result<Vec<u32>> {
        ensure!(text == "hello", "fixture text");
        Ok(vec![1])
    }
    fn stream(&self) -> Result<Box<dyn TextOutput + '_>> {
        Ok(Box::new(Text(String::new())))
    }
    fn decode(&self, ids: &[u32]) -> Result<String> {
        Ok(ids.iter().map(|id| format!("T{id}")).collect())
    }
}
struct Backend {
    next: BTreeMap<u64, u32>,
    calls: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
    started: Arc<Notify>,
    gate: Option<Arc<Semaphore>>,
}
impl ModelBackend for Backend {
    fn info(&self) -> BackendInfo {
        BackendInfo {
            identity: "http-synthetic-control".into(),
            width: 0,
            vocabulary_size: 248320,
            max_context_tokens: 64,
            workspace_bytes: 16,
            max_in_flight: 3,
            eos_tokens: vec![999],
        }
    }
    fn step(&mut self, batch: Vec<StepInput>) -> BackendFuture<'_> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.started.notify_one();
            if let Some(gate) = &self.gate {
                gate.acquire().await?.forget();
            }
            self.completed.fetch_add(1, Ordering::SeqCst);
            Ok(batch
                .into_iter()
                .map(|i| {
                    let token = if i.prompt_complete {
                        let n = self.next.entry(i.sequence).or_insert(0);
                        *n += 1;
                        Some(*n)
                    } else {
                        None
                    };
                    StepOutput {
                        sequence: i.sequence,
                        kv: vec![],
                        next_token: token,
                        state_bytes: 16,
                    }
                })
                .collect())
        })
    }
    fn release(&mut self, id: u64) -> ReleaseFuture<'_> {
        self.next.remove(&id);
        Box::pin(async { Ok(()) })
    }
}
struct Fixture {
    engine: Arc<Engine>,
    server: http::Server,
    calls: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
    started: Arc<Notify>,
}
async fn setup(gate: Option<Arc<Semaphore>>, body_timeout_ms: u64) -> Result<Fixture> {
    let calls = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(Notify::new());
    let backend = Backend {
        next: BTreeMap::new(),
        calls: calls.clone(),
        completed: completed.clone(),
        started: started.clone(),
        gate,
    };
    let engine = Arc::new(Engine::new(
        Config {
            version: 1,
            max_queued: 3,
            max_sequences: 3,
            max_batch_tokens: 8,
            prefill_chunk_tokens: 8,
            max_context_tokens: 64,
            max_new_tokens: 128,
            model_memory_bytes: 1024,
            max_in_flight: 3,
        },
        Storage::BackendManaged {
            fingerprint: [0; 32],
            sequence_bytes: 16,
            experimental: true,
        },
        backend,
    )?);
    let server = http::Server::bind(
        "127.0.0.1:0".parse()?,
        engine.clone(),
        Arc::new(Codec),
        http::Config {
            max_connections: 4,
            body_timeout_ms,
            shutdown_grace_ms: 200,
            stream_capacity: 1,
            profile: "SYNTHETIC-NONMODEL HTTP control".into(),
        },
    )
    .await?;
    Ok(Fixture {
        engine,
        server,
        calls,
        completed,
        started,
    })
}
async fn connect(
    f: &Fixture,
) -> Result<(
    hyper::client::conn::http1::SendRequest<Full<Bytes>>,
    tokio::task::JoinHandle<()>,
)> {
    let stream = TcpStream::connect(f.server.address).await?;
    let (tx, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
    let task = tokio::spawn(async {
        let _ = connection.await;
    });
    Ok((tx, task))
}
fn request(f: &Fixture, max: usize, deadline: u64) -> hyper::http::request::Builder {
    Request::builder()
        .method("POST")
        .uri("/v1/generate")
        .header("host", f.server.address.to_string())
        .header("content-type", "text/plain; charset=utf-8")
        .header("x-rust-max-new-tokens", max)
        .header("x-rust-deadline-ms", deadline)
}
async fn frames(
    response: hyper::Response<Incoming>,
) -> Result<Vec<rust_server::serving::stream_wire::Frame>> {
    ensure!(response.status() == StatusCode::OK, "HTTP status");
    let bytes = response.into_body().collect().await?.to_bytes();
    let mut d = rust_server::serving::stream_wire::Decoder::default();
    let frames = d.push(&bytes)?;
    d.finish()?;
    Ok(frames)
}
async fn finish(f: Fixture) -> Result<()> {
    let stats = f.server.shutdown().await?;
    assert_eq!(stats.active_producers, 0);
    assert_eq!(stats.active_connections, 0);
    let e = Arc::try_unwrap(f.engine)
        .map_err(|_| anyhow::anyhow!("test engine retained"))?
        .shutdown()
        .await?;
    assert_eq!(e.memory_used, 0);
    Ok(())
}
#[tokio::test]
async fn loopback_concurrent_amst_and_discovery() -> Result<()> {
    let f = setup(None, 500).await?;
    let (mut tx, task) = connect(&f).await?;
    let r = tx
        .send_request(
            Request::builder()
                .uri("/v1/info")
                .header("host", f.server.address.to_string())
                .body(Full::new(Bytes::new()))?,
        )
        .await?;
    let bytes = r.into_body().collect().await?.to_bytes();
    assert_eq!(
        rust_server::serving::stream_wire::Frame::decode(&bytes)?.kind,
        0
    );
    task.await?;
    let mut jobs = Vec::new();
    for _ in 0..3 {
        let (mut tx, task) = connect(&f).await?;
        let req = request(&f, 8, 3000).body(Full::new(Bytes::from_static(b"hello")))?;
        jobs.push(tokio::spawn(async move {
            let r = frames(tx.send_request(req).await?).await?;
            task.await?;
            Ok::<_, anyhow::Error>(r)
        }));
    }
    for job in jobs {
        let rows = job.await??;
        assert_eq!(rows.len(), 10);
        assert_eq!(rows.last().unwrap().kind, 2);
        let text: String = rows
            .iter()
            .filter(|r| r.kind == 1)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(text, "T1T2T3T4T5T6T7T8");
    }
    assert_eq!(f.server.stats().queued_finishes, 3);
    finish(f).await
}
#[tokio::test]
async fn invalid_controls_bodies_and_authority_never_admit() -> Result<()> {
    let f = setup(None, 300).await?;
    for variant in 0..9 {
        let (mut tx, task) = connect(&f).await?;
        let mut req = request(&f, 1, 1000).body(Full::new(Bytes::from_static(b"hello")))?;
        match variant {
            0 => {
                req.headers_mut()
                    .insert("origin", "https://evil.example".parse()?);
            }
            1 => {
                req.headers_mut().insert("host", "evil.example".parse()?);
            }
            2 => {
                req.headers_mut()
                    .append("x-rust-deadline-ms", "1000".parse()?);
            }
            3 => {
                req.headers_mut().insert("x-rust-unknown", "1".parse()?);
            }
            4 => {
                req.headers_mut()
                    .insert("content-type", "application/json".parse()?);
            }
            5 => {
                *req.uri_mut() = "/v1/generate?width=8".parse()?;
            }
            6 => {
                *req.body_mut() = Full::new(Bytes::from(vec![255]));
            }
            7 => {
                *req.body_mut() = Full::new(Bytes::from(vec![b'x'; 65537]));
            }
            8 => {
                req.headers_mut()
                    .insert("x-rust-max-new-tokens", "129".parse()?);
            }
            _ => unreachable!(),
        }
        let r = tx.send_request(req).await?;
        assert!(r.status().is_client_error());
        drop(r);
        task.await?;
    }
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
    assert_eq!(f.engine.stats().admitted, 0);
    finish(f).await
}
#[tokio::test]
async fn deadline_and_disconnect_preserve_inflight_ownership() -> Result<()> {
    for disconnect in [false, true] {
        let gate = Arc::new(Semaphore::new(0));
        let f = setup(Some(gate.clone()), 500).await?;
        let (mut tx, task) = connect(&f).await?;
        let response = tx
            .send_request(
                request(&f, 8, if disconnect { 3000 } else { 100 })
                    .body(Full::new(Bytes::from_static(b"hello")))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        tokio::time::timeout(Duration::from_secs(1), f.started.notified()).await?;
        if disconnect {
            drop(response);
            task.abort();
        } else {
            let rows = frames(response).await?;
            assert_eq!(rows.last().unwrap().kind, 3);
            assert_eq!(rows.last().unwrap().error_code, 2);
            task.await?;
        }
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert_eq!(f.completed.load(Ordering::SeqCst), 0);
        assert_eq!(f.calls.load(Ordering::SeqCst), 1);
        gate.add_permits(1);
        finish(f).await?;
    }
    Ok(())
}
#[tokio::test]
async fn slow_headers_body_and_shutdown_are_bounded() -> Result<()> {
    let f = setup(None, 100).await?;
    let mut slow = TcpStream::connect(f.server.address).await?;
    slow.write_all(b"POST /v1/generate HTTP/1.1\r\nHo").await?;
    let mut buf = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), slow.read_to_end(&mut buf)).await??;
    let mut body = TcpStream::connect(f.server.address).await?;
    body.write_all(format!("POST /v1/generate HTTP/1.1\r\nHost: {}\r\nContent-Type: text/plain\r\nx-rust-max-new-tokens: 1\r\nx-rust-deadline-ms: 1000\r\nContent-Length: 100\r\n\r\nh",f.server.address).as_bytes()).await?;
    let mut b = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), body.read_to_end(&mut b)).await??;
    assert!(String::from_utf8_lossy(&b).contains("408"));
    assert_eq!(f.calls.load(Ordering::SeqCst), 0);
    let _stalled = TcpStream::connect(f.server.address).await?;
    tokio::time::timeout(Duration::from_secs(1), finish(f)).await??;
    Ok(())
}
