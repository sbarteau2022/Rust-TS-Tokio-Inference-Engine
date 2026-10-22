#![cfg(feature = "legacy-python")]
use anyhow::Result;
use rust_server::serving::{
    rapid::{RapidBackend, RapidConfig},
    scheduler::Engine,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static ID: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rust-serving-rpc-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn config(&self) -> RapidConfig {
        let mut c: RapidConfig =
            serde_json::from_str(include_str!("../fixtures/qwen-serving.json")).unwrap();
        c.python = PathBuf::from("/usr/bin/python3");
        c.model = self.0.clone();
        c.rpc_timeout_seconds = 1;
        c
    }
    fn script(&self, body: &str) -> PathBuf {
        let p = self.0.join("worker.py");
        std::fs::write(&p, body).unwrap();
        p
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
#[tokio::test]
async fn malformed_backend_readiness_is_bounded_and_rejected() -> Result<()> {
    for body in ["print('not json',flush=True)","print('x'*2200000,flush=True)","pass","import time;time.sleep(3)","import json;print(json.dumps({'ready':True,'identity':'test','vocabulary_size':100,'eos_tokens':[],'prompts':[[1]],'codec_fingerprint':'é'*32}),flush=True)"] {
    let temp=Temp::new();let cfg=temp.config();let script=temp.script(body);
    assert!(RapidBackend::start(&cfg,&script,&["hello".into()],None).await.is_err());
 }
    Ok(())
}
#[tokio::test]
async fn managed_backend_state_release_and_bad_wire_shapes() -> Result<()> {
    for fault in ["ok", "oversized-state", "bad-kv", "bad-release"] {
        let temp = Temp::new();
        let cfg = temp.config();
        let script=temp.script(&format!(r#"
import json,sys
init=json.loads(sys.stdin.readline())
print(json.dumps({{'ready':True,'identity':'SYNTHETIC-NONMODEL-RPC','vocabulary_size':100,'eos_tokens':[],'prompts':[[1]],'codec_fingerprint':'a'*64}}),flush=True)
for line in sys.stdin:
    r=json.loads(line)
    if r['op']=='step':
        out=[{{'sequence':i['sequence'],'kv':[] if '{fault}'!='bad-kv' else [{{}}],'next_token':2,'state_bytes':0 if '{fault}'!='oversized-state' else init['sequence_bytes']+1}} for i in r['batch']]
        print(json.dumps({{'outputs':out}}),flush=True)
    elif r['op']=='release':print(json.dumps({{'released':'{fault}'!='bad-release'}}),flush=True)
"#));
        let (backend, storage, prompts, _, _report) =
            RapidBackend::start(&cfg, &script, &["hello".into()], None).await?;
        let engine = Engine::new(cfg.scheduler, storage, backend)?;
        let result = engine.submit(prompts[0].clone(), 1)?.result().await;
        assert_eq!(result.is_ok(), fault == "ok");
        let stats = engine.shutdown().await?;
        assert_eq!(stats.memory_used, 0);
        assert_eq!(stats.admission_closed, fault != "ok");
    }
    Ok(())
}
