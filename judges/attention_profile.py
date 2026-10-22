"""Isolated attention profiles for external experiments; no server imports this module."""
import suite_reference as base


def explicit_fp32(q, k, v, cache, scale, mask, sinks=None):
    import mlx.core as mx
    base.require(sinks is None and not hasattr(cache, 'bits'), 'unsupported attention sinks/quantization')
    base.require(q.ndim == k.ndim == v.ndim == 4 and q.shape[0] == k.shape[0] == v.shape[0] == 1, 'attention batch/shape')
    base.require(q.dtype == k.dtype == v.dtype == mx.float32, 'attention F32 required')
    base.require(all(d > 0 for a in [q, k, v] for d in a.shape), 'positive attention dimensions')
    base.require(k.shape == v.shape and q.shape[-1] == k.shape[-1] and q.shape[1] % k.shape[1] == 0, 'attention GQA geometry')
    count, end = q.shape[2], k.shape[2]
    offset = end - count
    base.require(0 < count <= 8 and 0 <= offset and end <= 512, 'attention context bounds')
    base.require((isinstance(mask, str) and mask == 'causal') or (mask is None and count == 1), 'unsupported attention mask')
    base.require(scale == q.shape[-1] ** -0.5, 'attention scale')
    rows = []
    for row in range(count):
        grouped = q[:, :, row:row+1, :].reshape(1, k.shape[1], q.shape[1] // k.shape[1], 1, q.shape[-1])
        keys = mx.expand_dims(k[:, :, :offset+row+1, :], 2)
        values = mx.expand_dims(v[:, :, :offset+row+1, :], 2)
        scores = (grouped * keys).sum(axis=-1) * scale
        probabilities = mx.expand_dims(mx.softmax(scores, axis=-1, precise=True), -1)
        rows.append((probabilities * values).sum(axis=-2).reshape(1, q.shape[1], 1, q.shape[-1]))
    return mx.concatenate(rows, axis=2)


def fp64_oracle(q, k, v, scale):
    import numpy as np
    base.require(q.ndim == k.ndim == v.ndim == 4 and q.shape[0] == k.shape[0] == v.shape[0] == 1, 'oracle batch')
    base.require(all(d > 0 for a in [q, k, v] for d in a.shape), 'positive attention dimensions')
    base.require(k.shape == v.shape and q.shape[-1] == k.shape[-1] and q.shape[1] % k.shape[1] == 0, 'oracle geometry')
    count, end = q.shape[2], k.shape[2]
    base.require(0 < count <= 8 and count <= end <= 512, 'oracle context')
    base.require(all(np.isfinite(a).all() for a in [q, k, v]), 'oracle nonfinite')
    base.require(scale == q.shape[-1] ** -0.5, 'oracle scale')
    q, k, v = [a.astype(np.float64) for a in [q, k, v]]
    k, v = [np.repeat(a, q.shape[1] // a.shape[1], axis=1) for a in [k, v]]
    scores = (q @ k.swapaxes(-1, -2)) * scale
    visible = np.arange(end)[None, :] <= (end-count+np.arange(count))[:, None]
    scores = np.where(visible, scores, -np.inf)
    probabilities = np.exp(scores - scores.max(axis=-1, keepdims=True))
    probabilities /= probabilities.sum(axis=-1, keepdims=True)
    return probabilities @ v
