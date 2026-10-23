"""Adversarial accounting checks using the retained native control observation."""
import copy,json,subprocess,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
class Report(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.baseline=json.loads((ROOT/'evidence/native-layer-groups/group-1/report.json').read_text())
    def run_report(self,mutate):
        d=copy.deepcopy(self.baseline);mutate(d)
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'candidate';p.mkdir();(p/'report.json').write_text(json.dumps(d))
            return subprocess.run([sys.executable,str(ROOT/'judges/layer_groups_report.py'),tmp],capture_output=True).returncode
    def test_control(self): self.assertEqual(self.run_report(lambda d:None),0)
    def test_missing_observation(self): self.assertNotEqual(self.run_report(lambda d:d['observations'].pop()),0)
    def test_wrong_token(self): self.assertNotEqual(self.run_report(lambda d:d['timings'][0]['tokens'].__setitem__(1,0)),0)
    def test_hidden_numeric_failure(self): self.assertNotEqual(self.run_report(lambda d:d['observations'][0]['state'].__setitem__('max_abs',1)),0)
    def test_nonfinite(self): self.assertNotEqual(self.run_report(lambda d:d['observations'][0]['logits'].__setitem__('nrmse',float('nan'))),0)
    def test_fabricated_throughput(self): self.assertNotEqual(self.run_report(lambda d:d['timings'][0].__setitem__('decode_tok_s',1000)),0)
    def test_residue(self): self.assertNotEqual(self.run_report(lambda d:d.__setitem__('native_bytes',4)),0)
    def test_unknown_kernel(self): self.assertNotEqual(self.run_report(lambda d:d.__setitem__('recurrent_kernel','auto')),0)
if __name__=='__main__':unittest.main()
