import copy
import json
import unittest
from pathlib import Path
from mtp_phase_report import validate


class PhaseAccounting(unittest.TestCase):
    def setUp(self):
        root = Path('evidence/mtp-phase-costs')
        self.control = json.loads((root / 'control/summary.json').read_text())
        self.timed = json.loads((root / 'timed/summary.json').read_text())
        self.frozen = json.loads(Path('evidence/native-mtp-stream-matrix/raw/references.json').read_text())

    def test_real_partition(self):
        report = validate(self.control, self.timed, self.frozen)
        self.assertEqual(report['totals']['emitted'], 256)
        self.assertAlmostEqual(sum(report['transaction_fraction'].values()), 1)

    def test_dropped_receipt_and_counter_corruption(self):
        for mutate in [lambda r: r['mtp_phase_receipts'].pop(), lambda r: r['mtp_phase_receipts'][0].update(emitted=0), lambda r: r['mtp_phase_receipts'][0].update(learned_accepted=100), lambda r: r['mtp_phase_receipts'][0].update(replayed_target_tokens=8)]:
            timed = copy.deepcopy(self.timed)
            row = next(r for r in timed['rows'] if r['profile'].startswith('mtp'))
            mutate(row)
            with self.assertRaises(AssertionError):
                validate(self.control, timed, self.frozen)

    def test_impossible_or_mislabeled_timings(self):
        for mutate in [lambda r: r.update(phase_timing=False), lambda r: r.update(graph_mode='compiled-decode'), lambda r: r.update(expert_kernel='fused'), lambda r: r.update(capture_wait=True)]:
            timed = copy.deepcopy(self.timed)
            mutate(timed)
            with self.assertRaises(AssertionError):
                validate(self.control, timed, self.frozen)
        for field, value in [('draft_ns', -1), ('verify_ns', float('nan')), ('commit_ns', 0), ('refresh_ns', True), ('replay_ns', 2**64)]:
            timed = copy.deepcopy(self.timed)
            row = next(r for r in timed['rows'] if r['profile'].startswith('mtp'))
            row['mtp_phase_receipts'][0]['timing'][field] = value
            with self.assertRaises(AssertionError):
                validate(self.control, timed, self.frozen)

    def test_prefill_and_target_scope(self):
        for target in [False, True]:
            timed = copy.deepcopy(self.timed)
            row = next(r for r in timed['rows'] if (r['profile'] == 'target-greedy') == target)
            if target:
                row['mtp_prefill_timing'] = {}
            else:
                row['mtp_prefill_timing']['target_prefill_ns'] = 2**64 - 1
            with self.assertRaises(AssertionError):
                validate(self.control, timed, self.frozen)

    def test_candidate_requires_same_instrumentation_and_explicit_kernel(self):
        fused = json.loads(Path('evidence/mtp-phase-costs/fused/summary.json').read_text())
        self.assertTrue(validate(self.timed, fused, self.frozen, candidate=True)['pass'])
        with self.assertRaises(AssertionError):
            validate(self.control, fused, self.frozen, candidate=True)
        with self.assertRaises(AssertionError):
            validate(self.timed, self.timed, self.frozen, candidate=True)
