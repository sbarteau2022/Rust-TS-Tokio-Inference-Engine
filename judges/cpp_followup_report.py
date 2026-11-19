"""Independent frozen-output, policy and transaction-envelope follow-up accounting."""
import json
import math
from pathlib import Path
from tonight_kernel_report import ROOT, FROZEN, benchmark
from http_controls_verify import require

PRESETS = {'noalloc-multi': (4, 1.25), 'noalloc-single': (4, 1.25),
           'phase': (4, 1.25), 'w8-m250': (8, 2.5),
           'w8-m250-repeat': (8, 2.5), 'width4-repeat': (4, 1.25)}
PHASES = {'proposal', 'verify', 'decision', 'commit', 'refresh'}


def summarize(reports, frozen):
    require(set(reports) == set(PRESETS), 'missing control/repeat')
    result, phases = {}, dict.fromkeys(PHASES, 0.0)
    for name, (width, margin) in PRESETS.items():
        report = reports[name]
        require(report.get('pass') is True and report.get('output_congruence') is True,
                'unsuccessful report')
        for field in ['dspark_execution', 'turboquant_execution', 'production_qualified',
                      'rust_runtime']:
            require(report.get(field) is False, 'scope overclaim')
        require(report.get('logical_bytes_at_shutdown') == 0, 'resident state')
        # Earlier controls predate explicit margin metadata; their width is still checked.
        require(report.get('draft_width') == width and
                report.get('draft_margin', 1.25) == margin, 'policy mismatch')
        result[name] = benchmark(report, frozen)
        if name == 'phase':
            for row in report['rows']:
                measured = row.get('transaction_phase_seconds', {})
                require(set(measured) == PHASES, 'phase coverage')
                require(all(type(v) in [float, int] and math.isfinite(v) and v >= 0
                            for v in measured.values()), 'invalid phase time')
                require(sum(measured.values()) <= row['decode_seconds'] * (1 + 1e-9),
                        'phase exceeds decode envelope')
                if row['profile'] == 'target-greedy':
                    require(sum(measured.values()) == 0, 'target counted as MTP')
                else:
                    require(sum(measured.values()) > 0, 'empty MTP envelope')
                    for key, value in measured.items():
                        phases[key] += value
    acceptance = {}
    for name in ['w8-m250', 'w8-m250-repeat']:
        row = result[name]['mtp-width8-reuse']
        require(row['proposed'] > 0, 'missing proposals')
        acceptance[name] = row['accepted'] * 172 >= row['proposed'] * 166
    total = sum(phases.values())
    require(total > 0, 'missing phase accounting')
    return {'controls': result, 'transaction_envelope_percent':
            {key: value / total * 100 for key, value in sorted(phases.items())},
            'width8_acceptance_floor': acceptance, 'production_promoted': False,
            'combined_125_gate_pass': False,
            'scope': 'fixture controls; host envelopes, not GPU timestamps or HTTP'}


if __name__ == '__main__':
    evidence = ROOT / 'evidence/decode-arch-kernel-proof'
    reports = {name: json.loads((evidence / (name + '.json')).read_text())
               for name in PRESETS}
    print(json.dumps(summarize(reports, json.loads(FROZEN.read_text())), indent=2))
