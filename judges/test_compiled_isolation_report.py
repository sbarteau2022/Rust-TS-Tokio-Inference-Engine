import copy
import json
import unittest
from pathlib import Path
from compiled_isolation_report import gate


class CompiledJournal(unittest.TestCase):
    def setUp(self):
        root = Path('evidence/compiled-isolation/shared-constants-group4')
        self.report = json.loads((root / 'summary.json').read_text())
        self.journal = [json.loads(s) for s in (root / 'progress.jsonl').read_text().splitlines()]
        self.reference = json.loads(Path('evidence/tonight-kernel/mtp-eager-committed/summary.json').read_text())

    def test_completed(self):
        self.assertTrue(gate(self.report, self.journal, self.reference)['pass'])

    def test_omitted_repeated_or_reordered_phase(self):
        for mutate in [lambda j: j.pop(), lambda j: j.append(j[-1]), lambda j: j.reverse()]:
            journal = copy.deepcopy(self.journal)
            mutate(journal)
            with self.assertRaises(AssertionError):
                gate(self.report, journal, self.reference)

    def test_memory_clock_and_output_corruption(self):
        for field, value in [('peak_bytes', 55 * 1024**3), ('active_bytes', 0), ('compiled_entries', 1115)]:
            journal = copy.deepcopy(self.journal)
            next(r for r in journal if r['phase'] == 'start.memory')[field] = value
            with self.assertRaises(AssertionError):
                gate(self.report, journal, self.reference)
        for value in [float('nan'), -1, 180000]:
            journal = copy.deepcopy(self.journal)
            journal[-1]['elapsed_ms'] = value
            with self.assertRaises(AssertionError):
                gate(self.report, journal, self.reference)
        report = copy.deepcopy(self.report)
        report['cases'][0]['tokens'][0] += 1
        with self.assertRaises(AssertionError):
            gate(report, self.journal, self.reference)
