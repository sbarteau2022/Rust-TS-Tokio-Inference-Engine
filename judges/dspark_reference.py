"""Test-only installed reference oracle. Never used by native serving."""
import json
import sys
from pathlib import Path
import mlx.core as mx
import mlx.nn as nn
from mlx_vlm.speculative.drafters.dspark.config import DSparkConfig
from mlx_vlm.speculative.drafters.dspark.dspark import DSparkDraftModel

artifact, destination = map(Path, sys.argv[1:3])
bf16 = len(sys.argv) == 4 and sys.argv[3] == "bf16"
destination.mkdir(parents=True, exist_ok=True)
config = DSparkConfig.from_dict(json.loads((artifact / 'config.json').read_text()))
assert config.sample_from_anchor and not config.is_causal
assert config.target_layer_ids == [1, 6, 11, 16, 22, 27, 32, 37]
model = DSparkDraftModel(config)
model.lm_head = nn.Linear(2048, 248320, bias=False)
weights = model.sanitize(mx.load(str(artifact / 'model.safetensors')))
model.load_weights(list(weights.items()), strict=True)
if not bf16:
    model.apply(lambda value: value.astype(mx.float32))
mx.eval(model.parameters())
cache = model.make_cache()
inputs, expected = {}, {}
for index, (width, count) in enumerate([(2, 3), (4, 2), (8, 1), (2, 506)]):
    # Reproducible finite synthetic component inputs, not acceptance evidence.
    features = mx.sin(mx.arange(count * 16384, dtype=mx.float32) * 0.001 + index)
    features = features.reshape(1, count, 16384)
    anchor = 100 + index
    ids = mx.array([[anchor] + [config.mask_token_id] * (width - 2)], mx.uint32)
    inputs[f'taps{index}'] = features
    hidden = model._hidden(ids, features.astype(mx.bfloat16) if bf16 else features, cache)
    logits = model._logits(hidden)
    tokens = model.markov_head.sample_block(logits, first_prev_tokens=mx.array([anchor]),
                                          sampler=lambda scores: mx.argmax(scores, axis=-1))
    confidence = model.confidence(hidden, mx.array([anchor]), tokens)
    expected.update({f'hidden{index}': hidden, f'logits{index}': logits,
                     f'tokens{index}': tokens, f'confidence{index}': confidence})
    for layer, item in enumerate(cache):
        expected[f'key{index}_{layer}'] = item.keys[:, :, :item.offset]
        expected[f'value{index}_{layer}'] = item.values[:, :, :item.offset]
    mx.eval(expected)
mx.save_safetensors(str(destination / 'inputs.safetensors'), inputs)
mx.save_safetensors(str(destination / 'expected.safetensors'), expected)
(destination / 'reference.json').write_text(json.dumps({
    'reference': 'installed DSparkDraftModel FP32, checkpoint own-head lane',
    'widths': [2, 4, 8, 2], 'contexts': [3, 5, 6, 512], 'target_integration': False,
    'acceptance_qualification': False}, indent=2) + '\n')
