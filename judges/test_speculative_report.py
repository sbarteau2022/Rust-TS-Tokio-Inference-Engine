"""Malformed retained measurements must fail closed, including nonfinite timings."""
import copy
import unittest
from speculative_report import IDENTITY, inventory, judge


def sample():
    rows = [dict(suite=s, id=i, generated=[1, 2], prompt_ids=[3], terminal=False, quality_pass=None if s == 'agent-smoke' else False, state_error=[0., 0.], proposed=1, accepted=1, target_calls=1, replayed=0, baseline_seconds=.2, speculative_seconds=.1) for s, i in sorted(inventory())]
    summary = dict(pass_=True, identity=IDENTITY, requests=25, frozen_free_greedy_requests=21, authored_agent_smoke_requests=4, boundary_cases=19, context_edge=512, cancellation=True, production_qualified=False, trained_mtp=False, scheduler_integrated=False, stochastic_sampling=False, uploaded_bytes=0, residents_after_drain=0, per_position_logit_maxima=[0., 0.], block_state_maxima=[0., 0.], retrieval_proposed=25, retrieval_accepted=25, retrieval_target_calls=25, retrieval_committed_tokens=50, retrieval_acceptance=1.)
    summary['pass'] = summary.pop('pass_')
    return summary, rows


class Evidence(unittest.TestCase):
    def test_rates_and_bad_receipts(self):
        summary, rows = sample()
        result = judge(summary, rows)
        self.assertEqual(result['groups']['agent-smoke']['speculative_decode_tok_s'], 20.)
        for key, value in [('baseline_seconds', float('nan')), ('speculative_seconds', 0), ('accepted', 2), ('target_calls', -1), ('proposed', True), ('state_error', [1., 0.]), ('generated', [248320]), ('quality_pass', True)]:
            damaged = copy.deepcopy(rows)
            authored = next(r for r in damaged if r['suite'] == 'agent-smoke')
            authored[key] = value
            with self.assertRaises(AssertionError, msg=key):
                judge(summary, damaged)
        with self.assertRaises(AssertionError):
            judge(summary, rows[:-1] + [rows[0]])
        for key, value in [('identity', 'wrong'), ('retrieval_accepted', 24), ('retrieval_acceptance', .5), ('trained_mtp', True), ('residents_after_drain', 1)]:
            damaged = dict(summary, **{key: value})
            with self.assertRaises(AssertionError, msg=key):
                judge(damaged, rows)


if __name__ == '__main__':
    unittest.main()
