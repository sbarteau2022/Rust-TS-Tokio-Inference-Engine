#!/usr/bin/env python3
"""Private bounded JSONL worker: Rapid TurboQuant prefix codec + its MLX-LM model backend.
Regular/native model arithmetic is intentionally distinct from custom Q1.31 Metal kernels.
"""
import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
import time

MAX_LINE = 512 * 1024


def read_message():
    line = sys.stdin.buffer.readline(MAX_LINE + 1)
    if not line:
        return None
    if len(line) > MAX_LINE or not line.endswith(b"\n"):
        raise ValueError("oversized or truncated RPC")
    return json.loads(line)


def send(value):
    print(json.dumps(value, allow_nan=False), flush=True)


def file_digest(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(8 * 1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def inventory(directory):
    directory = Path(directory).resolve(strict=True)
    config = directory / "config.json"
    if config.stat().st_size > 1024 * 1024:
        raise ValueError("model configuration too large")
    cfg = json.loads(config.read_text())
    # Hash actual local artifacts, including tokenizer and optional MTP weights. Never execute remote code.
    paths = sorted(p for p in directory.iterdir() if p.is_file() and p.suffix in {".json", ".safetensors", ".model"})
    before = {p.name: (p.stat().st_size, p.stat().st_mtime_ns) for p in paths}
    hashes = {p.name: file_digest(p) for p in paths}
    if before != {p.name: (p.stat().st_size, p.stat().st_mtime_ns) for p in paths}:
        raise ValueError("model artifacts changed during fingerprinting")
    return directory, cfg, hashes, before


def main():
    init = read_message()
    if init is None or init.get("op") != "init":
        raise ValueError("init required")
    mode = init["codec"]
    if mode not in {"regular", "rapid-turboquant-v4", "rapid-turboquant-k8v4"}:
        raise ValueError("unsupported codec")
    directory, cfg, hashes, stamps = inventory(init["model"])
    import mlx.core as mx
    from mlx.utils import tree_flatten
    from mlx_lm import load
    from mlx_lm.models.cache import KVCache, make_prompt_cache
    from rapid_mlx.turboquant import TurboQuantConfig, TurboQuantKVCache
    import rapid_mlx.turboquant as rapid_codec
    versions = {name: importlib.metadata.version(name) for name in ["mlx", "mlx-lm", "rapid-mlx"]}
    source_hashes = {"worker": file_digest(Path(__file__)), "rapid_codec": file_digest(Path(rapid_codec.__file__))}
    identity = hashlib.sha256(json.dumps({"artifacts": hashes, "dependencies": versions, "source": source_hashes}, sort_keys=True).encode()).hexdigest()
    codec_identity = hashlib.sha256(json.dumps({"version": 1, "mode": mode, "bits": 4, "group_size": 32, "rotation_seed": 42, "source": source_hashes, "versions": versions}, sort_keys=True).encode()).hexdigest()
    mx.set_memory_limit(init["allocator_limit_bytes"])
    load_start = time.perf_counter()
    model, tokenizer = load(str(directory), tokenizer_config={"trust_remote_code": False})
    # Recheck metadata after loading, before any admission under the artifact fingerprint.
    if stamps != {name: ((directory / name).stat().st_size, (directory / name).stat().st_mtime_ns) for name in stamps}:
        raise ValueError("model artifacts changed during load")
    text_cfg = cfg.get("text_config", cfg)
    vocab = text_cfg["vocab_size"]
    eos = cfg.get("eos_token_id", text_cfg.get("eos_token_id", []))
    eos = eos if isinstance(eos, list) else [eos]
    prompts = []
    for text in init["prompts"]:
        rendered = tokenizer.apply_chat_template([{"role": "user", "content": text}], tokenize=False, add_generation_prompt=True, enable_thinking=False)
        prompts.append(tokenizer.encode(rendered, add_special_tokens=False))
    tq_config = TurboQuantConfig(bits=4, group_size=32, rotation_seed=42, mode="k8v4" if mode.endswith("k8v4") else "v4")
    sessions = {}
    records = []
    total_steps = 0
    diagnostic_records_omitted = 0
    probe_dir = Path(init["probe_directory"]).resolve() if init.get("probe_directory") else None
    if probe_dir:
        probe_dir.mkdir(parents=True, exist_ok=False)

    def arrays(cache):
        result = []
        for layer in cache:
            state = (layer.keys, layer.values_compressed, layer.keys_compressed) if isinstance(layer, TurboQuantKVCache) else layer.state
            result.extend(v for _, v in tree_flatten(state) if isinstance(v, mx.array))
        return result

    send({"ready": True, "identity": identity, "codec_fingerprint": codec_identity, "vocabulary_size": vocab, "eos_tokens": eos, "prompts": prompts, "versions": versions, "load_seconds": time.perf_counter()-load_start, "artifact_sha256": hashes, "numerical_contract": "MLX-native-quantized-weights-floating-activations", "codec_scope": "regular" if mode == "regular" else "prefix-storage-roundtrip; attention-materializes-native-KV", "vectorized_batching": False, "host_step_concurrency": 1, "mlx_native_command_buffer_cap_enforced": False})
    while (request := read_message()) is not None:
        if request["op"] == "release":
            sessions.pop(request["sequence"], None)
            mx.synchronize()
            send({"released": True})
        elif request["op"] == "step":
            outputs = []
            for item in request["batch"]:
                sid = item["sequence"]
                if sid not in sessions:
                    if item["position"] != 0 or len(sessions) >= init["max_sequences"]:
                        raise ValueError("session admission/position")
                    sessions[sid] = {"cache": make_prompt_cache(model), "position": 0, "prefill_done": False}
                state = sessions[sid]
                if state["position"] != item["position"] or not item["tokens"] or item["position"]+len(item["tokens"]) > init["max_context_tokens"]:
                    raise ValueError("step position/context")
                if any(type(t) is not int or not 0 <= t < vocab for t in item["tokens"]):
                    raise ValueError("invalid token")
                begin = time.perf_counter()
                cache = [v.to_kv_cache() if isinstance(v, TurboQuantKVCache) else v for v in state["cache"]]
                logits = model(mx.array([item["tokens"]], dtype=mx.int32), cache=cache)
                mx.eval(logits, *arrays(cache))
                next_token = int(mx.argmax(logits[0, -1]).item()) if item["prompt_complete"] else None
                first_decode = state["prefill_done"] and not state.get("decoded", False)
                if first_decode:
                    state["decoded"] = True
                before_bytes = sum(v.nbytes for v in arrays(cache))
                attention_layers = sum(isinstance(v, KVCache) for v in cache)
                recurrent_layers = len(cache)-attention_layers
                # Match Rapid's prefix-storage semantics: compress once, after prefill, then materialize on reuse.
                compressed = mode != "regular" and item["prompt_complete"] and not state["prefill_done"]
                if compressed:
                    cache = [TurboQuantKVCache.from_kv_cache(v, tq_config) if isinstance(v, KVCache) else v for v in cache]
                    mx.eval(*arrays(cache))
                state["position"] += len(item["tokens"])
                state["prefill_done"] |= item["prompt_complete"]
                state["cache"] = cache
                resident = sum(v.nbytes for v in arrays(cache))
                if resident > init["sequence_bytes"]:
                    raise MemoryError("backend state exceeded reserved sequence bytes")
                elapsed = time.perf_counter()-begin
                record = {"sequence": sid, "position": item["position"], "input_tokens": len(item["tokens"]), "native_next_token": next_token, "step_seconds": elapsed, "state_bytes": resident, "uncompressed_state_bytes": before_bytes, "prefix_compressed": compressed, "first_decode": first_decode, "attention_layers": attention_layers, "recurrent_layers": recurrent_layers, "mlx_active_bytes": mx.get_active_memory(), "mlx_peak_bytes": mx.get_peak_memory()}
                # Probe IO is excluded from step_seconds; Rust wall time includes it and labels the run accordingly.
                if probe_dir and first_decode:
                    path = probe_dir / f"sequence-{sid}-first-decode.npy"
                    mx.save(str(path), logits[0, -1].astype(mx.float32))
                    record["logits_probe"] = str(path)
                total_steps += 1
                if len(records) < 1024:
                    records.append(record)
                else:
                    diagnostic_records_omitted += 1
                outputs.append({"sequence": sid, "kv": [], "next_token": next_token, "state_bytes": resident})
            send({"outputs": outputs})
        elif request["op"] == "finish":
            send({"records": records, "total_steps": total_steps, "diagnostic_records_omitted": diagnostic_records_omitted, "active_sessions": len(sessions), "mlx_active_bytes": mx.get_active_memory(), "mlx_peak_bytes": mx.get_peak_memory(), "texts": [tokenizer.decode(tokens) for tokens in request["tokens"]]})
        else:
            raise ValueError("unknown RPC operation")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        send({"error": f"{type(exc).__name__}: {exc}"})
        raise
