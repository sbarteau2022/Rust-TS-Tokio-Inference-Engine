import copy
import json
import unittest
from pathlib import Path
from tonight_kernel_report import benchmark, component


class MtpAccounting(unittest.TestCase):
    def setUp(self):
        self.report = json.loads(Path('evidence/tonight-kernel/mtp-postnorm-fixed-accounting/summary.json').read_text())
        self.frozen = json.loads(Path('evidence/native-mtp-stream-matrix/raw/references.json').read_text())

    def test_real(self):
        self.assertEqual(benchmark(self.report, self.frozen)['mtp-width4-reuse']['accepted'], 166)

    def test_corrupted_claims(self):
        for field, value in [('tokens', 64), ('decode_seconds', float('nan')), ('decode_tokens_per_second', 150), ('learned_accepted', 1000), ('target_calls', 0)]:
            damaged = copy.deepcopy(self.report)
            damaged['rows'][1][field] = value
            with self.assertRaises(AssertionError):
                benchmark(damaged, self.frozen)
        damaged = copy.deepcopy(self.report)
        damaged['rows'][1]['ids'][0] = 0
        with self.assertRaises(AssertionError):
            benchmark(damaged, self.frozen)

    def test_duplicate_row_and_identity(self):
        for mutate in [lambda r: r['rows'].__setitem__(1, r['rows'][0]), lambda r: r.update(identity='other')]:
            damaged = copy.deepcopy(self.report)
            mutate(damaged)
            with self.assertRaises(AssertionError):
                benchmark(damaged, self.frozen)

class ComponentClaims(unittest.TestCase):
    def test_real_full_cap_and_shapes(self):
        report=json.loads(Path('evidence/tonight-kernel/mtp-dense-rowwise/summary.json').read_text())
        self.assertTrue(component(report)['drained'])

    def test_corrupt_component_shape_drain_and_tolerance(self):
        original=json.loads(Path('evidence/tonight-kernel/mtp-dense-rowwise/summary.json').read_text())
        for mutate in [lambda r:r.update(context=1024),lambda r:r.update(logical_bytes_at_shutdown=4),lambda r:r['cases'].pop(),lambda r:r['cases'][1].update(kv_relative_l2=float('nan')),lambda r:r['cases'][1].update(logit_relative_l2=0.001)]:
            report=copy.deepcopy(original);mutate(report)
            with self.assertRaises(AssertionError):component(report)
