"""Fail-closed checks for compiler inventory, profiler accounting and retained tokens."""
import copy,json,tempfile,unittest
from pathlib import Path
from compiled_execution_report import ROOT,verify
class Report(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.baseline=json.loads((ROOT/'evidence/native-compiled-decode/compiled-timing/report.json').read_text())
    def check(self,mutate):
        d=copy.deepcopy(self.baseline);mutate(d)
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'report.json';p.write_text(json.dumps(d));return verify(p)
    def test_valid(self):self.assertTrue(self.check(lambda d:None)['pass'])
    def test_inventory(self):
        with self.assertRaises(AssertionError):self.check(lambda d:d.__setitem__('compiled_entries',91))
    def test_silent_eager(self):
        with self.assertRaises(AssertionError):self.check(lambda d:d.__setitem__('graph_mode','eager'))
    def test_completed_exceeds_calls(self):
        with self.assertRaises(AssertionError):self.check(lambda d:d['timings'][0]['decode_execution'].__setitem__('completed',64))
    def test_impossible_wait(self):
        with self.assertRaises(AssertionError):self.check(lambda d:d['timings'][0]['decode_execution'].__setitem__('wait_ns',10**15))
    def test_fake_disabled(self):
        with self.assertRaises(AssertionError):self.check(lambda d:d.__setitem__('execution_timing','off'))
    def test_missing_observation(self):
        with self.assertRaises(Exception):self.check(lambda d:d['observations'].pop())
    def test_wrong_token(self):
        with self.assertRaises(Exception):self.check(lambda d:d['timings'][0]['tokens'].__setitem__(0,0))
if __name__=='__main__':unittest.main()
