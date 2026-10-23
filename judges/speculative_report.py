"""Offline-only accounting judge; not a runtime transport or quality promotion."""
import json
import math
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
IDENTITY = '509cc92543203798f2db08b6fb6cbb38a80942fc0437ae81a3ef9ff27720915b'


def inventory():
    raw = {('raw', line.split('\t')[0]) for line in (ROOT/'fixtures/native-heldout.tsv').read_text().splitlines()}
    domain = {('domain', cols[0]) for line in (ROOT/'fixtures/native-domain-heldout.tsv').read_text().splitlines() if (cols := line.split('\t'))[2] == '-'}
    authored = {('agent-smoke', line.split('\t')[0]) for line in (ROOT/'fixtures/native-speculative-prompts.tsv').read_text().splitlines()}
    assert len(raw) == 16 and len(domain) == 5 and len(authored) == 4
    return raw | domain | authored


def integer(value):
    assert type(value) is int and value >= 0
    return value


def finite(value):
    assert type(value) in (int, float) and math.isfinite(value)
    return value


def judge(summary, rows):
    assert summary['pass'] is True and summary['identity'] == IDENTITY
    assert summary['requests'] == 25 and summary['frozen_free_greedy_requests'] == 21 and summary['authored_agent_smoke_requests'] == 4
    assert summary['boundary_cases'] == 19 and summary['context_edge'] == 512 and summary['cancellation'] is True
    for flag in ['production_qualified', 'trained_mtp', 'scheduler_integrated', 'stochastic_sampling']:
        assert summary[flag] is False
    assert summary['uploaded_bytes'] == summary['residents_after_drain'] == 0
    for key in ['per_position_logit_maxima', 'block_state_maxima']:
        assert len(summary[key]) == 2 and 0 <= finite(summary[key][0]) <= 1e-5 and finite(summary[key][1]) >= 0
    expected = inventory()
    assert len(rows) == len(expected) and {(r['suite'], r['id']) for r in rows} == expected
    groups = {}
    for r in rows:
        group = groups.setdefault(r['suite'], {'requests': 0, 'tokens': 0, 'proposed': 0, 'accepted': 0, 'target_calls': 0, 'baseline_seconds': 0., 'speculative_seconds': 0.})
        authored = r['suite'] == 'agent-smoke'
        assert 0 < len(r['generated']) <= (64 if authored else 32)
        assert r['prompt_ids'] and all(type(t) is int and 0 <= t < 248320 for t in r['prompt_ids'] + r['generated'])
        assert type(r['terminal']) is bool
        assert r['quality_pass'] is None if authored else type(r['quality_pass']) is bool
        assert len(r['state_error']) == 2 and 0 <= finite(r['state_error'][0]) <= 1e-5 and finite(r['state_error'][1]) >= 0
        proposed, accepted, calls, replay = (integer(r[k]) for k in ['proposed', 'accepted', 'target_calls', 'replayed'])
        assert accepted <= proposed and calls > 0 and replay <= len(r['generated'])
        assert calls <= 2*len(r['generated'])
        group['requests'] += 1
        group['tokens'] += len(r['generated'])
        for k in ['proposed', 'accepted', 'target_calls']:
            group[k] += r[k]
        for k in ['baseline_seconds', 'speculative_seconds']:
            assert finite(r[k]) > 0
            group[k] += r[k]
    for source, key in [('retrieval_proposed', 'proposed'), ('retrieval_accepted', 'accepted'), ('retrieval_target_calls', 'target_calls'), ('retrieval_committed_tokens', 'tokens')]:
        assert integer(summary[source]) == sum(g[key] for g in groups.values())
    p, a = summary['retrieval_proposed'], summary['retrieval_accepted']
    assert summary['retrieval_acceptance'] == (a/p if p else None)
    for group in groups.values():
        group['acceptance'] = group['accepted']/group['proposed'] if group['proposed'] else None
        group['baseline_decode_tok_s'] = group['tokens']/group['baseline_seconds']
        group['speculative_decode_tok_s'] = group['tokens']/group['speculative_seconds']
        group['committed_per_target_call'] = group['tokens']/group['target_calls']
    return {'pass': True, 'groups': groups, 'measurement_scope': 'single-lane offline decode-only; fixed baseline-then-variant order; prefill/load excluded; no confidence interval', 'production_qualified': False}


if __name__ == '__main__':
    assert len(sys.argv) == 2, 'speculative_report.py RUN_DIRECTORY'
    directory = Path(sys.argv[1])
    print(json.dumps(judge(json.loads((directory/'summary.json').read_text()), [json.loads(line) for line in (directory/'requests.jsonl').read_text().splitlines()]), indent=2, allow_nan=False))
