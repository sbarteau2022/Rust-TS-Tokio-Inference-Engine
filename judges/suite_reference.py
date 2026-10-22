"""External stock mlx-lm experiment. This module is never used by the Rust server.
Pure helpers import without MLX; actual inference requires the user's stock env.
"""
import argparse
import base64
import hashlib
import importlib.metadata
import json
import math
from pathlib import Path
import struct
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
MODEL = Path.home() / 'models/fused/Qwen3.6-MoE-Combined'
IDENTITY = '509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def inventory():
    def rows(name, count, width):
        result = [line.split('\t') for line in (ROOT / 'fixtures' / name).read_text().splitlines()]
        require(len(result) == count and all(len(r) == width and all(r) for r in result), 'fixture inventory')
        require(len({r[0] for r in result}) == count, 'duplicate fixture')
        return result
    raw = rows('native-heldout.tsv', 16, 3)
    extra = rows('native-choice-heldout.tsv', 4, 3)
    cases = []
    for suite in ['raw', 'contract']:
        for caseid, prompt, expected in raw + (extra if suite == 'contract' else []):
            choices = ['yes', 'no'] if suite == 'contract' and (caseid == 'rust-borrow' or caseid.startswith('choice-')) else None
            cases.append(dict(suite=suite, id=caseid, prompt=prompt, choices=choices, expected=expected))
    for caseid, domain, choices, prompt, expected in rows('native-domain-heldout.tsv', 24, 5):
        require(domain in ['rust', 'security'], 'domain')
        cases.append(dict(suite='domain', id=caseid, prompt=prompt, choices=None if choices == '-' else choices.split(','), expected=expected))
    require(len(cases) == 60, 'suite coverage')
    return cases


def contract(paths, vocab, eos, budget=32):
    if paths is None:
        return None
    require(bool(eos) and all(0 <= t < vocab for t in eos), 'EOS bounds')
    require(1 <= len(paths) <= 16, 'choice count')
    paths = sorted(tuple(p) for p in paths)
    require(len(set(paths)) == len(paths), 'duplicate paths')
    require(all(0 < len(p) <= 64 and len(p) < budget and all(0 <= t < vocab and t not in eos for t in p) for p in paths), 'path bounds')
    words = [vocab, len(set(eos)), *sorted(set(eos)), len(paths)]
    for p in paths:
        words.extend([len(p), *p])
    return sha(b'canonical-token-choices-v1' + struct.pack('<' + 'I' * len(words), *words))


def allowed(paths, prefix, eos):
    if paths is None:
        return None
    result = set()
    for path in paths:
        if list(path[:len(prefix)]) == list(prefix):
            result.update(eos if len(path) == len(prefix) else [path[len(prefix)]])
    return sorted(result)


def select(scores, legal=None):
    require(len(scores) > 0 and all(math.isfinite(float(s)) for s in scores), 'nonfinite/empty logits')
    ids = range(len(scores)) if legal is None else legal
    require(legal is None or bool(legal), 'empty legal set')
    require(legal is None or all(0 <= t < len(scores) for t in legal), 'legal bounds')
    # Rust total_cmp distinguishes +0 from -0; remaining finite ties use lowest ID.
    return max(ids, key=lambda t: (float(scores[t]), math.copysign(1., float(scores[t])) if scores[t] == 0 else 0, -t))


def native_records(scratch):
    path = scratch / 'native.jsonl'
    require(path.stat().st_size <= 8 * 1024 * 1024, 'native report cap')
    records = [json.loads(line) for line in path.read_text().splitlines()]
    cases = inventory()
    require(len(records) == len(cases), 'native record count')
    require((scratch / 'drain.txt').read_text() == 'leases=0 cases=60\n', 'native incomplete drain')
    for record, case in zip(records, cases, strict=True):
        require(record['case'] == case and record['identity'] == IDENTITY, 'native case/model binding')
        require(record['profile'] == 'native-f32', 'native profile')
        require(record['first_logits_file'] == f"{case['suite']}-{case['id']}.f32le", 'vector path')
        require(record['finish'] in ['eos', 'length'], 'finish reason')
        require(record['quality_pass'] == (record['finish'] == 'eos' and record['text'].strip() == case['expected']), 'native quality accounting')
        require(0 < len(record['generated']) <= 32, 'generation bounds')
    return records


