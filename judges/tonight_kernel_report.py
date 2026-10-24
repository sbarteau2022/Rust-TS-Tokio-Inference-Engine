"""Offline independent accounting; never invokes a model or changes qualification gates."""
import json
import math
import re
from pathlib import Path
from edit_audit_report import validate
from http_controls_verify import arithmetic, archive, drain, require

ROOT = Path(__file__).resolve().parents[1]
FROZEN = ROOT / 'evidence/native-mtp-stream-matrix/raw/references.json'


def benchmark(report, frozen):
    require(report['identity'] == frozen['identity'], 'model identity')
    require(report['lanes'] == 1 and not report['http_streaming'] and not report['continuous_batching'], 'microbench scope')
    width = report.get('draft_width', 4)
    require(type(width) is int and 1 <= width <= 8, 'draft width')
    names = [line.split('\t')[0] for line in (ROOT / 'fixtures/native-speculative-prompts.tsv').read_text().splitlines()]
    rows = report['rows']
    require(len(rows) == 16, 'round coverage')
    groups = {}
    seen = set()
    for row in rows:
        key = row['round'], row['prompt'], row['profile']
        require(key not in seen and row['round'] in [0, 1], 'duplicate/invalid round')
        seen.add(key)
        require(row['prompt'] in names and row['profile'] in ['target-greedy', f'mtp-width{width}-reuse'], 'profile/prompt')
        require(row['ids'] == frozen['tokens'][names.index(row['prompt'])][:32] and row['tokens'] == 32, 'frozen token mismatch')
        for key in ['decode_seconds', 'prefill_seconds', 'first_receipt_ms']:
            require(math.isfinite(row[key]) and row[key] > 0, 'finite timing')
        require(math.isclose(row['decode_tokens_per_second'], 32 / row['decode_seconds'], rel_tol=1e-12), 'decode arithmetic')
        for key in ['learned_accepted', 'learned_proposed', 'target_calls', 'draft_calls', 'target_forward_tokens']:
            require(type(row[key]) is int and row[key] >= 0, 'integer counter')
        require(row['target_calls'] > 0 and row['learned_accepted'] <= row['learned_proposed'], 'acceptance accounting')
        if row['profile'] == 'target-greedy':
            require(row['learned_accepted'] == row['learned_proposed'] == row['draft_calls'] == 0 and row['target_calls'] == 32, 'target counters')
        groups.setdefault(row['profile'], []).append(row)
    require(len(seen) == 16 and all(len(g) == 8 for g in groups.values()), 'profile coverage')
    return {p: {'tokens': sum(r['tokens'] for r in g), 'decode_tok_s': sum(r['tokens'] for r in g) / sum(r['decode_seconds'] for r in g), 'accepted': sum(r['learned_accepted'] for r in g), 'proposed': sum(r['learned_proposed'] for r in g), 'committed_per_target_call': sum(r['tokens'] for r in g) / sum(r['target_calls'] for r in g)} for p, g in groups.items()}


