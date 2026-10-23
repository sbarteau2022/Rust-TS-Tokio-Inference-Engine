import json,shutil,tempfile,unittest
from pathlib import Path
from compiled_gate_report import ROOT,verify
class Gates(unittest.TestCase):
    def mutate(self,file,change):
        with tempfile.TemporaryDirectory() as t:
            out=Path(t)/'run';shutil.copytree(ROOT/'evidence/native-compiled-decode',out)
            p=out/file
            if p.suffix=='.jsonl':
                d=[json.loads(s) for s in p.read_text().splitlines()];change(d);p.write_text(''.join(json.dumps(r)+'\n' for r in d))
            else:d=json.loads(p.read_text());change(d);p.write_text(json.dumps(d))
            return verify(out)
    def test_control(self):self.assertTrue(verify(ROOT/'evidence/native-compiled-decode')['pass'])
    def test_changed_heldout_token(self):
        with self.assertRaises(AssertionError):self.mutate('heldout/suite.jsonl',lambda d:d[0]['generated'].__setitem__(0,0))
    def test_omitted_case(self):
        with self.assertRaises(AssertionError):self.mutate('heldout/suite.jsonl',lambda d:d.pop())
    def test_forged_packed_gate(self):
        with self.assertRaises(AssertionError):self.mutate('packed/summary.json',lambda d:d['observations'][0]['state'].__setitem__('max_abs',0.1))
    def test_context_failure(self):
        with self.assertRaises(AssertionError):self.mutate('pressure/summary.json',lambda d:d.__setitem__('fourth_step_rejected',False))
    def test_mislabeled_graph(self):
        with self.assertRaises(AssertionError):self.mutate('http/summary.json',lambda d:d.__setitem__('graph_mode','eager'))
    def test_wrong_model(self):
        with self.assertRaises(AssertionError):self.mutate('heldout/summary.json',lambda d:d.__setitem__('identity','other'))
if __name__=='__main__':unittest.main()
