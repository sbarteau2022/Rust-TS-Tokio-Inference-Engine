"""Adversarial standalone-plan checks; invalid plans never load model weights."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

BINARY = os.environ.get('NATIVE_CPP_BENCH')
PLAN = os.environ.get('NATIVE_CPP_PLAN')


@unittest.skipUnless(BINARY and PLAN, 'requires explicit standalone binary and checked plan')
class NativeCppPlan(unittest.TestCase):
    def invoke(self, plan, report):
        return subprocess.run([BINARY, str(plan), str(report)], capture_output=True,
                              text=True, timeout=10)

    def test_malformed_descriptors_refused_without_report(self):
        original = Path(PLAN).read_text()
        lines = original.splitlines()
        mutations = [(0, 'UNKNOWN'), (1, 'invalid'), (2, '64 9'), (3, '5'),
                     (4, '"relative"'), (9, '4097')]
        variants = []
        for index, value in mutations:
            bad = lines.copy()
            bad[index] = value
            variants.append('\n'.join(bad) + '\n')
        bad = lines.copy()
        tokens = bad[-1].split()
        tokens[0] = '248320'
        bad[-1] = ' '.join(tokens)
        variants.extend(['\n'.join(bad) + '\n', original + 'trailing\n'])
        with tempfile.TemporaryDirectory() as directory:
            for i, content in enumerate(variants):
                with self.subTest(mutation=i):
                    plan = Path(directory) / f'bad-{i}.txt'
                    report = Path(directory) / f'report-{i}.json'
                    plan.write_text(content)
                    result = self.invoke(plan, report)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn('FAIL ', result.stderr)
                    self.assertFalse(report.exists())

    def test_plan_size_bound_precedes_loading(self):
        with tempfile.TemporaryDirectory() as directory:
            plan, report = Path(directory) / 'huge', Path(directory) / 'report'
            plan.write_text('x' * 262145)
            result = self.invoke(plan, report)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('plan file size bound', result.stderr)
            self.assertFalse(report.exists())

    def test_existing_report_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / 'existing.json'
            report.write_text('keep this receipt')
            result = self.invoke(Path(directory) / 'nonexistent-plan', report)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('report already exists', result.stderr)
            self.assertEqual(report.read_text(), 'keep this receipt')


if __name__ == '__main__':
    unittest.main()
