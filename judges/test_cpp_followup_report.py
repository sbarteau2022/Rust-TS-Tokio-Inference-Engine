"""Reject lost controls, forged policy, timing corruption and scope overclaims."""
import copy
import json
import unittest
from cpp_followup_report import ROOT, FROZEN, PRESETS, summarize


class FollowupReport(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        directory = ROOT / 'evidence/decode-arch-kernel-proof'
        cls.reports = {name: json.loads((directory / (name + '.json')).read_text())
                       for name in PRESETS}
        cls.frozen = json.loads(FROZEN.read_text())

    def test_real_controls(self):
        report = summarize(self.reports, self.frozen)
        self.assertTrue(all(report['width8_acceptance_floor'].values()))
        self.assertFalse(report['combined_125_gate_pass'])

    def test_corrupt_controls(self):
        mutations = [lambda d: d.pop('width4-repeat'),
                     lambda d: d['w8-m250'].update(draft_margin=1.25),
                     lambda d: d['noalloc-single'].update(dspark_execution=True),
                     lambda d: d['noalloc-single'].update(logical_bytes_at_shutdown=1),
                     lambda d: d['phase']['rows'][0].update(transaction_phase_seconds={}),
                     lambda d: d['phase']['rows'][0]['transaction_phase_seconds'].update(verify=float('nan')),
                     lambda d: d['phase']['rows'][0]['transaction_phase_seconds'].update(verify=1000)]
        for i, mutation in enumerate(mutations):
            with self.subTest(corruption=i):
                bad = copy.deepcopy(self.reports)
                mutation(bad)
                with self.assertRaises((AssertionError, ValueError, RuntimeError)):
                    summarize(bad, self.frozen)
