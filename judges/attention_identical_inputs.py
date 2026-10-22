"""External same-operand attention experiment on the two frozen first chunks."""
import argparse
import json
from pathlib import Path
import suite_reference as base
import layer_reference as layer
from attention_profile import explicit_fp32, fp64_oracle


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scratch', type=Path, required=True)
    parser.add_argument('--view', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    rows = layer.records(args.scratch)
    view, sources = base.verify_view(args.view), base.package_sources()
    args.out.mkdir()
    import numpy as np
    import mlx.core as mx
    from mlx_lm import load
    from mlx_lm.models.cache import KVCache
    model, tokenizer = load(str(args.view), tokenizer_config={'trust_remote_code': False})
    model.set_dtype(mx.float32)
    mx.eval(model.parameters())
    attention = model.language_model.model.layers[3].self_attn
    records = []
    for caseid in ['s10', 'r11']:
        r = next(r for r in rows if r['case'] == caseid and r['offset'] == 0)
        count = r['count']
        trace = layer.load_native(args.scratch, r, 'trace').reshape(40, 5, count, 2048)
        x = mx.array(trace[3, 0][None, :, :])
        actual_stock = attention(x, mask='causal', cache=KVCache())
        q, gate = mx.split(attention.q_proj(x).reshape(1, count, 16, 512), 2, axis=-1)
        q = attention.rope(attention.q_norm(q).transpose(0, 2, 1, 3), offset=0)
        k = attention.rope(attention.k_norm(attention.k_proj(x).reshape(1, count, 2, 256)).transpose(0, 2, 1, 3), offset=0)
        v = attention.v_proj(x).reshape(1, count, 2, 256).transpose(0, 2, 1, 3)
        fast = mx.fast.scaled_dot_product_attention(q, k, v, scale=attention.scale, mask='causal')
        explicit = explicit_fp32(q, k, v, cache=None, scale=attention.scale, mask='causal')
        mx.eval(actual_stock, q, k, v, gate, fast, explicit)
        oracle = fp64_oracle(*[np.array(a) for a in [q, k, v]], attention.scale)
        def project(y):
            out = attention.o_proj(y.transpose(0, 2, 1, 3).reshape(1, count, 4096) * mx.sigmoid(gate.reshape(1, count, 4096)))
            mx.eval(out)
            return np.array(out)
        fast_output = project(fast)
        explicit_output = project(explicit)
        oracle_output = project(mx.array(oracle.astype(np.float32)))
        native_output = trace[3, 1][None, :, :]
        state = layer.load_native(args.scratch, r, 'state')
        kv_control = {}
        for spec, value in zip(layer.state_layout(count)[6:8], [k, v], strict=True):
            kv_control[spec['state']] = layer.metrics(state[spec['start']:spec['end']].reshape(spec['shape']), np.array(value))
        result = dict(case=caseid, layer=3, offset=0, count=count,
            attention_fast_vs_fp64=layer.metrics(np.array(fast), oracle),
            attention_explicit_vs_fp64=layer.metrics(np.array(explicit), oracle),
            native_vs_fast_output=layer.metrics(native_output, fast_output),
            native_vs_explicit_output=layer.metrics(native_output, explicit_output),
            native_vs_oracle_projected=layer.metrics(native_output, oracle_output),
            original_stock_vs_recomposed_fast=layer.metrics(np.array(actual_stock), fast_output),
            native_vs_identical_input_stock_kv=kv_control,
            inputs_sha256={name:base.sha(np.array(a).tobytes()) for name,a in [('x',x),('q',q),('k',k),('v',v)]})
        records.append(result)
        print(json.dumps(result), flush=True)
    base.require(sources == base.package_sources() and view == base.verify_view(args.view), 'provenance changed')
    (args.out/'results.json').write_text(json.dumps(records,indent=2)+'\n')
    (args.out/'provenance.json').write_text(json.dumps(dict(identity=base.IDENTITY, package_sources=sources, view=view,
        scope='same native input; original stock weights/attention; FP64 CPU NumPy oracle'),indent=2)+'\n')


if __name__ == '__main__':
    main()
