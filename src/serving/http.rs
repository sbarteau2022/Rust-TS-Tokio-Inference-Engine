//! Experimental loopback HTTP/1 transport. Native state remains owned by Engine.
use super::{
    scheduler::{Engine, Request as ModelRequest},
    stream_wire::Frame as WireFrame,
};
use anyhow::{ensure, Context, Result};
use bytes::Bytes;
use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Full, Limited, StreamBody};
use hyper::{
    body::{Frame, Incoming},
    header,
    service::service_fn,
    Method, Request, Response, StatusCode,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot, watch, Notify, Semaphore},
    task::{JoinHandle, JoinSet},
    time::{sleep_until, timeout, Instant},
};
use tokio_stream::wrappers::ReceiverStream;
pub trait TextOutput: Send {
    fn push(&mut self, id: u32) -> Result<Option<String>>;
    fn finish(&mut self) -> Result<String>;
    fn text(&self) -> &str;
}
pub trait TextCodec: Send + Sync + 'static {
    fn identity(&self) -> &str;
    fn prepare(&self, text: &str, max_new: usize) -> Result<Vec<u32>>;
    fn stream(&self) -> Result<Box<dyn TextOutput + '_>>;
    fn decode(&self, ids: &[u32]) -> Result<String>;
}
#[derive(Clone, Debug)]
pub struct Config {
    pub max_connections: usize,
    pub body_timeout_ms: u64,
    pub shutdown_grace_ms: u64,
    pub stream_capacity: usize,
    pub profile: String,
}
impl Config {
    fn validate(&self) -> Result<()> {
        ensure!(
            (1..=64).contains(&self.max_connections)
                && (1..=30000).contains(&self.body_timeout_ms)
                && (100..=10000).contains(&self.shutdown_grace_ms)
                && (1..=128).contains(&self.stream_capacity),
            "HTTP bounds"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Stats {
    pub active_connections: usize,
    pub active_producers: usize,
    pub accepted_connections: usize,
    pub rejected_requests: usize,
    pub queued_finishes: usize,
    pub deadlines: usize,
    pub disconnected: usize,
    pub stopped: usize,
    pub producer_errors: usize,
    pub connection_errors: usize,
}
#[derive(Default)]
struct Counts {
    connections: AtomicUsize,
    producers: AtomicUsize,
    accepted: AtomicUsize,
    rejected: AtomicUsize,
    finished: AtomicUsize,
    deadlines: AtomicUsize,
    disconnected: AtomicUsize,
    stopped: AtomicUsize,
    errors: AtomicUsize,
    conn_errors: AtomicUsize,
}
struct App {
    engine: Arc<Engine>,
    codec: Arc<dyn TextCodec>,
    config: Config,
    authority: String,
    stop: watch::Sender<bool>,
    stopping: AtomicBool,
    counts: Counts,
    idle: Notify,
}
impl App {
    fn stats(&self) -> Stats {
        let c = &self.counts;
        let get = |v: &AtomicUsize| v.load(Ordering::Acquire);
        Stats {
            active_connections: get(&c.connections),
            active_producers: get(&c.producers),
            accepted_connections: get(&c.accepted),
            rejected_requests: get(&c.rejected),
            queued_finishes: get(&c.finished),
            deadlines: get(&c.deadlines),
            disconnected: get(&c.disconnected),
            stopped: get(&c.stopped),
            producer_errors: get(&c.errors),
            connection_errors: get(&c.conn_errors),
        }
    }
}
struct Active {
    app: Arc<App>,
    producer: bool,
}
impl Drop for Active {
    fn drop(&mut self) {
        if self.producer {
            if std::thread::panicking() {
                self.app.counts.errors.fetch_add(1, Ordering::Relaxed);
            }
            self.app.counts.producers.fetch_sub(1, Ordering::AcqRel);
            self.app.idle.notify_one();
        } else {
            self.app.counts.connections.fetch_sub(1, Ordering::AcqRel);
        }
    }
}
pub struct Server {
    pub address: SocketAddr,
    app: Arc<App>,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<Stats>>>,
}
impl Server {
    pub async fn bind(
        address: SocketAddr,
        engine: Arc<Engine>,
        codec: Arc<dyn TextCodec>,
        config: Config,
    ) -> Result<Self> {
        ensure!(address.ip().is_loopback(), "HTTP listener must be loopback");
        config.validate()?;
        WireFrame::metadata(codec.identity().into(), config.profile.clone()).encode()?;
        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let (signal, _) = watch::channel(false);
        let app = Arc::new(App {
            engine,
            codec,
            config,
            authority: address.to_string(),
            stop: signal,
            stopping: AtomicBool::new(false),
            counts: Counts::default(),
            idle: Notify::new(),
        });
        let (stop, rx) = oneshot::channel();
        let task = tokio::spawn(run(listener, app.clone(), rx));
        Ok(Self {
            address,
            app,
            stop: Some(stop),
            task: Some(task),
        })
    }
    pub fn stats(&self) -> Stats {
        self.app.stats()
    }
    pub async fn shutdown(mut self) -> Result<Stats> {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
        self.task.take().unwrap().await?
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
    }
}
async fn stopped(rx: &mut watch::Receiver<bool>) {
    if *rx.borrow() {
        return;
    }
    let _ = rx.changed().await;
}
async fn run(
    listener: TcpListener,
    app: Arc<App>,
    mut stop: oneshot::Receiver<()>,
) -> Result<Stats> {
    let permits = Arc::new(Semaphore::new(app.config.max_connections));
    let mut connections = JoinSet::new();
    let mut accept_error = None;
    loop {
        tokio::select! {biased;
         _=&mut stop=>break,
         Some(result)=connections.join_next(),if !connections.is_empty()=>{if !matches!(result,Ok(true)){app.counts.conn_errors.fetch_add(1,Ordering::Relaxed);}},
         permit=permits.clone().acquire_owned()=>{
          let permit=permit?;
          let accepted=tokio::select!{biased;_=&mut stop=>None,r=listener.accept()=>Some(r)};
          let (socket,_)=match accepted.transpose(){Ok(Some(v))=>v,Ok(None)=>break,Err(error)=>{accept_error=Some(error);break}};
          app.counts.connections.fetch_add(1,Ordering::AcqRel);app.counts.accepted.fetch_add(1,Ordering::Relaxed);
          let app=app.clone();connections.spawn(async move{
           let _permit=permit;let _active=Active{app:app.clone(),producer:false};let mut rx=app.stop.subscribe();
           let mut builder=hyper::server::conn::http1::Builder::new();builder.keep_alive(false).max_headers(32).max_buf_size(16*1024).timer(TokioTimer::new()).header_read_timeout(Duration::from_millis(app.config.body_timeout_ms));
           let (cutoff,mut changed)=watch::channel(Instant::now()+Duration::from_millis(app.config.body_timeout_ms+100));
           let state=app.clone();let connection=builder.serve_connection(TokioIo::new(socket),service_fn(move|request|handle(state.clone(),request,cutoff.clone())));
           tokio::pin!(connection);
           loop {let deadline=*changed.borrow_and_update();tokio::select!{biased;
            _=stopped(&mut rx)=>{connection.as_mut().graceful_shutdown();break matches!(timeout(Duration::from_millis(app.config.shutdown_grace_ms),&mut connection).await,Ok(Ok(())));},
            r=&mut connection=>break r.is_ok(),
            _=sleep_until(deadline)=>break false,
            _=changed.changed()=>{}
           }}
          });
         }
        }
    }
    drop(listener);
    app.stopping.store(true, Ordering::Release);
    app.stop.send_replace(true);
    while let Some(result) = connections.join_next().await {
        if !matches!(result, Ok(true)) {
            app.counts.conn_errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    timeout(Duration::from_millis(app.config.shutdown_grace_ms), async {
        loop {
            let notified = app.idle.notified();
            if app.counts.producers.load(Ordering::Acquire) == 0 {
                break;
            }
            notified.await;
        }
    })
    .await
    .context("HTTP producers failed to drain")?;
    if let Some(error) = accept_error {
        return Err(error.into());
    }
    Ok(app.stats())
}
type Body = UnsyncBoxBody<Bytes, std::io::Error>;
type Output = mpsc::Sender<std::result::Result<Frame<Bytes>, std::io::Error>>;
fn response(status: StatusCode, body: impl Into<Bytes>) -> Response<Body> {
    let mut r = Response::new(
        Full::new(body.into())
            .map_err(|never: Infallible| match never {})
            .boxed_unsync(),
    );
    *r.status_mut() = status;
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    r.headers_mut().insert(
        header::CONNECTION,
        header::HeaderValue::from_static("close"),
    );
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    r
}
fn reject(app: &App, status: StatusCode, message: &'static str) -> Response<Body> {
    app.counts.rejected.fetch_add(1, Ordering::Relaxed);
    response(status, message)
}
fn single<'a>(headers: &'a hyper::HeaderMap, key: &str) -> Result<&'a str> {
    ensure!(
        headers.get_all(key).iter().count() == 1,
        "singleton header required"
    );
    Ok(headers.get(key).unwrap().to_str()?)
}
fn controls(request: &Request<Incoming>, authority: &str) -> Result<(usize, u64)> {
    ensure!(
        request.uri().query().is_none()
            && request.uri().scheme().is_none()
            && request.uri().authority().is_none(),
        "origin-form target only"
    );
    ensure!(
        single(request.headers(), "host")? == authority
            && !request.headers().contains_key("origin")
            && !request.headers().contains_key("content-encoding"),
        "unsupported authority/origin/encoding"
    );
    for key in request.headers().keys() {
        if key.as_str().starts_with("x-rust-") {
            ensure!(
                matches!(key.as_str(), "x-rust-max-new-tokens" | "x-rust-deadline-ms"),
                "unknown control"
            );
        }
    }
    if request.method() == Method::POST && request.uri().path() == "/v1/generate" {
        let content = single(request.headers(), "content-type")?.to_ascii_lowercase();
        ensure!(
            matches!(content.trim(), "text/plain" | "text/plain; charset=utf-8"),
            "unsupported content type"
        );
        let max = single(request.headers(), "x-rust-max-new-tokens")?;
        let deadline = single(request.headers(), "x-rust-deadline-ms")?;
        ensure!(
            !max.is_empty()
                && !deadline.is_empty()
                && max.bytes().all(|b| b.is_ascii_digit())
                && deadline.bytes().all(|b| b.is_ascii_digit()),
            "decimal controls only"
        );
        let (max, deadline) = (max.parse::<usize>()?, deadline.parse::<u64>()?);
        ensure!(
            (1..=128).contains(&max) && (1..=300000).contains(&deadline),
            "request bounds"
        );
        Ok((max, deadline))
    } else {
        Ok((0, 0))
    }
}
async fn handle(
    app: Arc<App>,
    request: Request<Incoming>,
    cutoff: watch::Sender<Instant>,
) -> std::result::Result<Response<Body>, Infallible> {
    let response = handle_inner(app, request, cutoff.clone()).await;
    if !response.headers().contains_key("x-rust-stream") {
        cutoff.send_replace(Instant::now() + Duration::from_millis(100));
    }
    Ok(response)
}
async fn handle_inner(
    app: Arc<App>,
    request: Request<Incoming>,
    cutoff: watch::Sender<Instant>,
) -> Response<Body> {
    let entered = Instant::now();
    if app.stopping.load(Ordering::Acquire) {
        return reject(&app, StatusCode::SERVICE_UNAVAILABLE, "stopping\n");
    }
    let (max, deadline_ms) = match controls(&request, &app.authority) {
        Ok(v) => v,
        Err(_) => return reject(&app, StatusCode::BAD_REQUEST, "invalid request controls\n"),
    };
    if request.method() == Method::GET && request.uri().path() == "/health" {
        return if app.engine.stats().admission_closed {
            response(StatusCode::SERVICE_UNAVAILABLE, "admission closed\n")
        } else {
            response(StatusCode::OK, "ready\n")
        };
    }
    if request.method() == Method::GET && request.uri().path() == "/v1/info" {
        let frame = WireFrame::metadata(app.codec.identity().into(), app.config.profile.clone())
            .encode()
            .expect("validated metadata");
        let mut r = response(StatusCode::OK, frame);
        r.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/vnd.rust-server.amst; version=1"),
        );
        return r;
    }
    if request.method() != Method::POST || request.uri().path() != "/v1/generate" {
        return reject(&app, StatusCode::NOT_FOUND, "unknown route\n");
    }
    if let Some(n) = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
    {
        if n > 65536 {
            return reject(&app, StatusCode::PAYLOAD_TOO_LARGE, "body cap\n");
        }
    }
    let deadline = entered + Duration::from_millis(deadline_ms);
    cutoff.send_replace(deadline + Duration::from_millis(100));
    let body_until =
        deadline.min(Instant::now() + Duration::from_millis(app.config.body_timeout_ms));
    let body = match tokio::time::timeout_at(
        body_until,
        Limited::new(request.into_body(), 65536).collect(),
    )
    .await
    {
        Ok(Ok(b)) => b.to_bytes(),
        Ok(Err(_)) => {
            return reject(
                &app,
                StatusCode::PAYLOAD_TOO_LARGE,
                "body cap/read failure\n",
            )
        }
        Err(_) => {
            return reject(
                &app,
                if Instant::now() >= deadline {
                    StatusCode::GATEWAY_TIMEOUT
                } else {
                    StatusCode::REQUEST_TIMEOUT
                },
                "body deadline\n",
            )
        }
    };
    let text = match std::str::from_utf8(&body) {
        Ok(t) if !t.is_empty() => t,
        _ => return reject(&app, StatusCode::BAD_REQUEST, "UTF-8 text required\n"),
    };
    let ids = match app.codec.prepare(text, max) {
        Ok(v) => v,
        Err(_) => return reject(&app, StatusCode::BAD_REQUEST, "prompt/context rejected\n"),
    };
    if Instant::now() >= deadline {
        return reject(&app, StatusCode::GATEWAY_TIMEOUT, "request deadline\n");
    }
    if app.stopping.load(Ordering::Acquire) {
        return reject(&app, StatusCode::SERVICE_UNAVAILABLE, "stopping\n");
    }
    let request = match app
        .engine
        .submit_stream(ids, max, app.config.stream_capacity)
    {
        Ok(r) => r,
        Err(_) => {
            return reject(
                &app,
                StatusCode::SERVICE_UNAVAILABLE,
                "admission unavailable\n",
            )
        }
    };
    let (tx, rx) = mpsc::channel(1);
    app.counts.producers.fetch_add(1, Ordering::AcqRel);
    let active = Active {
        app: app.clone(),
        producer: true,
    };
    tokio::spawn(async move {
        let _active = active;
        produce(app, request, tx, deadline).await;
    });
    let mut r = Response::new(StreamBody::new(ReceiverStream::new(rx)).boxed_unsync());
    r.headers_mut()
        .insert("x-rust-stream", header::HeaderValue::from_static("1"));
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/vnd.rust-server.amst; version=1"),
    );
    r.headers_mut().insert(
        header::CONNECTION,
        header::HeaderValue::from_static("close"),
    );
    r.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    r
}
#[derive(Clone, Copy, Debug)]
enum Stop {
    Deadline,
    Disconnected,
    Shutdown,
    Backend,
}
async fn send(
    tx: &Output,
    frame: WireFrame,
    deadline: Instant,
    stop: &mut watch::Receiver<bool>,
) -> std::result::Result<(), Stop> {
    let bytes = frame.encode().map_err(|_| Stop::Backend)?;
    tokio::select! {biased;_=stopped(stop)=>Err(Stop::Shutdown),_=sleep_until(deadline)=>Err(Stop::Deadline),r=tx.send(Ok(Frame::data(Bytes::from(bytes))))=>r.map_err(|_|Stop::Disconnected)}
}
async fn produce(app: Arc<App>, mut request: ModelRequest, tx: Output, deadline: Instant) {
    let mut stop = app.stop.subscribe();
    let (mut sequence, mut count, mut last) = (0u64, 0u32, 0u64);
    let mut metadata = false;
    let mut text_bytes = 0usize;
    let result:std::result::Result<(),Stop>=async{
  send(&tx,WireFrame::metadata(app.codec.identity().into(),app.config.profile.clone()),deadline,&mut stop).await?;metadata=true;
  let mut decoder=app.codec.stream().map_err(|_|Stop::Backend)?;
  loop{
   let permit=tokio::select!{biased;_=stopped(&mut stop)=>return Err(Stop::Shutdown),_=sleep_until(deadline)=>return Err(Stop::Deadline),p=tx.reserve()=>p.map_err(|_|Stop::Disconnected)?};
   let event=tokio::select!{biased;_=stopped(&mut stop)=>return Err(Stop::Shutdown),_=sleep_until(deadline)=>return Err(Stop::Deadline),_=tx.closed()=>return Err(Stop::Disconnected),e=request.next_token()=>e};
   let Some(event)=event else{drop(permit);break};
   let text=decoder.push(event.token).map_err(|_|Stop::Backend)?.unwrap_or_default();
   text_bytes+=text.len();if text_bytes>1024*1024{return Err(Stop::Backend)}
   let frame=WireFrame{kind:1,sequence:event.sequence,index:u32::try_from(event.index).map_err(|_|Stop::Backend)?,token_id:event.token,elapsed_us:(event.elapsed_ms*1000.) as u64,text,..WireFrame::default()};
   let bytes=frame.encode().map_err(|_|Stop::Backend)?;permit.send(Ok(Frame::data(Bytes::from(bytes))));sequence=event.sequence;count+=1;last=frame.elapsed_us;
  }
  let generation=tokio::select!{biased;_=stopped(&mut stop)=>return Err(Stop::Shutdown),_=sleep_until(deadline)=>return Err(Stop::Deadline),_=tx.closed()=>return Err(Stop::Disconnected),r=request.result()=>r.map_err(|_|Stop::Backend)?};
  let tail=decoder.finish().map_err(|_|Stop::Backend)?;if text_bytes+tail.len()>1024*1024{return Err(Stop::Backend)}let text=app.codec.decode(&generation.tokens).map_err(|_|Stop::Backend)?;
  if decoder.text()!=text||generation.tokens.len()!=count as usize{return Err(Stop::Backend)}
  let reason=match generation.finish_reason.as_str(){"length"=>1,"eos"=>2,_=>return Err(Stop::Backend)};
  send(&tx,WireFrame{kind:2,sequence,output_tokens:count,stop:reason,elapsed_us:(generation.elapsed_ms*1000.) as u64,text:tail,..WireFrame::default()},deadline,&mut stop).await?;Ok(())
 }.await;
    // Request's Drop cancels the lane even when the response body disappears mid-step.
    match result {
        Ok(()) => {
            app.counts.finished.fetch_add(1, Ordering::Relaxed);
        }
        Err(reason) => {
            let (code, message) = match reason {
                Stop::Deadline => {
                    app.counts.deadlines.fetch_add(1, Ordering::Relaxed);
                    (2, "request deadline")
                }
                Stop::Disconnected => {
                    app.counts.disconnected.fetch_add(1, Ordering::Relaxed);
                    (3, "client disconnected")
                }
                Stop::Shutdown => {
                    app.counts.stopped.fetch_add(1, Ordering::Relaxed);
                    (3, "server stopping")
                }
                Stop::Backend => {
                    app.counts.errors.fetch_add(1, Ordering::Relaxed);
                    (4, "backend/text failure")
                }
            };
            if metadata && !matches!(reason, Stop::Disconnected) {
                if let Ok(bytes) = (WireFrame {
                    kind: 3,
                    sequence,
                    output_tokens: count,
                    elapsed_us: last,
                    error_code: code,
                    text: message.into(),
                    ..WireFrame::default()
                })
                .encode()
                {
                    let _ = timeout(
                        Duration::from_millis(100),
                        tx.send(Ok(Frame::data(Bytes::from(bytes)))),
                    )
                    .await;
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::serving::{config::Config as ModelConfig, scheduler::*};
    struct B(Arc<AtomicUsize>);
    impl ModelBackend for B {
        fn info(&self) -> BackendInfo {
            BackendInfo {
                identity: "HTTP-backpressure-synthetic".into(),
                width: 0,
                vocabulary_size: 248320,
                max_context_tokens: 64,
                workspace_bytes: 16,
                max_in_flight: 3,
                eos_tokens: vec![],
            }
        }
        fn step(&mut self, b: Vec<StepInput>) -> BackendFuture<'_> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                Ok(b.into_iter()
                    .map(|i| StepOutput {
                        sequence: i.sequence,
                        kv: vec![],
                        next_token: Some(1),
                        state_bytes: 16,
                    })
                    .collect())
            })
        }
        fn release(&mut self, _: u64) -> ReleaseFuture<'_> {
            Box::pin(async { Ok(()) })
        }
    }
    struct C;
    struct T(String);
    impl TextOutput for T {
        fn push(&mut self, _: u32) -> Result<Option<String>> {
            self.0.push('x');
            Ok(Some("x".into()))
        }
        fn finish(&mut self) -> Result<String> {
            Ok(String::new())
        }
        fn text(&self) -> &str {
            &self.0
        }
    }
    impl TextCodec for C {
        fn identity(&self) -> &str {
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }
        fn prepare(&self, _: &str, _: usize) -> Result<Vec<u32>> {
            Ok(vec![1])
        }
        fn stream(&self) -> Result<Box<dyn TextOutput + '_>> {
            Ok(Box::new(T(String::new())))
        }
        fn decode(&self, ids: &[u32]) -> Result<String> {
            Ok("x".repeat(ids.len()))
        }
    }
    #[tokio::test]
    async fn full_network_slot_stops_receiving_scheduler_tokens() -> Result<()> {
        let calls = Arc::new(AtomicUsize::new(0));
        let e = Arc::new(Engine::new(
            ModelConfig {
                version: 1,
                max_queued: 3,
                max_sequences: 3,
                max_batch_tokens: 8,
                prefill_chunk_tokens: 8,
                max_context_tokens: 64,
                max_new_tokens: 32,
                model_memory_bytes: 1024,
                max_in_flight: 3,
            },
            Storage::BackendManaged {
                fingerprint: [0; 32],
                sequence_bytes: 16,
                experimental: true,
            },
            B(calls.clone()),
        )?);
        let (stop, _) = watch::channel(false);
        let app = Arc::new(App {
            engine: e.clone(),
            codec: Arc::new(C),
            config: Config {
                max_connections: 1,
                body_timeout_ms: 100,
                shutdown_grace_ms: 200,
                stream_capacity: 1,
                profile: "backpressure-control".into(),
            },
            authority: "127.0.0.1:1".into(),
            stop,
            stopping: AtomicBool::new(false),
            counts: Counts::default(),
            idle: Notify::new(),
        });
        let request = e.submit_stream(vec![1], 32, 1)?;
        let (tx, mut rx) = mpsc::channel(1);
        let state = app.clone();
        let task = tokio::spawn(async move {
            produce(state, request, tx, Instant::now() + Duration::from_secs(3)).await;
        });
        timeout(Duration::from_secs(1), async {
            while calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        app.stop.send_replace(true);
        timeout(Duration::from_secs(1), task).await??;
        assert_eq!(app.stats().stopped, 1);
        assert!(rx.recv().await.is_some());
        assert!(rx.recv().await.is_none());
        drop(app);
        let stats = Arc::try_unwrap(e)
            .map_err(|_| anyhow::anyhow!("test retained engine"))?
            .shutdown()
            .await?;
        assert_eq!(stats.memory_used, 0);
        assert_eq!(stats.cancelled, 1);
        Ok(())
    }
}
