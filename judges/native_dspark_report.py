"""Independent accounting of native DSpark integration diagnostic evidence."""
import json
import math
from pathlib import Path
from http_controls_verify import require
from tonight_kernel_report import ROOT, FROZEN


def validate(report, frozen):
    require(report['pass'] is True and report['dspark_execution'] is True, 'execution')
    require(report['identity'] == frozen['identity'], 'target identity')
    require(report['turboquant_execution'] is False and
            report['production_qualified'] is False and
            report['http_streaming'] is False, 'scope')
    require(type(report['combined_head']) is bool and type(report['checkpoint_head']) is bool and
            report['combined_head'] != report['checkpoint_head'], 'head identity')
    require(report['target_taps_trace_checked'] is True and
            report['branch_and_budget_pressure'] is True and
            report['zero_drain'] is True, 'pressure gates')
    if 'tuning' in report:
        tuning = report['tuning']
        require(type(tuning['width']) is int and tuning['width'] in [2, 4, 8] and
                type(tuning['adaptive']) is bool, 'tuning policy')
        require(any(math.isclose(tuning['confidence'], value, abs_tol=1e-6)
                    for value in [-1, .8, .9, .95, .98]), 'confidence policy')
        require(math.isfinite(report['proposal_seconds']) and
                0 <= report['proposal_seconds'] <= report['seconds'], 'proposal envelope')
    names = [line.split('\t')[0] for line in
             (ROOT / 'fixtures/native-speculative-prompts.tsv').read_text().splitlines()]
    require(len(report['rows']) == 4, 'coverage')
    for i, row in enumerate(report['rows']):
        require(row['prompt'] == names[i] and row['tokens'] == 32 and
                row['ids'] == frozen['tokens'][i][:32], 'frozen tokens')
        require(type(row['accepted']) is int and type(row['proposed']) is int and
                0 <= row['accepted'] <= row['proposed'], 'learned counters')
        require(math.isfinite(row['seconds']) and row['seconds'] > 0, 'timing')
    for total, row_key in [('tokens', 'tokens'), ('learned_proposed', 'proposed'),
                           ('learned_accepted', 'accepted')]:
        require(report[total] == sum(row[row_key] for row in report['rows']), 'totals')
    require(math.isfinite(report['seconds']) and report['seconds'] > 0 and
            math.isclose(report['seconds'], sum(row['seconds'] for row in report['rows']),
                         rel_tol=1e-5, abs_tol=1e-5), 'total time')
    require(math.isclose(report['decode_tok_s'], report['tokens'] / report['seconds'],
                         rel_tol=1e-12), 'rate')
    generated, admitted = report['generated_proposals'], report['learned_proposed']
    require(type(generated) is int and generated >= admitted > 0, 'proposal coverage')
    return {'decode_tok_s': report['decode_tok_s'],
            'accepted': report['learned_accepted'], 'admitted': admitted,
            'generated': generated, 'coverage': admitted / generated,
            'acceptance_floor_pass': report['learned_accepted'] * 172 >= admitted * 166,
            'combined_125_gate_pass': False, 'production_promoted': False}


if __name__ == '__main__':
    import sys
    print(json.dumps(validate(json.loads(Path(sys.argv[1]).read_text()),
                             json.loads(FROZEN.read_text())), indent=2))
