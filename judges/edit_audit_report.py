"""Strict reader for the user-edit audit; never trusts serialized pass flags."""
import json
import math
from pathlib import Path


def validate(report, frozen):
    assert report['identity'] == frozen['identity']
    assert report['execution_revision'] == 'user-edit-audit-v1'
    assert report['attention_kernel'] == 'reference'
    assert report['operator_profile'] in ('off', 'stages')
    assert report['native_residents'] == report['native_bytes'] == 0
    observations = report['observations']
    assert len(observations) == 63
    for row in observations:
        assert row['pass'] is True
        for name in ('logits', 'state'):
            value = row[name]
            assert value['pass'] is True
            assert math.isfinite(value['nrmse']) and 0 <= value['nrmse'] <= 1e-5
            assert math.isfinite(value['max_abs']) and 0 <= value['max_abs'] <= 1e-3
    timings = report['timings']
    assert len(timings) == 4
    for index, row in enumerate(timings):
        assert row['prompt'] == index
        assert len(row['tokens']) == 64
        assert row['tokens'] == frozen['tokens'][index]
        assert row['decode_steps'] == 63
        for key in ('decode_seconds', 'ttft_seconds'):
            assert math.isfinite(row[key]) and row[key] > 0
        assert math.isclose(row['decode_tok_s'], 63 / row['decode_seconds'], rel_tol=1e-12)
        if report['operator_profile'] == 'stages':
            expected = dict(zip(('embedding','input_norm','recurrent_projection','recurrent_prepare','recurrent_update','recurrent_output','attention_projection','attention_reduction','attention_output','residual_norm','moe_routing','moe_up_gate','moe_down','moe_mixture','moe_shared','moe_merge','layer_output','final_projection','state_validation'), (1,40,30,30,30,30,10,10,10,40,40,40,40,40,40,40,40,1,1)))
            assert report['graph_mode'] == 'eager' and report['execution_timing'] == 'off'
            for phase in ('prefill_operators', 'decode_operators'):
                counters = row[phase]
                assert set(counters) == set(expected)
                calls = 63 if phase == 'decode_operators' else counters['embedding']['calls']
                assert calls > 0
                for name, factor in expected.items():
                    assert counters[name]['calls'] == calls * factor
                    assert counters[name]['host_ns'] > 0 and counters[name]['wait_ns'] > 0
        if report['operator_profile'] == 'off':
            assert all(v == 0 for stage in row['decode_operators'].values() for v in stage.values())
    assert report['pass'] is True
    return True


if __name__ == '__main__':
    import sys
    validate(json.loads(Path(sys.argv[1]).read_text()), json.loads(Path('evidence/native-mtp-stream-matrix/raw/references.json').read_text()))
    print('edit-audit report passed')