def http(directory, frozen):
    summary = json.loads((directory / 'summary.json').read_text())
    options = summary['options']
    require(options['profiles'] == ['mtp'] and options['tokens'] in [1000, 1500, 2000], 'HTTP scope')
    require(len(summary['cells']) == 2 and [c['lanes'] for c in summary['cells']] == [1, 3], 'lane coverage')
    require(summary['total_closed_tokens'] == 2 * options['tokens'], 'total token budget')
    base = directory / 'mtp'
    rows = []
    with (directory / 'archive-verification.log').open('w') as log:
        for cell in summary['cells']:
            require(cell['tokens'] == options['tokens'], 'cell budget')
            arithmetic(cell, cell['requests'])
            rows.extend(cell['requests'])
        loads = summary['open_loop']
        if options['arrival'] == 'closed+open':
            require(len(loads) == 1, 'open-loop coverage')
            load = loads[0]
            require(load['rejected_503'] > 0 and load['failed_requests'] == 0, 'pressure/rejection')
            successes = [a['row'] for a in load['attempts'] if a['status'] == 'success']
            require(len(successes) == load['successful_requests'], 'open success count')
            arithmetic(load, successes)
            rows.extend(successes)
        else:
            require(not loads, 'unexpected open traffic')
        rows.append(json.loads((base / 'warmup.json').read_text()))
        seen = set()
        for row in rows:
            require(row['ordinal'] not in seen, 'duplicate request')
            seen.add(row['ordinal'])
            profile = row['metadata']['profile']
            for text in [f'width={options["draftWidth"]} ', f'layer_group={options["layerGroup"]} ', f'recurrent_kernel={options["recurrentKernel"]} ', 'expert_kernel=reference ', 'sampling=host ', f'graph_mode={options["graphMode"]} ', f'execution_timing={options["executionTiming"]} ', 'physical_tensor_batching=false']:
                require(text in profile, 'metadata tuning mismatch')
            archive(base / f'request-{row["ordinal"]}.amst', row, frozen, log)
    server_log = (base / 'server.log').read_text()
    drained = drain(server_log, len(rows), sum(l['rejected_503'] for l in loads), 'mtp')
    match = re.search(r'DRAFT_LEDGER proposed=(\d+) accepted=(\d+) known_seed=(\d+) committed=(\d+) target_calls=(\d+) failures=(\d+) unfinished=(\d+)', server_log)
    require(match is not None, 'missing ledger')
    proposed, accepted, seed, committed, calls, failures, unfinished = map(int, match.groups())
    require(proposed >= accepted and failures == unfinished == 0 and calls > 0 and committed > 0, 'draft accounting')
    receipt = json.loads((base / 'acceptance.json').read_text())
    require([receipt[k] for k in ['proposed', 'accepted', 'known_seed', 'committed', 'target_calls', 'failures', 'unfinished']] == [proposed, accepted, seed, committed, calls, failures, unfinished], 'ledger artifact mismatch')
    require(math.isclose(receipt['fraction'], accepted / proposed, rel_tol=1e-12), 'acceptance arithmetic')
    baseline = re.search(r'BASELINE_LEDGER committed=(\d+) cost_ns=(\d+)', server_log)
    if baseline:
        baseline_committed, baseline_cost = map(int, baseline.groups())
        require(receipt['baseline_committed'] == baseline_committed and receipt['baseline_cost_ns'] == baseline_cost, 'baseline artifact mismatch')
        require(committed + baseline_committed == sum(len(r['tokens']) for r in rows), 'complete commit/delivery accounting')
    require(summary['pass'] is True, 'failed HTTP run')
    return {'baseline_coverage_checked': baseline is not None, 'closed_tokens': summary['total_closed_tokens'], 'archive_requests': len(rows), 'learned_acceptance': accepted / proposed, 'committed_per_target_call': committed / calls, 'drain': drained}


def component(report):
    require(report['pass'] and report['context'] == 512 and report['logical_bytes_at_shutdown'] == 0, 'component context/drain')
    require(report['branch_bitwise'] and report['combined_budget'] and report['cancellation_drained'], 'component pressure')
    require([r['count'] for r in report['cases']] == list(range(1, 9)), 'component shape coverage')
    for row in report['cases']:
        for key in ['logit_relative_l2', 'kv_relative_l2', 'hidden_relative_l2']:
            require(math.isfinite(row[key]) and 0 <= row[key] <= 1e-5, 'component numerical tolerance')
    return {'counts': list(range(1, 9)), 'context': 512, 'full_kv_hidden_logits': True, 'drained': True}


def main():
    import sys
    directory = Path(sys.argv[1])
    frozen = json.loads(FROZEN.read_text())
    result = {'pass': True, 'scope': 'short-context explicit controls; not production qualification', 'kernels': {}, 'mtp': {}, 'http': {}}
    for name in ['current', 'sorted', 'fused']:
        report = json.loads((directory / name / 'report.json').read_text())
        validate(report, frozen)
        result['kernels'][name] = {'strict_comparisons': 63, 'frozen_tokens': 256}
    for name in ['mtp-postnorm-fixed-accounting', 'mtp-prenorm', 'mtp-width8']:
        result['mtp'][name] = benchmark(json.loads((directory / name / 'summary.json').read_text()), frozen)
    for name in ['http-mtp-width8', 'http-mtp-width4', 'http-mtp-width6', 'http-rowwise-width4', 'http-rowwise-width6']:
        if (directory / name / 'summary.json').is_file():
            result['http'][name] = http(directory / name, frozen)
    result['component'] = component(json.loads((directory / 'mtp-dense-rowwise/summary.json').read_text()))
    result['unqualified_experiments'] = [json.loads((directory / name / 'FAILED.json').read_text()) for name in ['http-mtp-width6', 'mtp-compiled-group4']]
    (directory / 'independent-verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    main()
