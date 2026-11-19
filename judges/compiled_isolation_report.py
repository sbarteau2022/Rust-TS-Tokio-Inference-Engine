"""Independent compiled-state and phase-journal checks; no model execution."""
import json
import math
from pathlib import Path
from http_controls_verify import require


def gate(report, journal, reference):
    require(report['pass'] is True and report['graph_mode'] == 'compiled-decode', 'compiled gate')
    require(report['layer_group'] in [4, 40], 'qualified group')
    require(report['cases'] == reference['cases'], 'eager committed tokens/counters mismatch')
    expected = [('open.begin', 0, 0), ('open.end', 0, 0)]
    for width in [1, 2, 4, 8]:
        expected.extend((phase, width, 0) for phase in ['start.begin', 'start.end', 'start.memory', 'limited.begin', 'limited.end'])
        for iteration in range(4):
            expected.extend((phase, width, iteration) for phase in ['decode.begin', 'decode.end', 'reference.begin', 'reference.end'])
    expected.extend([('drain.begin', 0, 0), ('drain.end', 0, 0)])
    actual = [(r['phase'], r['width'], r.get('iteration', 0)) for r in journal]
    require(actual == expected, 'unfinished/reordered/duplicate phase')
    times = [r['elapsed_ms'] for r in journal if 'elapsed_ms' in r]
    require(all(math.isfinite(t) and t >= 0 for t in times) and times == sorted(times), 'phase clock')
    require(times[-1] < 180000, 'bounded completion')
    memory = [r for r in journal if r['phase'] == 'start.memory']
    require(all(0 < r['active_bytes'] <= r['peak_bytes'] < 32 * 1024**3 for r in memory), 'bounded MLX memory')
    require(all(0 < r['compiled_entries'] <= 1114 for r in memory), 'compiler cache bounds')
    require(max(r['active_bytes'] for r in memory) - min(r['active_bytes'] for r in memory) < 1024**3, 'shape cache duplicates weights')
    return {'pass': True, 'full_state_gate': True, 'widths': [1, 2, 4, 8], 'total_ms': times[-1], 'max_recorded_peak_bytes': max(r['peak_bytes'] for r in memory), 'serving_qualified': False}


def main():
    import sys
    directory = Path(sys.argv[1])
    reference = json.loads(Path('evidence/tonight-kernel/mtp-eager-committed/summary.json').read_text())
    results = {}
    for name in ['shared-constants-group4', 'shared-constants-group40']:
        folder = directory / name
        report = json.loads((folder / 'summary.json').read_text())
        journal = [json.loads(line) for line in (folder / 'progress.jsonl').read_text().splitlines()]
        results[name] = gate(report, journal, reference)
    (directory / 'state-verification.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results))


if __name__ == '__main__':
    main()
