"""Offline compiled-path gate audit; live runners remain the numerical oracle."""
import json,math,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def verify(base):
    base=Path(base);read=lambda p:json.loads((base/p).read_text())
    gold=[json.loads(s) for s in (ROOT/'evidence/native-mtp/preservation/suite.jsonl').read_text().splitlines()]
    rows=[json.loads(s) for s in (base/'heldout/suite.jsonl').read_text().splitlines()]
    assert len(rows)==len(gold)==60
    for a,b in zip(rows,gold):
        for k in ('suite','id','prompt_ids','generated','finish','text','quality_pass'):assert a[k]==b[k],k
        assert len(a['maxima'])==6 and all(math.isfinite(v) and v>=0 for v in a['maxima'])
        assert a['maxima'][0]<=1e-5 and a['maxima'][1]<=1e-3 and a['maxima'][4]<=1e-5 and a['maxima'][5]<=1e-3
    identity=json.loads((ROOT/'evidence/native-mtp-stream-matrix/raw/references.json').read_text())['identity']
    held=read('heldout/summary.json');assert held['identity']==identity and held['pass'] and held['quality']==[15,20,23] and held['cases']==60 and held['residents_after_drain']==held['uploaded_bytes']==0
    assert sum(r['quality_pass'] for r in rows)==58
    assert held['compiled_entries']==held['compiled_misses']==30 and held['compiled_calls']>0 and held['trace_compared'] is False
    packed=read('packed/summary.json');assert packed['identity']==identity and packed['pass'] and packed['native_bytes']==packed['native_residents']==0 and len(packed['observations'])==96
    assert {r['batch'] for r in packed['observations']}=={1,2,3}
    for r in packed['observations']:
        assert r['pass'] and r['token']==r['expected']
        for kind in ('logits','state'):
            v=r[kind];assert v['pass'] and math.isfinite(v['nrmse']) and 0<=v['nrmse']<=1e-5 and math.isfinite(v['max_abs']) and 0<=v['max_abs']<=1e-3
    pressure=read('pressure/summary.json');assert pressure['context']==512 and pressure['pass']
    for k in ('branch_isolation','fourth_step_rejected','pending_receipt_cancel','recovery','restart_foreign_epoch_rejected','slot_exhaustion','three_admitted_queue_cancel'):assert pressure[k] is True
    assert pressure['hot_path_exported_bytes']==pressure['hot_path_uploaded_bytes']==0
    for k in ('chunk_single_logit_drift','chunk_single_state_drift'):
        v=pressure[k];assert len(v)==2 and all(math.isfinite(x) and x>=0 for x in v) and v[0]<=1e-5 and v[1]<=1e-3
    http=read('http/summary.json');assert http['pass'] and http['batch_metrics']['packed_calls']>0
    assert (base/'http/ts-client.log').read_text().strip()=='HTTP_TS_PASS tokens=8'
    log=(base/'http/server.log').read_text();assert 'native_bytes=0' in log and 'memory_used: 0' in log and 'active_connections: 0' in log and 'active_producers: 0' in log
    match=re.search(r'COMPILED_CACHE entries=(\d+) misses=(\d+) calls=(\d+)',log)
    assert match and int(match[1])==int(match[2])==60 and int(match[3])>0
    for d in (held,packed,pressure,http):assert d['graph_mode']=='compiled-decode' and d['layer_group']==10 and d['recurrent_kernel']=='fused'
    return {'pass':True,'scope':'retained numerical/lifetime/HTTP observation integrity; no throughput speedup or production qualification','heldout_exact_cases':60,'task_successes':58,'packed_rows':96,'context':512,'http_compiled_entries':60}
if __name__=='__main__':print(json.dumps(verify(sys.argv[1]),indent=2))
