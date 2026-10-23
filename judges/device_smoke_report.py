"""Derive per-lane decode rates from saved Rust scheduler publication timestamps.
Offline evidence only; never imported by serving or used to select model output.
"""
import json
import math
from pathlib import Path
import sys

def percentile(values, q):
    if not values or any(not math.isfinite(v) or v < 0 for v in values):
        raise ValueError("invalid samples")
    return sorted(values)[max(0, math.ceil(len(values)*q)-1)]

def report(directory):
    summary = json.loads((directory / "summary.json").read_text())
    rows = [json.loads(line) for line in (directory / "requests.jsonl").read_text().splitlines()]
    if len(rows) != 24 or len(summary["windows"]) != 4:
        raise ValueError("incomplete four-window smoke")
    for window in summary["windows"]:
        selected = [r for r in rows if r["mode"] == window["mode"] and r["concurrency"] == window["concurrency"]]
        if sorted(r["id"] for r in selected) != list(range(6)):
            raise ValueError("missing/duplicate request")
        speeds = []
        for row in selected:
            g = row["result"]
            times = g["token_times_ms"]
            if len(times) != len(g["tokens"]) or len(times) < 2 or any(not math.isfinite(t) or t < 0 for t in times) or any(b < a for a,b in zip(times,times[1:])) or times[-1] <= times[0]:
                raise ValueError("invalid token timing")
            speeds.append((len(times)-1)*1000/(times[-1]-times[0]))
        window.update(per_lane_decode_tps_p50=percentile(speeds,.5), per_lane_decode_tps_p95=percentile(speeds,.95),
                      admitted_requests=6, completed_requests=6, request_admission_percent=100.,
                      speculative_acceptance_percent=None, speculation_enabled=False,
                      continuous_iteration_scheduling=True, tensor_batching=False,
                      native_rows_serial=True, network_token_streaming_measured=False)
    return summary

if __name__ == "__main__":
    print(json.dumps(report(Path(sys.argv[1])), indent=2))
