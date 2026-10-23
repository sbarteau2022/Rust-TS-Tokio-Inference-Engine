"""Offline HTTP qualification driver; model execution remains Rust/native MLX."""
from pathlib import Path
import concurrent.futures, hashlib, http.client, json, signal, socket, struct, subprocess, time
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'evidence/native-seed/http'
OUT.mkdir(parents=True,exist_ok=True)
# This parser is for test observations only; Rust and TypeScript independently validate saved bodies.
def frames(raw):
    result=[];pos=0
    while pos<len(raw):
        assert pos+4<=len(raw)
        n=struct.unpack_from('<I',raw,pos)[0];end=pos+4+n;assert end<=len(raw)
        b=raw[pos:end];assert b[8:12]==b'AMST'
        root=4+struct.unpack_from('<I',b,4)[0];vt=root-struct.unpack_from('<i',b,root)[0];vlen=struct.unpack_from('<H',b,vt)[0]
        def field(i,fmt,default=0):
            at=4+2*i
            if at>=vlen:return default
            off=struct.unpack_from('<H',b,vt+at)[0]
            return struct.unpack_from(fmt,b,root+off)[0] if off else default
        result.append({'kind':field(1,'<B'),'sequence':field(2,'<Q'),'index':field(3,'<I'),'token':field(4,'<I'),'elapsed_us':field(5,'<Q'),'count':field(9,'<I'),'error':field(11,'<I')})
        pos=end
    return result
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
flags=['--seed-mode','deferred','--execution-profile','mtp-audited','--draft-width','4','--stream-capacity','1','--qualification-trials','64','--probe-epochs','2','--initial-mass','2147483647','--admission-mass','268435455','--reward-mass','536870911']
command=[str(ROOT/'target/release/rust-server'),'native-mtp-serve',f'127.0.0.1:{port}','1000','8',*flags]
(OUT/'command.json').write_text(json.dumps({'command':command,'model':'/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined','binary_sha256':hashlib.sha256(Path(command[0]).read_bytes()).hexdigest()},indent=2))
log=(OUT/'server.log').open('w');proc=subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
def request(name,maximum=8,deadline=30000,prompt='Complete: 1, 2, 3,',disconnect=False):
    c=http.client.HTTPConnection('127.0.0.1',port,timeout=45);started=time.monotonic()
    c.request('POST','/v1/generate',prompt.encode(),{'Content-Type':'text/plain; charset=utf-8','x-rust-max-new-tokens':str(maximum),'x-rust-deadline-ms':str(deadline)})
    r=c.getresponse()
    if disconnect:
        prefix=bytearray()
        while True:
            header=r.read(4);assert len(header)==4
            size=struct.unpack('<I',header)[0];assert size<=72*1024
            body=r.read(size);assert len(body)==size;prefix.extend(header+body)
            if frames(prefix)[-1]['kind']==1:break
        (OUT/'disconnect-prefix.amst').write_bytes(prefix)
        c.close();return {'status':r.status,'disconnected_after_token':frames(prefix)[-1]['token']}
    raw=r.read();elapsed=time.monotonic()-started;c.close();(OUT/f'{name}.amst').write_bytes(raw)
    if r.status!=200:return {'status':r.status,'wall_seconds':elapsed}
    fs=frames(raw);return {'status':r.status,'wall_seconds':elapsed,'frames':fs,'tokens':[f['token'] for f in fs if f['kind']==1]}