def verify_view(view):
    baseline = json.loads((ROOT / 'evidence/native-discriminators/indexed-stock-view.json').read_text())
    expected = {f['name']: f for f in baseline['files']}
    require({p.name for p in view.iterdir()} == set(expected), 'index-only view inventory')
    result = []
    for name, old in sorted(expected.items()):
        path = view / name
        require(path.is_symlink() and path.resolve() == MODEL / name, 'checkpoint symlink target')
        stat = path.stat()
        current = dict(name=name, target=str(path.resolve()), device=stat.st_dev, inode=stat.st_ino,
                       size=stat.st_size, mtime_ns=stat.st_mtime_ns)
        require(current == old, 'checkpoint artifact changed since ingestion audit')
        result.append(current)
    return result


def package_sources():
    dist = importlib.metadata.distribution('mlx-lm')
    sources = {}
    for item in dist.files:
        if str(item).startswith('mlx_lm/') and str(item).endswith('.py'):
            data = Path(dist.locate_file(item)).read_bytes()
            require(item.hash is not None and item.hash.mode == 'sha256', 'package RECORD hash missing')
            require(base64.urlsafe_b64encode(hashlib.sha256(data).digest()).decode().rstrip('=') == item.hash.value, 'stock source modified')
            sources[str(item)] = sha(data)
    require(len(sources) >= 100, 'stock package inventory')
    return sources


