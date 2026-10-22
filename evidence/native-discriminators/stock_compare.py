"""User-requested stock mlx-lm reference only; never imported by the server."""
import os,sys,time,json,hashlib,inspect,importlib.metadata
from pathlib import Path
os.environ['HF_HUB_OFFLINE']='1';os.environ['TRANSFORMERS_OFFLINE']='1'
import mlx.core as mx
import mlx_lm
from mlx_lm import load
from mlx_lm.models.cache import make_prompt_cache,QuantizedKVCache
from mlx.utils import tree_flatten
root=Path('/Users/stewartbarteau/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine')
out=root/'evidence/native-discriminators'
tag=os.environ.get('REFERENCE_TAG','stock')
cases=[]
for filename,caseid in [('native-domain-heldout.tsv','r10'),('native-heldout.tsv','rust-borrow')]:
 row=next(s.split('\t') for s in (root/'fixtures'/filename).read_text().splitlines() if s.startswith(caseid+'\t'))
 cases.append((caseid,row[3] if caseid=='r10' else row[1]))
if os.environ.get('REFERENCE_PROBES')=='1':
 cases=[tuple(line.split('\t')) for line in (root/'fixtures/native-interpretation-probes.tsv').read_text().splitlines()]
started=time.time();model,tokenizer=load(os.environ.get('REFERENCE_MODEL','/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined'),tokenizer_config={'trust_remote_code':False})
mx.eval(model.parameters())
records=[]
for profile in ['checkpoint-dtypes','f32-parameters']:
 if profile=='f32-parameters':model.set_dtype(mx.float32);mx.eval(model.parameters())
 for caseid,prompt in cases:
  rendered=tokenizer.apply_chat_template([{'role':'user','content':prompt}],tokenize=False,add_generation_prompt=True,enable_thinking=False)
  manual='<|im_start|>user\n'+prompt.strip()+'<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'
  assert rendered==manual,(caseid,repr(rendered),repr(manual))
  ids=tokenizer.encode(rendered,add_special_tokens=False)
  cache=make_prompt_cache(model);assert not any(isinstance(c,QuantizedKVCache) for c in cache)
  for offset in range(0,len(ids),8):
   logits=model(mx.array([ids[offset:offset+8]]),cache=cache)[:,-1,:].astype(mx.float32);mx.eval(logits,[c.state for c in cache])
  lp=logits-mx.logsumexp(logits,axis=-1,keepdims=True);mx.eval(lp)
  scores={}
  for text in ['yes','no','Yes','No']:
   toks=tokenizer.encode(text,add_special_tokens=False);assert len(toks)==1,(text,toks)
   scores[text]={'token':toks[0],'logit':float(logits[0,toks[0]].item()),'logprob':float(lp[0,toks[0]].item())}
  kv=[{'class':type(c).__name__,'dtypes':[str(v.dtype) for _,v in tree_flatten(c.state) if isinstance(v,mx.array)]} for c in cache]
  generated=[]
  for _ in range(32):
   tok=int(mx.argmax(logits,axis=-1).item());generated.append(tok)
   if tok in tokenizer.eos_token_ids:break
   logits=model(mx.array([[tok]]),cache=cache)[:,-1,:].astype(mx.float32);mx.eval(logits,[c.state for c in cache])
  result={'case':caseid,'profile':profile,'prompt':prompt,'rendered':rendered,'prompt_ids':ids,'scores':scores,'no_minus_yes':scores['no']['logprob']-scores['yes']['logprob'],'capital_No_minus_no':scores['No']['logprob']-scores['no']['logprob'],'generated':generated,'text':tokenizer.decode(generated,skip_special_tokens=True),'raw_text':tokenizer.decode(generated),'cache':kv,'speculation':False,'prefix_reuse':False,'kv_quantization':False}
  records.append(result);print('STOCK',caseid,profile,result['text'],result['no_minus_yes'],flush=True)
  (out/(tag+'-results.json')).write_text(json.dumps(records,indent=2)+'\n')
  del cache,logits;mx.clear_cache()
blocked=[m for m in sys.modules if any(part in m.lower() for part in ['dspark','dflash','rapid_mlx','rapidmlx'])]
assert not blocked,blocked
sources={}
for name,module in list(sys.modules.items()):
 if name.startswith('mlx_lm') and getattr(module,'__file__',None):
  p=Path(module.__file__)
  if p.suffix=='.py':sources[str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/(tag+'-provenance.json')).write_text(json.dumps({'mlx_lm':importlib.metadata.version('mlx-lm'),'mlx':importlib.metadata.version('mlx'),'python':sys.version,'executable':sys.executable,'forbidden_modules':blocked,'source_sha256':sources,'seconds':time.time()-started},indent=2)+'\n')