try:
    until=time.monotonic()+180
    while True:
        if proc.poll() is not None:raise RuntimeError(f'server exited {proc.returncode}')
        if 'READY http://' in (OUT/'server.log').read_text():break
        if time.monotonic()>until:raise TimeoutError('server readiness')
        time.sleep(.1)
    c=http.client.HTTPConnection('127.0.0.1',port,timeout=10);c.request('GET','/health');r=c.getresponse();assert r.status==200;r.read();c.close()
    c=http.client.HTTPConnection('127.0.0.1',port,timeout=10);c.request('GET','/v1/info');r=c.getresponse();assert r.status==200;info=r.read();c.close();(OUT/'info.amst').write_bytes(info);assert [f['kind'] for f in frames(info)]==[0];assert b'seed_mode=deferred' in info
    baseline=request('single',16);assert baseline['status']==200 and baseline['frames'][-1]['kind']==2 and len(baseline['tokens'])==16
    reference=json.loads((ROOT/'evidence/native-mtp-live-audit/smoke/summary.json').read_text())['generations'][-1]['tokens']
    assert baseline['tokens']==reference, 'HTTP changed previously recorded native token reference'
    with concurrent.futures.ThreadPoolExecutor(3) as pool:lanes=list(pool.map(lambda n:request(f'lane-{n}',8),range(3)))
    for lane in lanes:assert lane['status']==200 and lane['frames'][-1]['kind']==2 and lane['tokens']==baseline['tokens'][:8]
    deadline=request('deadline',128,10);assert deadline['status']==504 or deadline['status']==200 and deadline['frames'][-1]['kind']==3 and deadline['frames'][-1]['error']==2
    disconnected=request('disconnect',128,30000,disconnect=True)
    recovered=request('recovered',8);assert recovered['status']==200 and recovered['frames'][-1]['kind']==2 and recovered['tokens']==baseline['tokens'][:8]
    # Exercise the actual TS 7.1 client over this same persistent server.
    ts="import {generate} from './ts/http-client.ts';let n=0;await generate(process.argv[1],'Complete: 1, 2, 3,',8,30000,f=>{if(f.kind===1)n++});if(n!==8)throw Error('token count');console.log('HTTP_TS_PASS tokens='+n);"
    subprocess.run(['node','--import','tsx','--input-type=module','-e',ts,f'http://127.0.0.1:{port}'],cwd=ROOT,check=True,stdout=(OUT/'ts-client.log').open('w'),stderr=subprocess.STDOUT,timeout=60)
    summary={'pass':True,'scope':'bounded HTTP transport smoke, repeated short prompt; not throughput qualification','single':baseline,'three_lanes':lanes,'deadline':deadline,'disconnect':disconnected,'recovered':recovered}
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2))
finally:
    if proc.poll() is None:proc.send_signal(signal.SIGTERM)
    try:code=proc.wait(timeout=60)
    except subprocess.TimeoutExpired:
        proc.kill();proc.wait();raise RuntimeError('owned HTTP server failed shutdown deadline')
    log.close()
assert code==0,(code,(OUT/'server.log').read_text()[-4000:])
assert 'DRAIN http=' in (OUT/'server.log').read_text() and 'native_bytes=0' in (OUT/'server.log').read_text()
for args,label in [([command[0],'stream-check'],'rust'),(['node','--import','tsx','ts/stream-inspect.ts'],'ts')]:
    with (OUT/f'disconnect-prefix.{label}.txt').open('w') as out:
        result=subprocess.run([*args,str(OUT/'disconnect-prefix.amst')],cwd=ROOT,stdout=out,stderr=subprocess.STDOUT,timeout=10)
        assert result.returncode!=0,'unfinished disconnect prefix accepted'
for name in ['single','lane-0','lane-1','lane-2','recovered']:
    with (OUT/f'{name}.rust.txt').open('w') as out:subprocess.run([command[0],'stream-check',str(OUT/f'{name}.amst')],cwd=ROOT,check=True,stdout=out,stderr=subprocess.STDOUT,timeout=10)
    with (OUT/f'{name}.ts.txt').open('w') as out:subprocess.run(['node','--import','tsx','ts/stream-inspect.ts',str(OUT/f'{name}.amst')],cwd=ROOT,check=True,stdout=out,stderr=subprocess.STDOUT,timeout=10)
import re
seed=re.search(r'SEED_METRICS SeedSnapshot \{ prefill_seed_receipts: (\d+), advance_attempts: (\d+), advance_completed: (\d+), pending_releases: (\d+) \}',(OUT/'server.log').read_text());assert seed is not None
receipts,attempts,completed,pending=map(int,seed.groups());assert attempts==completed and receipts==completed+pending and completed>=6
print('PASS deferred seed native HTTP single/three-lane parity, deadline, disconnect/recovery, TS client, both binary readers, drained shutdown')
