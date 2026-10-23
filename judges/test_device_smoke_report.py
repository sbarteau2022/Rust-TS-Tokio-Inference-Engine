import copy
import json
from pathlib import Path
import tempfile
import unittest
from device_smoke_report import report
class SmokeReportTests(unittest.TestCase):
    def fixture(self,directory):
        windows=[dict(mode=m,concurrency=c) for m in ["Snapshot","Device"] for c in [1,3]]
        rows=[dict(mode=w["mode"],concurrency=w["concurrency"],id=i,result=dict(tokens=[1,2,3],token_times_ms=[10.,20.,30.])) for w in windows for i in range(6)]
        (directory/"summary.json").write_text(json.dumps(dict(windows=windows)))
        return rows
    def test_rates_and_incomplete_nonfinite_duplicate_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);rows=self.fixture(directory)
            def write(v): (directory/"requests.jsonl").write_text("\n".join(json.dumps(r) for r in v))
            write(rows);result=report(directory)
            self.assertEqual(result["windows"][0]["per_lane_decode_tps_p50"],100.)
            self.assertIsNone(result["windows"][0]["speculative_acceptance_percent"])
            for invalid in [rows[:-1],rows+[rows[0]]]:
                write(invalid)
                with self.assertRaises(ValueError): report(directory)
            for times in [[1.,float("nan"),3.],[1.,3.,2.],[1.,1.,1.],[-1.,2.,3.]]:
                invalid=copy.deepcopy(rows);invalid[0]["result"]["token_times_ms"]=times;write(invalid)
                with self.assertRaises(ValueError): report(directory)
if __name__=="__main__":unittest.main()
