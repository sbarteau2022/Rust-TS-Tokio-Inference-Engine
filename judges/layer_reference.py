"""External stock observer for native layer/state captures. No installed source edits."""
import argparse
import json
from pathlib import Path
import sys
import suite_reference as base

STAGES = ['input_norm', 'mixer', 'residual', 'moe', 'output']


def state_layout(position):
    base.require(1 <= position <= 512, 'state position')
    result, start = [], 0
    for layer in range(40):
        shapes = [(1, 2, position, 256)] * 2 if layer % 4 == 3 else [(1, 3, 8192), (1, 32, 128, 128)]
        names = ['key', 'value'] if layer % 4 == 3 else ['conv', 'recurrent']
        for name, shape in zip(names, shapes, strict=True):
            size = 1
            for d in shape:
                size *= d
            result.append(dict(layer=layer, state=name, shape=shape, start=start, end=start + size))
            start += size
    return result


def metrics(a, b):
    import numpy as np
    base.require(a.shape == b.shape and a.size > 0 and np.isfinite(a).all() and np.isfinite(b).all(), 'invalid metric arrays')
    a, b = a.astype(np.float64), b.astype(np.float64)
    delta = a - b
    return dict(nrmse=float(np.linalg.norm(delta) / max(np.linalg.norm(b), 1e-30)), max_abs=float(np.max(np.abs(delta))))


def records(scratch):
    base.require((scratch / 'drain.txt').read_text() == 'leases=0 cases=2\n', 'capture incomplete')
    p = scratch / 'native.jsonl'
    base.require(p.stat().st_size <= 1024 * 1024, 'report cap')
    data = [json.loads(line) for line in p.read_text().splitlines()]
    inventory = {c['id']: c for c in base.inventory() if c['suite'] == 'domain'}
    cursor = 0
    for caseid in ['s10', 'r11']:
        subset = [r for r in data if r['case'] == caseid]
        base.require(bool(subset), 'missing case')
        ids = subset[0]['prompt_ids']
        base.require(0 < len(ids) <= 512, 'prompt bounds')
        for offset in range(0, len(ids), 8):
            base.require(cursor < len(data), 'missing chunk')
            r = data[cursor]
            count = min(8, len(ids) - offset)
            base.require(r['case'] == caseid and r['prompt'] == inventory[caseid]['prompt'] and r['prompt_ids'] == ids,
                         'case/order/prompt mismatch')
            base.require(r['identity'] == base.IDENTITY and r['offset'] == offset and r['count'] == count and r['position'] == offset + count, 'chunk binding')
            base.require(r['trace_values'] == 40 * 5 * count * 2048 and r['state_values'] == state_layout(offset + count)[-1]['end'], 'layout bounds')
            base.require(all(0 <= r[key] <= 1e-6 for key in ['logit_control', 'state_control']), 'native observer perturbation')
            cursor += 1
    base.require(cursor == len(data), 'extra records')
    return data