def compare_vectors(left, right):
    import numpy as np
    require(left.shape == right.shape == (248320,) and np.isfinite(left).all() and np.isfinite(right).all(), 'vector bounds/finiteness')
    a, b = left.astype(np.float64), right.astype(np.float64)
    delta = a - b
    centered = (a - a.mean()) - (b - b.mean())
    return dict(max_abs=float(np.max(np.abs(delta))),
                nrmse=float(np.linalg.norm(delta) / max(np.linalg.norm(b), 1e-30)),
                centered_nrmse=float(np.linalg.norm(centered) / max(np.linalg.norm(b - b.mean()), 1e-30)),
                mean_offset=float(delta.mean()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--view', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--attention-profile', choices=['stock', 'explicit-fp32-control'], default='stock')
    args = parser.parse_args()
    records = native_records(args.scratch)
    view = verify_view(args.view)
    sources = package_sources()
    args.out.mkdir()  # New evidence only; do not overwrite a previous run.
    import os
    os.environ['HF_HUB_OFFLINE'] = '1'
    os.environ['TRANSFORMERS_OFFLINE'] = '1'
    import numpy as np
    import mlx.core as mx
    from mlx.utils import tree_flatten
    from mlx_lm import load
    from mlx_lm.models.cache import make_prompt_cache, QuantizedKVCache
    started = time.time()
    model, tokenizer = load(str(args.view), tokenizer_config={'trust_remote_code': False})
    model.set_dtype(mx.float32)
    mx.eval(model.parameters())
    if args.attention_profile == 'explicit-fp32-control':
        from mlx_lm.models import qwen3_next
        from attention_profile import explicit_fp32
        qwen3_next.scaled_dot_product_attention = explicit_fp32
    # set_dtype's floating predicate must preserve packed weights.
    require(any(v.dtype == mx.uint32 for _, v in tree_flatten(model.parameters())), 'packed weights absent')
    results = []
    for native in records:
        case = native['case']
        prompt = tokenizer.apply_chat_template([dict(role='user', content=case['prompt'])], tokenize=False,
                                               add_generation_prompt=True, enable_thinking=False)
        manual = '<|im_start|>user\n' + case['prompt'].strip() + '<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n'
        require(prompt == manual, 'chat template mismatch')
        ids = tokenizer.encode(prompt, add_special_tokens=False)
        require(ids == native['prompt_ids'], 'prompt token mismatch')
        paths = None if case['choices'] is None else [tokenizer.encode(c, add_special_tokens=False) for c in case['choices']]
        if paths is not None:
            require(all(tokenizer.decode(p, skip_special_tokens=True) == c for p, c in zip(paths, case['choices'], strict=True)), 'choice roundtrip')
        eos = sorted(tokenizer.eos_token_ids)
        require(eos == sorted(native['eos']) and paths == native['choice_paths'], 'tokenizer contract mismatch')
        fingerprint = contract(paths, 248320, eos)
        require(fingerprint == native['contract'], 'choice fingerprint mismatch')
        cache = make_prompt_cache(model)
        require(not any(isinstance(c, QuantizedKVCache) for c in cache), 'quantized cache')
        for offset in range(0, len(ids), 8):
            logits = model(mx.array([ids[offset:offset + 8]]), cache=cache)[:, -1, :].astype(mx.float32)
            mx.eval(logits, [c.state for c in cache])
        first = np.array(logits[0], dtype='<f4')
        data = (args.scratch / native['first_logits_file']).read_bytes()
        require(len(data) == 248320 * 4 and sha(data) == native['first_logits_sha256'], 'native vector corrupted')
        vector = compare_vectors(np.frombuffer(data, dtype='<f4'), first)
        cache_types = [dict(kind=type(c).__name__, dtypes=[str(v.dtype) for _, v in tree_flatten(c.state) if isinstance(v, mx.array)]) for c in cache]
        require(all(t == 'mlx.core.float32' for c in cache_types for t in c['dtypes']), 'cache precision mismatch')
        generated, finish = [], 'length'
        for _ in range(32):
            scores = np.array(logits[0], dtype=np.float32)
            token = select(scores, allowed(paths, generated, eos))
            generated.append(token)
            if token in eos:
                finish = 'eos'
                break
            if len(generated) < 32:
                logits = model(mx.array([[token]]), cache=cache)[:, -1, :].astype(mx.float32)
                mx.eval(logits, [c.state for c in cache])
        text = tokenizer.decode(generated, skip_special_tokens=True)
        ranked = sorted(range(len(first)), key=lambda i: (-float(first[i]), i))[:16]
        candidate = {}
        for word in ['yes', 'no', 'Yes', 'No', 'A', 'B']:
            tokens = tokenizer.encode(word, add_special_tokens=False)
            if len(tokens) == 1:
                candidate[word] = dict(token=tokens[0], native=float(np.frombuffer(data, dtype='<f4')[tokens[0]]), stock=float(first[tokens[0]]))
        result = dict(case=case, profile='stock-f32-parameters' if args.attention_profile == 'stock' else 'precision-aligned-f32-control', generated=generated, text=text, finish=finish,
                      quality_pass=finish == 'eos' and text.strip() == case['expected'],
                      exact_match=generated == native['generated'] and text == native['text'] and finish == native['finish'],
                      native_generated=native['generated'], native_text=native['text'], native_finish=native['finish'],
                      native_quality_pass=native['quality_pass'], contract=fingerprint, prompt_ids=ids,
                      native_first_sha256=sha(data), stock_first_sha256=sha(first.tobytes()),
                      first_vector_comparison=vector, top16=[(i, float(first[i])) for i in ranked],
                      candidates=candidate, cache=cache_types)
        results.append(result)
        (args.out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
        print('PAIR', case['suite'], case['id'], 'exact=' + str(result['exact_match']), repr(text), flush=True)
        del cache, logits
        mx.clear_cache()
    blocked = [m for m in sys.modules if any(p in m.lower() for p in ['dspark', 'dflash', 'rapid_mlx', 'rapidmlx'])]
    require(not blocked, 'nonstock imports')
    require(sources == package_sources() and view == verify_view(args.view), 'provenance changed during run')
    summary = {}
    for suite in ['raw', 'contract', 'domain']:
        selected = [r for r in results if r['case']['suite'] == suite]
        summary[suite] = dict(cases=len(selected), exact=sum(r['exact_match'] for r in selected),
                              stock_quality=sum(r['quality_pass'] for r in selected), native_quality=sum(r['native_quality_pass'] for r in selected))
    (args.out / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    provenance = dict(model=IDENTITY, attention_profile=args.attention_profile, executable=sys.executable, python=sys.version,
                      mlx=importlib.metadata.version('mlx'), mlx_lm=importlib.metadata.version('mlx-lm'),
                      package_sources=sources, view=view, forbidden_modules=blocked, seconds=time.time() - started,
                      profile='F32 floating parameters; packed integer weights unchanged',
                      scope='60 fresh-state trajectories; full first-answer vectors; no all-layer equivalence claim')
    (args.out / 'provenance.json').write_text(json.dumps(provenance, indent=2) + '\n')
    print(json.dumps(summary), flush=True)
    if not all(r['exact_match'] for r in results):
        raise SystemExit(1)  # Preserve mismatches as evidence; never convert to a pass.


if __name__ == '__main__':
    main()
