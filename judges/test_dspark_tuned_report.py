"""Tuned acceptance is reported with coverage, not promoted as production proof."""
import copy
import json
import unittest
from native_dspark_report import ROOT, FROZEN, validate


class TunedReport(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads((ROOT / 'evidence/dspark-tuning/w4-c90.json').read_text())
        cls.frozen = json.loads(FROZEN.read_text())

    def test_acceptance_with_explicit_coverage(self):
        result = validate(self.report, self.frozen)
        self.assertTrue(result['acceptance_floor_pass'])
        self.assertEqual((result['accepted'], result['admitted'], result['generated']),
                         (59, 59, 194))
        self.assertFalse(result['production_promoted'])
        self.assertFalse(result['combined_125_gate_pass'])

    def test_corrupt_policy_or_time(self):
        changes = [lambda d: d['tuning'].update(width=3),
                   lambda d: d['tuning'].update(confidence=float('nan')),
                   lambda d: d['tuning'].update(adaptive='yes'),
                   lambda d: d.update(proposal_seconds=d['seconds']+1)]
        for change in changes:
            bad = copy.deepcopy(self.report)
            change(bad)
            with self.assertRaises((AssertionError, ValueError, RuntimeError)):
                validate(bad, self.frozen)