def load_native(scratch, r, kind):
    import numpy as np
    name = f"{r['case']}-{r['offset']}-{kind}.f32le"
    path = scratch / name
    count = 248320 if kind == 'logits' else r[kind + '_values']
    base.require(path.stat().st_size == count * 4, 'vector bytes')
    raw = path.read_bytes()
    base.require(base.sha(raw) == r[kind + '_sha256'], 'vector hash')
    return np.frombuffer(raw, dtype='<f4')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--stock-scratch', type=Path, required=True)
    parser.add_argument('--view', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--attention-profile', choices=['stock', 'explicit-fp32-control'], default='stock')
    args = parser.parse_args()
    data = records(args.scratch)
    view, sources = base.verify_view(args.view), base.package_sources()
    args.stock_scratch.mkdir()
    args.out.mkdir()
    import os
    os.environ['HF_HUB_OFFLINE'] = '1'
    os.environ['TRANSFORMERS_OFFLINE'] = '1'
    import numpy as np
    import mlx.core as mx
    import mlx.nn as nn
    from mlx_lm import load
    from mlx_lm.models.cache import make_prompt_cache
    model, tokenizer = load(str(args.view), tokenizer_config={'trust_remote_code': False})
    model.set_dtype(mx.float32)
    mx.eval(model.parameters())
    if args.attention_profile == 'explicit-fp32-control':
        from mlx_lm.models import qwen3_next
        from attention_profile import explicit_fp32
        qwen3_next.scaled_dot_product_attention = explicit_fp32
    observed = {}

    class Tap(nn.Module):
        def __init__(self, inner, layer, stage, take_input=False):
            super().__init__()
            self.inner, self.layer, self.stage, self.take_input = inner, layer, stage, take_input

        @property
        def is_linear(self):
            return self.inner.is_linear

        def __call__(self, *a, **kw):
            value = self.inner(*a, **kw)
            observed[(self.layer, self.stage)] = a[0] if self.take_input else value
            return value

    def capture_state(cache, position):
        arrays = []
        layout = state_layout(position)
        for i, c in enumerate(cache):
            for a, spec in zip(c.state, layout[2 * i:2 * i + 2], strict=True):
                base.require(tuple(a.shape) == spec['shape'] and a.dtype == mx.float32, 'stock cache geometry/precision')
                arrays.append(np.array(a, dtype='<f4').reshape(-1))
        return np.concatenate(arrays)

    results = []
    for mode in ['untouched', 'observed']:
        if mode == 'observed':
            for i, layer in enumerate(model.language_model.model.layers):
                layer.input_layernorm = Tap(layer.input_layernorm, i, 0)
                if layer.is_linear:
                    layer.linear_attn = Tap(layer.linear_attn, i, 1)
                else:
                    layer.self_attn = Tap(layer.self_attn, i, 1)
                layer.post_attention_layernorm = Tap(layer.post_attention_layernorm, i, 2, True)
                layer.mlp = Tap(layer.mlp, i, 3)
                model.language_model.model.layers[i] = Tap(layer, i, 4)
        for caseid in ['s10', 'r11']:
            cache = make_prompt_cache(model)
            selected = [r for r in data if r['case'] == caseid]
            ids = tokenizer.apply_chat_template([dict(role='user', content=selected[0]['prompt'])], tokenize=True,
                                                 add_generation_prompt=True, enable_thinking=False)
            base.require(ids == selected[0]['prompt_ids'], 'prompt token mismatch')
            for r in selected:
                observed.clear()
                offset, count = r['offset'], r['count']
                logits = model(mx.array([ids[offset:offset + count]]), cache=cache)[:, -1, :].astype(mx.float32)
                mx.eval(logits, [c.state for c in cache], list(observed.values()))
                values = np.array(logits[0], dtype='<f4')
                state = capture_state(cache, r['position'])
                prefix = args.stock_scratch / f'{caseid}-{offset}'
                if mode == 'untouched':
                    np.save(str(prefix) + '-plain-state.npy', state, allow_pickle=False)
                    np.save(str(prefix) + '-plain-logits.npy', values, allow_pickle=False)
                    continue
                control = dict(logits=metrics(values, np.load(str(prefix) + '-plain-logits.npy', allow_pickle=False)),
                               state=metrics(state, np.load(str(prefix) + '-plain-state.npy', allow_pickle=False)))
                base.require(all(v['nrmse'] <= 1e-6 for v in control.values()), 'stock observer perturbation')
                base.require(len(observed) == 200, 'stage inventory')
                trace = np.stack([np.array(observed[(l, s)], dtype='<f4').reshape(count, 2048)
                                  for l in range(40) for s in range(5)])
                np.save(str(prefix) + '-trace.npy', trace, allow_pickle=False)
                native_trace = load_native(args.scratch, r, 'trace').reshape(200, count, 2048)
                native_state = load_native(args.scratch, r, 'state')
                native_logits = load_native(args.scratch, r, 'logits')
                stages = [dict(layer=l, stage=STAGES[s], **metrics(native_trace[5 * l + s], trace[5 * l + s]))
                          for l in range(40) for s in range(5)]
                states = [dict(layer=v['layer'], state=v['state'], **metrics(native_state[v['start']:v['end']], state[v['start']:v['end']]))
                          for v in state_layout(r['position'])]
                first = next((v for v in stages if v['nrmse'] > 1e-4), None)
                record = dict(case=caseid, offset=offset, count=count, position=r['position'], observer_control=control,
                              first_over_1e_minus_4=first, stages=stages, states=states, logits=metrics(native_logits, values),
                              stock_trace_sha256=base.sha(trace.tobytes()), stock_state_sha256=base.sha(state.tobytes()),
                              stock_logits_sha256=base.sha(values.tobytes()), native=r)
                results.append(record)
                (args.out / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
                print('LAYER-PAIR', caseid, offset, first, 'logits', record['logits'], flush=True)
            del cache
            mx.clear_cache()
    base.require(view == base.verify_view(args.view) and sources == base.package_sources(), 'provenance changed')
    blocked = [m for m in sys.modules if any(p in m.lower() for p in ['dspark', 'dflash', 'rapid_mlx', 'rapidmlx'])]
    base.require(not blocked, 'nonstock imports')
    (args.out / 'provenance.json').write_text(json.dumps(dict(identity=base.IDENTITY, view=view, package_sources=sources,
        executable=sys.executable, forbidden_modules=blocked, attention_profile=args.attention_profile, observer='in-memory delegating modules; installed source untouched'), indent=2) + '\n')


if __name__ == '__main__':
    main()
