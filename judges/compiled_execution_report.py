"""Offline integrity checks on native observations; not a GPU numerical oracle."""
import json, math, subprocess, sys, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def verify(path):
    d=json.loads(Path(path).read_text())
    # Reuse strict frozen-token, all-observation numerical and ownership checks.
    with tempfile.TemporaryDirectory() as temp:
        p=Path(temp)/'run';p.mkdir();(p/'report.json').write_text(json.dumps(d))
        subprocess.run([sys.executable,str(ROOT/'judges/layer_groups_report.py'),temp],check=True,stdout=subprocess.DEVNULL)
    assert d['execution_timing'] in ('off','on') and d['graph_mode'] in ('eager','compiled-decode')
    for key in ('compiled_entries','compiled_misses','compiled_calls'):
        assert type(d[key]) is int and d[key]>=0
    if d['graph_mode']=='compiled-decode':
        assert d['compiled_entries']==d['compiled_misses']==30 and d['compiled_calls']>0
    else:assert d['compiled_entries']==d['compiled_misses']==d['compiled_calls']==0
    assert sum(r['compiled_misses_delta'] for r in d['observations'])==d['compiled_misses']
    totals={}
    for t in d['timings']:
        for phase in ('prefill','decode'):
            m=t[phase+'_execution'];assert len(m)==7
            assert all(type(v) is int and v>=0 for v in m.values())
            if d['execution_timing']=='off':assert all(v==0 for v in m.values());continue
            calls=t['decode_steps'] if phase=='decode' else len([r for r in d['observations'] if r['prompt']==t['prompt'] and not r['decode']])
            assert m['calls']==m['completed']==calls
            assert m['completion_groups']==calls*(40//d['layer_group']+1)
            assert 0<m['graph_ns']+m['wait_ns']+m['validation_export_ns']<=m['wall_ns']
            if phase=='decode':
                assert m['wall_ns']<=t['decode_seconds']*1e9
                for k,v in m.items():totals[k]=totals.get(k,0)+v
    return {'pass':True,'graph_mode':d['graph_mode'],'execution_timing':d['execution_timing'],'warm_native_decode_tok_s':252/sum(t['decode_seconds'] for t in d['timings']),'decode_execution':totals,'scope':'four-prompt warm native observation integrity; no HTTP, held-out or speedup claim'}
if __name__=='__main__':print(json.dumps(verify(sys.argv[1]),indent=2))
