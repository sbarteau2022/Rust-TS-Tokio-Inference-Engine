"""Independent arbitrary-precision fixed-point oracle; GPU timing is Rust only."""
import argparse
import json
import statistics
from pathlib import Path


def stats(samples):
    ordered = sorted(samples)
    return {"n": len(samples), "median_ms": statistics.median(samples),
            "p95_ms": ordered[int((len(ordered)-1)*0.95)], "min_ms": min(samples)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    parser.add_argument("rust_report", type=Path)
    args = parser.parse_args()
    d = args.directory
    plan = json.loads((d / "plan.json").read_text())
    inputs = json.loads((d / "inputs.json").read_text())
    actual = json.loads((d / "result.json").read_text())["values"]
    report = json.loads(args.rust_report.read_text())
    m, k, n = plan["m"], plan["k"], plan["n"]
    names = plan["inputs"]
    a, bias = inputs[names[0]], inputs[names[-1]]
    assert all(type(x) is int and -(1 << 31) <= x < (1 << 31)
               for values in inputs.values() for x in values)
    expected = []
    for row in range(m):
        for col in range(n):
            if plan["kind"] == "gemm_bias_relu":
                b = inputs[names[1]]
                value = sum((a[row*k+x]*b[x*n+col]) // (1 << 31) for x in range(k))
            else:
                value = a[row*n+col]
            expected.append(min(max(value+bias[col], 0), (1 << 31)-1))
    assert actual == expected
    samples = report["samples_ms"][3:]
    assert len(samples) == 50
    print(json.dumps({"pass": True, "shape_mkn": [m,k,n], "kernel_family": plan["kind"],
                      "dtype": "q1_31_i32", "fractional_bits": 31,
                      "oracle": "Python arbitrary-precision integers; per-product floor division",
                      "exact_matches": len(expected), "max_raw_error": 0,
                      "rust_dispatch_wall": stats([x["dispatch_wall_ms"] for x in samples]),
                      "rust_gpu": stats([x["gpu_ms"] for x in samples]),
                      "scope": "Integer correctness and local dispatch timing only; no MLX float equivalence or throughput claim."}, indent=2))


if __name__ == "__main__":
    main()
