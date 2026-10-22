"""Summarize measured retained-vs-recreated resources; verify reuse counters."""
import json
import statistics
from pathlib import Path

root = Path(__file__).resolve().parents[1]
summary = {}
for name in ["square", "rectangular", "elementwise"]:
    report = json.loads((root / f"evidence/resources-{name}.json").read_text())
    variants = {}
    for variant in ["persistent", "recreated"]:
        samples = report[variant][3:]
        assert len(samples) == 50
        variants[variant] = {
            "n": len(samples),
            "caller_wall_median_ms": statistics.median(x["caller_wall_ms"] for x in samples),
            "prepare_median_ms": statistics.median(x["metrics"]["prepare_ms"] for x in samples),
            "compile_median_ms": statistics.median(x["metrics"]["compile_ms"] for x in samples),
            "dispatch_wall_median_ms": statistics.median(x["metrics"]["dispatch_wall_ms"] for x in samples),
            "gpu_median_ms": statistics.median(x["metrics"]["gpu_ms"] for x in samples),
            "validation_median_ms": statistics.median(x["metrics"]["validation_ms"] for x in samples),
            "device_allocated_min_bytes": min(x["metrics"]["resources"]["device_current_allocated_bytes"] for x in samples),
            "device_allocated_max_bytes": max(x["metrics"]["resources"]["device_current_allocated_bytes"] for x in samples),
            "final_resources": samples[-1]["metrics"]["resources"],
        }
    warm = report["persistent"][3:]
    assert all(x["metrics"]["pipeline_cache_hit"] for x in warm)
    assert all(x["metrics"]["compile_ms"] == 0 for x in warm)
    assert len({x["metrics"]["resources"]["buffer_allocations"] for x in warm}) == 1
    assert variants["persistent"]["final_resources"]["pipeline_hits"] == 52
    summary[name] = {"shape_mkn": report["shape_mkn"], "kernel": report["kernel"],
                     "variants": variants,
                     "recreated_over_persistent_wall": variants["recreated"]["caller_wall_median_ms"] / variants["persistent"]["caller_wall_median_ms"],
                     "scope": report["scope"]}
print(json.dumps({"pass": True, "results": summary}, indent=2))
