import copy
import json
import unittest
from pathlib import Path
from edit_audit_report import validate


class AuditReaderTests(unittest.TestCase):
    def setUp(self):
        self.report = json.loads(Path('evidence/native-edit-audit/attention-reference/report.json').read_text())
        self.frozen = json.loads(Path('evidence/native-mtp-stream-matrix/raw/references.json').read_text())

    def test_real_report(self):
        self.assertTrue(validate(self.report, self.frozen))

    def test_real_stage_inventory_and_corruption(self):
        report = json.loads(Path('evidence/native-edit-audit/operator-stages/report.json').read_text())
        self.assertTrue(validate(report, self.frozen))
        for field in ('calls', 'host_ns', 'wait_ns'):
            damaged = copy.deepcopy(report)
            damaged['timings'][0]['decode_operators']['moe_down'][field] = 0
            with self.assertRaises(AssertionError):
                validate(damaged, self.frozen)

    def test_adversarial_claims(self):
        mutations = [
            lambda r: r['observations'][0]['logits'].update(nrmse=float('nan')),
            lambda r: r['observations'][0]['state'].update(max_abs=0.01),
            lambda r: r['timings'][0]['tokens'].__setitem__(0, 0),
            lambda r: r.update(attention_kernel='fused'),
            lambda r: r.update(native_residents=1),
            lambda r: r['observations'].pop(),
            lambda r: r['timings'][0].update(decode_tok_s=100),
            lambda r: r['timings'][0]['decode_operators']['embedding'].update(calls=1),
            lambda r: r.update(identity='wrong'),
        ]
        for mutate in mutations:
            with self.subTest(mutation=mutate):
                report = copy.deepcopy(self.report)
                mutate(report)
                with self.assertRaises(AssertionError):
                    validate(report, self.frozen)


if __name__ == '__main__':
    unittest.main()
