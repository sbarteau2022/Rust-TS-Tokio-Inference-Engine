"""Offline Rust-boundary attribution; no GPU-time or throughput promotion claim."""
import json
import math
from pathlib import Path
from tonight_kernel_report import benchmark
from http_controls_verify import require

FIELDS = ['prefill_ns', 'target_prefill_ns', 'draft_prefill_ns', 'prepare_ns', 'draft_ns', 'commit_ns', 'verify_ns', 'replay_ns', 'refresh_ns']
PHASES = ['draft_ns', 'verify_ns', 'replay_ns', 'refresh_ns']
COUNTERS = ['learned_proposed', 'learned_accepted', 'target_calls', 'draft_calls', 'target_forward_tokens']


def durations(timing):
    require(set(timing) == set(FIELDS), 'timing inventory')
    require(all(type(v) is int and 0 <= v <= 2**64 - 1 for v in timing.values()), 'integer duration')


def validate(control, timed, frozen, candidate=False):
    require(control['phase_timing'] is candidate and timed['phase_timing'] is True, 'diagnostic build labels')
    require(not control.get('capture_wait', False) and not timed.get('capture_wait', False), 'captured run is not a throughput control')
    for key in ['draft_width', 'graph_mode', 'layer_group', 'identity']:
        require(control[key] == timed[key], 'control configuration mismatch')
    if candidate:
        require(control.get('expert_kernel', 'reference') == 'reference' and timed.get('expert_kernel') in ['sorted', 'fused'], 'candidate kernel identity')
    else:
        require(control.get('expert_kernel', 'reference') == timed.get('expert_kernel', 'reference'), 'instrumentation kernel drift')
    base = benchmark(control, frozen)
    measured = benchmark(timed, frozen)
    indexed = {(r['round'], r['prompt'], r['profile']): r for r in control['rows']}
    total = dict.fromkeys(PHASES + ['host_ns', 'envelope_ns', 'decode_ns', 'prefill_ns', 'target_prefill_ns', 'draft_prefill_ns', 'prefill_host_ns', 'emitted', 'accepted', 'proposed', 'target_calls', 'reused', 'replayed'], 0)
    for row in timed['rows']:
        prior = indexed[(row['round'], row['prompt'], row['profile'])]
        require(all(row[k] == prior[k] for k in (['ids', 'tokens'] if candidate else COUNTERS + ['ids', 'tokens'])), 'output/counter drift')
        if not candidate:
            require('mtp_phase_receipts' not in prior and 'mtp_prefill_timing' not in prior, 'instrumented control')
        if row['profile'] == 'target-greedy':
            require(row['mtp_prefill_timing'] is None and row['mtp_phase_receipts'] == [], 'target mislabeled as MTP')
            continue
        p = row['mtp_prefill_timing']
        durations(p)
        require(p['prefill_ns'] > 0 and p['target_prefill_ns'] > 0 and p['draft_prefill_ns'] > 0, 'prefill coverage')
        require(p['target_prefill_ns'] + p['draft_prefill_ns'] <= p['prefill_ns'] <= row['prefill_seconds'] * 1e9, 'prefill envelope')
        require(all(p[k] == 0 for k in ['prepare_ns', 'commit_ns'] + PHASES), 'prefill/transaction overlap')
        receipts = row['mtp_phase_receipts']
        require(bool(receipts) and sum(r['emitted'] for r in receipts) == row['tokens'], 'receipt coverage')
        for key in COUNTERS:
            require(sum(r[key] for r in receipts) == row[key], 'receipt counter reconciliation')
        row_envelope = 0
        for r in receipts:
            require(all(type(r[k]) is int and r[k] >= 0 for k in ['emitted', 'reused_target_tokens', 'replayed_target_tokens'] + COUNTERS), 'integer receipt counters')
            require(1 <= r['emitted'] <= timed['draft_width'] and r['learned_accepted'] <= r['learned_proposed'] < timed['draft_width'], 'receipt proposal bounds')
            require(r['emitted'] == r['reused_target_tokens'] + r['replayed_target_tokens'], 'reuse/replay coverage')
            require((r['reused_target_tokens'] > 0) == (r['learned_accepted'] == r['learned_proposed']), 'accepted-window reuse')
            if r['reused_target_tokens']:
                require(r['emitted'] == r['learned_proposed'] + 1, 'full-window yield')
            require(r['target_forward_tokens'] == r['learned_proposed'] + 1 + r['replayed_target_tokens'], 'forward coverage')
            require(r['target_calls'] in [1, 2] and r['draft_calls'] >= 1, 'call coverage')
            require((r['target_calls'] == 1) == (r['replayed_target_tokens'] == 0), 'replay call coverage')
            t = r['timing']
            durations(t)
            require(all(t[k] == 0 for k in ['prefill_ns', 'target_prefill_ns', 'draft_prefill_ns']), 'transaction/prefill overlap')
            require(t['verify_ns'] > 0 and t['refresh_ns'] > 0, 'await coverage')
            require((t['draft_ns'] == 0) == (r['draft_calls'] == 1), 'proposal timing coverage')
            require((t['replay_ns'] == 0) == (r['target_calls'] == 1), 'replay timing coverage')
            require(t['draft_ns'] <= t['prepare_ns'], 'prepare envelope')
            require(t['verify_ns'] + t['replay_ns'] + t['refresh_ns'] <= t['commit_ns'], 'commit envelope')
            envelope = t['prepare_ns'] + t['commit_ns']
            row_envelope += envelope
            for k in PHASES:
                total[k] += t[k]
            total['host_ns'] += envelope - sum(t[k] for k in PHASES)
            total['emitted'] += r['emitted']
            total['accepted'] += r['learned_accepted']
            total['proposed'] += r['learned_proposed']
            total['target_calls'] += r['target_calls']
            total['reused'] += r['reused_target_tokens']
            total['replayed'] += r['replayed_target_tokens']
        require(row_envelope <= row['decode_seconds'] * 1e9, 'decode envelope')
        total['envelope_ns'] += row_envelope
        total['decode_ns'] += row['decode_seconds'] * 1e9
        total['prefill_ns'] += p['prefill_ns']
        total['target_prefill_ns'] += p['target_prefill_ns']
        total['draft_prefill_ns'] += p['draft_prefill_ns']
        total['prefill_host_ns'] += p['prefill_ns'] - p['target_prefill_ns'] - p['draft_prefill_ns']
    require(total['emitted'] == 256 and total['envelope_ns'] > 0, 'MTP coverage')
    require(total['proposed'] > 0 and total['accepted'] / total['proposed'] >= 0.9, 'learned acceptance gate')
    mtp = f'mtp-width{timed["draft_width"]}-reuse'
    baseline_seconds = sum(r['decode_seconds'] for r in control['rows'] if r['profile'] == mtp)
    timed_seconds = total['decode_ns'] / 1e9
    fractions = {k: total[k] / total['envelope_ns'] for k in PHASES + ['host_ns']}
    require(math.isclose(sum(fractions.values()), 1, rel_tol=1e-12), 'partition')
    return {'pass': True, 'scope': 'successful awaited Rust-boundary calls; queue/wait included, not GPU timestamps; one ordered control/timed pair', 'base': base, 'timed': measured, 'totals': total, 'transaction_fraction': fractions, 'comparison': 'candidate vs control' if candidate else 'timing-feature vs control', 'observed_decode_delta_percent': (timed_seconds / baseline_seconds - 1) * 100, 'committed_tokens_per_target_call': total['emitted'] / total['target_calls'], 'serving_qualified': False, 'throughput_promotion': False}


def main():
    import sys
    directory = Path(sys.argv[1])
    result = validate(json.loads((directory / 'control/summary.json').read_text()), json.loads((directory / 'timed/summary.json').read_text()), json.loads(Path('evidence/native-mtp-stream-matrix/raw/references.json').read_text()))
    (directory / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
