"""Invalid tuning flags must fail before reading a plan or loading weights."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


@unittest.skipUnless(os.environ.get('DSPARK_TUNING_BIN'), 'requires explicit tuning binary')
class TuningCLI(unittest.TestCase):
    def test_bad_flags_precede_model_loading(self):
        binary = os.environ['DSPARK_TUNING_BIN']
        cases = [(['--width=3'], 'width must'),
                 (['--confidence=nan'], 'unsupported confidence'),
                 (['--width=4', '--width=8'], 'duplicate width'),
                 (['--adaptive', '--adaptive'], 'duplicate adaptive'),
                 (['bf16', 'checkpoint-head'], 'duplicate head'),
                 (['--unknown'], 'unknown DSpark')]
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / 'existing'
            report.write_text('preserve')
            for args, message in cases:
                with self.subTest(args=args):
                    run = subprocess.run([binary, '/missing-plan', '/missing-model',
                                          str(report)] + args, capture_output=True,
                                         text=True, timeout=10)
                    self.assertNotEqual(run.returncode, 0)
                    self.assertIn(message, run.stderr)
                    self.assertEqual(report.read_text(), 'preserve')


class SweepEvidence(unittest.TestCase):
    def test_existing_sweep_directory_never_overwritten(self):
        import sys
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            files = [base / name for name in ['binary', 'plan', 'drafter']]
            for path in files:
                path.write_text('unused')
            output = base / 'evidence'
            output.mkdir()
            marker = output / 'summary.json'
            marker.write_text('preserve')
            run = subprocess.run([sys.executable, str(root / 'scripts/run_dspark_tuning.py')]
                                 + [str(path) for path in files] + [str(output)],
                                 capture_output=True, text=True, timeout=10)
            self.assertNotEqual(run.returncode, 0)
            self.assertIn('FileExistsError', run.stderr)
            self.assertEqual(marker.read_text(), 'preserve')
            self.assertEqual(len(list(output.iterdir())), 1)
