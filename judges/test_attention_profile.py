"""Independent causal/GQA checks; no model weights required."""
import unittest
import attention_profile as attention


class AttentionTests(unittest.TestCase):
    def arrays(self):
        try:
            import numpy as np
        except ModuleNotFoundError:
            self.skipTest('NumPy tests run in external stock environment')
        return np

    def test_fp64_cached_causal_visibility_and_grouping(self):
        np = self.arrays()
        q = np.zeros((1, 4, 3, 4), dtype=np.float32)
        k = np.ones((1, 2, 7, 4), dtype=np.float32)
        v = np.broadcast_to(np.arange(7)[None, None, :, None], (1, 2, 7, 4)).astype(np.float32).copy()
        v[:, 1] += 10
        actual = attention.fp64_oracle(q, k, v, 0.5)
        expected = np.broadcast_to(np.array([2., 2.5, 3.])[None, None, :, None], actual.shape).copy()
        expected[:, 2:] += 10
        np.testing.assert_allclose(actual, expected, atol=1e-12, rtol=0)
        for bad in [q[..., :0, :], q[:, :0], np.full_like(q, np.nan)]:
            with self.assertRaises(ValueError):
                attention.fp64_oracle(bad, k, v, 0.5)

    def test_mlx_explicit_matches_oracle_and_rejects_unsupported(self):
        np = self.arrays()
        try:
            import mlx.core as mx
        except ModuleNotFoundError:
            self.skipTest('MLX operator tests run in external stock environment')
        rng = np.random.default_rng(1451)
        q = rng.normal(size=(1, 4, 3, 4)).astype(np.float32)
        k = rng.normal(size=(1, 2, 7, 4)).astype(np.float32)
        v = rng.normal(size=(1, 2, 7, 4)).astype(np.float32)
        mq, mk, mv = [mx.array(a) for a in [q, k, v]]
        actual = attention.explicit_fp32(mq, mk, mv, None, 0.5, 'causal')
        np.testing.assert_allclose(np.array(actual), attention.fp64_oracle(q,k,v,0.5), atol=2e-6, rtol=2e-6)
        one = attention.explicit_fp32(mq[:, :, -1:, :], mk, mv, None, 0.5, None)
        np.testing.assert_allclose(np.array(one), attention.fp64_oracle(q[:, :, -1:, :],k,v,0.5), atol=2e-6, rtol=2e-6)
        for change in [dict(mask=None), dict(scale=1.), dict(sinks=mx.array([0.])), dict(q=mq.astype(mx.bfloat16))]:
            args = dict(q=mq,k=mk,v=mv,cache=None,scale=0.5,mask='causal')
            args.update(change)
            with self.assertRaises(ValueError):
                attention.explicit_fp32(**args)


if __name__ == '__main__':
    unittest.main()
