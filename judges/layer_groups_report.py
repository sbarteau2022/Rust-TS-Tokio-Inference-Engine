"""Offline observation/accounting verifier, not an independent recomputation of GPU arrays."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
base=ROOT/(sys.argv[1] if len(sys.argv)>1 else 'evidence/native-layer-groups')
refs=json.loads((ROOT/'evidence/native-mtp-stream-matrix/raw/references.json').read_text())
rows=[]
for path in sorted(base.glob('*/report.json')):
    d=json.loads(path.read_text());assert d['identity']==refs['identity']
    assert d['pass'] is True and d['layer_group'] in (1,4,10,40)
    assert d.get('recurrent_kernel','reference') in ('reference','fused')
    assert d['native_bytes']==d['native_residents']==0
    assert 0<d['mlx_active_bytes']<=d['mlx_peak_bytes']
    assert len(d['timings'])==4 and len(d['observations'])==63
    for prompt in range(4):
        obs=[r for r in d['observations'] if r['prompt']==prompt]
        assert len([r for r in obs if r['decode']])==8
        assert all(a['position']<b['position'] for a,b in zip(obs,obs[1:]))
        for r in obs:
            assert r['pass'] is True and 0<=r['token']<248320
            for kind in ['logits','state']:
                m=r[kind];assert m['pass'] is True
                assert math.isfinite(m['nrmse']) and 0<=m['nrmse']<=1e-5
                assert math.isfinite(m['max_abs']) and 0<=m['max_abs']<=1e-3
                assert isinstance(m['unequal_bits'],int) and m['unequal_bits']>=0
        t=d['timings'][prompt];assert t['prompt']==prompt and t['decode_steps']==63
        assert t['tokens']==refs['tokens'][prompt][:64]
        assert 0<t['ttft_seconds']<t['total_seconds']
        assert 0<t['decode_seconds']<t['total_seconds']
        assert math.isclose(t['decode_tok_s'],63/t['decode_seconds'],rel_tol=1e-12)
    seconds=sum(t['decode_seconds'] for t in d['timings'])
    rows.append({'directory':path.parent.name,'layer_group':d['layer_group'],'recurrent_kernel':d.get('recurrent_kernel','reference'),'native_decode_steps':252,'pooled_native_decode_tok_s':252/seconds,'mean_native_prefill_ms':1000*sum(t['ttft_seconds'] for t in d['timings'])/4,'mlx_peak_gib':d['mlx_peak_bytes']/2**30,'max_logit_abs':max(r['logits']['max_abs'] for r in d['observations']),'max_state_abs':max(r['state']['max_abs'] for r in d['observations']),'all_bitwise_equal':all(r[k]['unequal_bits']==0 for r in d['observations'] for k in ['logits','state'])})
assert rows
out={'pass':True,'scope':'reported numerical gates, exact saved frozen tokens and timing/accounting; not independent GPU-array or HTTP/held-out qualification','runs':rows,'acceptance':None,'speculation':False}
(base/'summary.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
