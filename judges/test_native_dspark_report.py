"""Adversarial independent checks prevent a failed DSpark gate becoming a headline."""
import copy
import json
import unittest
from native_dspark_report import ROOT, FROZEN, validate


class DSparkReport(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((ROOT / 'evidence/native-dspark2/accounted-integration.json').read_text())
        cls.frozen = json.loads(FROZEN.read_text())

    def test_actual_gate_stays_unqualified(self):
        result = validate(self.report, self.frozen)
        self.assertFalse(result['acceptance_floor_pass'])
        self.assertFalse(result['combined_125_gate_pass'])
        self.assertFalse(result['production_promoted'])

    def test_corruption(self):
        changes = [lambda d: d['rows'][0]['ids'].__setitem__(0, 0),
                   lambda d: d.update(learned_accepted=d['learned_proposed']+1),
                   lambda d: d.update(checkpoint_head=d['combined_head']),
                   lambda d: d.update(decode_tok_s=d['decode_tok_s']*2),
                   lambda d: d.update(generated_proposals=1),
                   lambda d: d.update(turboquant_execution=True),
                   lambda d: d.update(zero_drain=False),
                   lambda d: d['rows'][0].update(seconds=float('nan'))]
        for index, change in enumerate(changes):
            with self.subTest(corruption=index):
                bad = copy.deepcopy(self.report)
                change(bad)
                with self.assertRaises((AssertionError, ValueError, RuntimeError)):
                    validate(bad, self.frozen)
