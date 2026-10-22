"""Pure adversarial tests: no model or MLX dependency."""
import copy
import importlib.util
import math
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('suite_reference', Path(__file__).with_name('suite_reference.py'))
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)


class ReferenceTests(unittest.TestCase):
    def test_frozen_inventory_and_expectation_separation(self):
        cases = s.inventory()
        self.assertEqual([sum(c['suite'] == x for c in cases) for x in ['raw', 'contract', 'domain']], [16, 20, 24])
        self.assertEqual([sum(c['suite'] == x and c['choices'] is not None for c in cases) for x in ['raw', 'contract', 'domain']], [0, 5, 19])
        changed = copy.deepcopy(cases)
        for case in changed:
            case['expected'] = 'adversarial-wrong-label'
        self.assertEqual([(c['prompt'], c['choices']) for c in cases], [(c['prompt'], c['choices']) for c in changed])

    def test_shared_terminal_prefixes_and_fingerprint(self):
        paths = [[2], [2, 3], [4, 5]]
        self.assertEqual(s.allowed(paths, [], [9]), [2, 4])
        self.assertEqual(s.allowed(paths, [2], [9]), [3, 9])
        self.assertEqual(s.allowed(paths, [4, 5], [9]), [9])
        self.assertEqual(s.allowed(paths, [7], [9]), [])
        self.assertEqual(s.contract(paths, 10, [9], 3), s.contract(paths[::-1], 10, [9, 9], 3))

    def test_bounds_and_nonfinite_fail_closed(self):
        for paths in [[], [[]], [[9]], [[10]], [[1], [1]], [[1]] * 17, [[1] * 65], [[-1]]]:
            with self.assertRaises(ValueError):
                s.contract(paths, 10, [9], 100)
        for values, legal in [([], None), ([math.nan], None), ([math.inf], None), ([0.], []), ([0.], [1]), ([0.], [-1])]:
            with self.assertRaises(ValueError):
                s.select(values, legal)
        with self.assertRaises(ValueError):
            s.contract([[1, 2]], 10, [9], 2)

    def test_ties_mask_and_signed_zero(self):
        self.assertEqual(s.select([1., 5., 5.], [2, 1]), 1)
        self.assertEqual(s.select([100., 2., 3.], [1, 2]), 2)
        self.assertEqual(s.select([-0., 0.]), 1)
        self.assertEqual(s.select([0., -0.]), 0)

    def test_missing_duplicate_truncated_and_wrong_model_reports_reject(self):
        def records():
            return [dict(case=c, identity=s.IDENTITY, profile='native-f32',
                         first_logits_file=f"{c['suite']}-{c['id']}.f32le",
                         finish='eos', generated=[9], text=c['expected'], quality_pass=True)
                    for c in s.inventory()]
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / 'drain.txt').write_text('leases=0 cases=60\n')
            def write(data):
                (root / 'native.jsonl').write_text(''.join(json.dumps(r) + '\n' for r in data))
            good = records()
            write(good)
            self.assertEqual(len(s.native_records(root)), 60)
            for mutation in ['missing', 'duplicate', 'model', 'quality', 'path']:
                data = records()
                if mutation == 'missing': data.pop()
                if mutation == 'duplicate': data[1] = data[0]
                if mutation == 'model': data[0]['identity'] = 'wrong'
                if mutation == 'quality': data[0]['quality_pass'] = False
                if mutation == 'path': data[0]['first_logits_file'] = '../../escape'
                write(data)
                with self.assertRaises(ValueError):
                    s.native_records(root)
            (root / 'native.jsonl').write_text('{')
            with self.assertRaises(ValueError):
                s.native_records(root)
            write(good)
            (root / 'drain.txt').write_text('leases=1 cases=60\n')
            with self.assertRaises(ValueError):
                s.native_records(root)

    def test_full_vocabulary_drift_and_invalid_vectors(self):
        try:
            import numpy as np
        except ModuleNotFoundError:
            self.skipTest('NumPy vector tests run in stock reference environment')
        x = np.linspace(-8, 8, 248320, dtype=np.float32)
        y = x + np.float32(0.25)
        result = s.compare_vectors(x, y)
        self.assertLess(abs(result['max_abs'] - 0.25), 1e-6)
        self.assertLess(result['centered_nrmse'], 1e-7)
        self.assertGreater(result['nrmse'], 0)
        self.assertEqual(s.compare_vectors(x, x)['max_abs'], 0)
        for a, b in [(x[:-1], x), (np.full_like(x, np.nan), x), (np.full_like(x, np.inf), x)]:
            with self.assertRaises(ValueError):
                s.compare_vectors(a, b)

    def test_wrong_view_inventory_rejects_before_inference(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(ValueError):
                s.verify_view(Path(d))
            (Path(d) / 'model-mtp.safetensors').touch()
            with self.assertRaises(ValueError):
                s.verify_view(Path(d))


if __name__ == '__main__':
    unittest.main()
