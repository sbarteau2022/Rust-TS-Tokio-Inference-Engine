"""Run ordered prototype judges and retain machine-readable evidence. Fail fast."""
import hashlib
import json
import os
import select
from pathlib import Path
import subprocess
import tempfile
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "evidence"
ENV = dict(os.environ, RUST_SERVER_IN_FLIGHT='2', PATH=f"/opt/homebrew/bin:{Path.home()}/.cargo/bin:{os.environ.get('PATH', '')}")


def run(args, output, *, env=ENV):
    with (EVIDENCE / output).open("w") as out, (EVIDENCE / (output + ".stderr")).open("w") as err:
        child = subprocess.Popen(args, cwd=ROOT, env=env, stdout=out, stderr=err)
        try:
            code = child.wait(timeout=300)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()
            raise
        if code:
            raise RuntimeError(f"judge failed ({code}): {args}; see evidence/{output}")
        return child.pid


def load(name):
    return json.loads((EVIDENCE / name).read_text())


def main():
    EVIDENCE.mkdir(exist_ok=True)
    versions = {}
    for name, command in [("cargo", ["cargo", "--version"]), ("rustc", ["rustc", "--version"]),
                          ("node", ["node", "--version"]), ("npm", ["npm", "--version"]),
                          ("flatc", ["flatc", "--version"]), ("tsrs", ["npx", "--no-install", "tsrs", "--version"]),
                          ("typescript", ["npx", "--no-install", "tsc", "--version"])]:
        versions[name] = subprocess.check_output(command, cwd=ROOT, env=ENV, text=True).strip()
    assert "7.1." in versions["typescript"] and "ts7.1" in json.loads((ROOT / "node_modules/tsrs/package.json").read_text())["version"]
    (EVIDENCE / "toolchain.json").write_text(json.dumps(versions, indent=2))
    run([sys.executable,"judges/test_speculative_report.py"],"speculative-report-tests.log")
    run([sys.executable,"judges/test_device_smoke_report.py"],"device-smoke-report-tests.log")
    run([sys.executable,"judges/test_attention_profile.py"],"attention-profile-tests.log")
    run([sys.executable,"judges/test_layer_reference.py"],"layer-reference-tests.log")
    run([sys.executable,"judges/test_suite_reference.py"],"suite-reference-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_discriminators"], "native-discriminator-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_suite_reference"], "native-suite-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_device_reference"], "native-device-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_device_pressure"], "native-device-pressure-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_speculative_smoke"], "native-speculative-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_automaton_smoke"], "native-automaton-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_hidden_smoke"], "native-hidden-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_smoke"], "native-mtp-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_decode_smoke"], "native-mtp-decode-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_bench"], "native-mtp-bench-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_automaton_smoke"], "native-mtp-automaton-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_scheduler_smoke"], "native-mtp-scheduler-example-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_mtp_scheduler_bench"], "native-mtp-scheduler-bench-tests.log")
    run(["cargo", "test", "--locked", "--example", "native_text_stream_smoke"], "native-text-stream-example-tests.log")
    run(["c++", "-std=c++20", "-O2", "-Ivendor/mlx/include", "judges/dense_precision.cpp", "-Lvendor/mlx/lib", "-lmlx", "-Wl,-rpath,"+str(ROOT/"vendor/mlx/lib"), "-o", "artifacts/dense-precision-judge"], "dense-precision-build.log")
    run(["artifacts/dense-precision-judge"], "dense-precision-default.log")
    run(["artifacts/dense-precision-judge"], "dense-precision-disabled.log",env=dict(ENV,MLX_ENABLE_TF32="0"))

    run([sys.executable,"judges/generate_cordic.py","--check"],"cordic-table-check.log")
    # Step 0: deterministic bindings and both requested compiler gates.
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        for schema in ["trace", "stream"]:
            subprocess.run(["flatc", "--rust", "-o", str(tmp), f"schema/{schema}.fbs"], cwd=ROOT, env=ENV, check=True)
            subprocess.run(["rustfmt", "--edition", "2021", str(tmp / f"{schema}_generated.rs")], env=ENV, check=True)
            assert (tmp / f"{schema}_generated.rs").read_bytes() == (ROOT / f"src/{schema}_generated.rs").read_bytes()
            subprocess.run(["flatc", "--ts", "-o", str(tmp / "ts"), f"schema/{schema}.fbs"], cwd=ROOT, env=ENV, check=True)
        generated = list((tmp / "ts").rglob("*.ts"))
        assert len(generated) == len(list((ROOT / "ts/generated").rglob("*.ts")))
        for path in generated:
            assert path.read_bytes() == (ROOT / "ts/generated" / path.relative_to(tmp / "ts")).read_bytes()
    run(["c++", "-std=c++20", "-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-Inative", "judges/projection_reference.cpp", "-o", "artifacts/projection-reference-judge"], "projection-reference-build.log")
    run(["artifacts/projection-reference-judge"], "projection-reference-tests.log")
    run(["c++", "-std=c++20", "-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-Inative", "judges/operator_reference.cpp", "-o", "artifacts/operator-reference-judge"], "operator-reference-build.log")
    run(["artifacts/operator-reference-judge"], "operator-reference-tests.log")
    run(["c++", "-std=c++20", "-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-Inative", "judges/state_table.cpp", "-o", "artifacts/state-table-judge"], "state-table-build.log")
    run(["artifacts/state-table-judge"], "state-table-tests.log")
    run(["c++", "-std=c++20", "-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-Inative", "-Ivendor/mlx/include", "native/mlx_bridge.cpp", "judges/device_abi.cpp", "-Lvendor/mlx/lib", "-lmlx", "-Wl,-rpath,"+str(ROOT/"vendor/mlx/lib"), "-o", "artifacts/device-abi-judge"], "device-abi-build.log")
    run(["artifacts/device-abi-judge"], "device-abi-tests.log")
    run(["cargo", "build", "--locked"], "step0-cargo.log")
    run(["npx", "--no-install", "tsrs", "-p", "tsconfig.json"], "step0-tsrs.log")
    run(["npx", "--no-install", "tsc", "--noEmit", "-p", "tsconfig.json"], "step0-ts71.log")
    idle = subprocess.Popen(["target/debug/rust-server", "idle"], cwd=ROOT, env=ENV, stdout=subprocess.PIPE, text=True)
    try:
        assert select.select([idle.stdout], [], [], 5)[0], "idle startup timeout"
        assert idle.stdout.readline().strip() == "ready" and idle.poll() is None
    finally:
        idle.terminate()
        idle.wait(timeout=5)
    (EVIDENCE / "step0-idle.log").write_text("PASS: starts, idles, exits on SIGTERM\n")
    # Step 1 must pass before running GPU judges.
    run(["npx", "--no-install", "tsx", "ts/trace.ts"], "step1-ts-ops.json")
    run(["target/debug/rust-server", "decode", "artifacts/trace.amvk"], "step1-rust-ops.json")
    assert load("step1-ts-ops.json") == load("step1-rust-ops.json")
    assert (ROOT / "artifacts/trace.amvk").read_bytes() == (ROOT / "fixtures/square.amvk").read_bytes()
    (EVIDENCE / "step1-roundtrip.log").write_text("PASS: seven ordered operations identical\n")
    run(["cargo", "build", "--release", "--locked"], "release-build.log")
    run(["target/release/rust-server", "stream-check", "fixtures/stream-v1.amst"], "stream-wire-rust-fixture.log")
    with tempfile.TemporaryDirectory() as wire_tmp:
        ts_path = str(Path(wire_tmp) / "typescript.amst")
        run(["npx", "--no-install", "tsx", "ts/stream-inspect.ts", "--fixture", ts_path], "stream-wire-ts-fixture.log")
        run(["target/release/rust-server", "stream-check", ts_path], "stream-wire-cross-rust.log")
    run(["npx", "--no-install", "tsx", "ts/stream-inspect.ts", "fixtures/stream-v1.amst"], "stream-wire-cross-ts.log")

    binary = "target/release/rust-server"
    run([binary, "uma-judge"], "step2-uma.json")
    assert load("step2-uma.json")["pass"]
    # Step 3: square and all three tail dimensions; 3 warmup + 50 measured dispatches.
    run([binary, "kernel-judge", "artifacts"], "step5-square-rust.json")
    run(["npx", "--no-install", "tsx", "ts/trace.ts", "artifacts/rectangular", "73", "29", "91"], "step3-rectangular-ts.json")
    assert (ROOT / "artifacts/rectangular/trace.amvk").read_bytes() == (ROOT / "fixtures/rectangular.amvk").read_bytes()
    run([binary, "kernel-judge", "artifacts/rectangular"], "step5-rectangular-rust.json")
    run(["npx", "--no-install", "tsx", "ts/trace.ts", "artifacts/elementwise", "73", "1", "91", "add-relu"], "elementwise-ts-ops.json")
    run([binary, "decode", "artifacts/elementwise/trace.amvk"], "elementwise-rust-ops.json")
    assert load("elementwise-ts-ops.json") == load("elementwise-rust-ops.json")
    assert (ROOT / "artifacts/elementwise/trace.amvk").read_bytes() == (ROOT / "fixtures/elementwise.amvk").read_bytes()
    run([binary, "kernel-judge", "artifacts/elementwise"], "elementwise-rust.json")
    for function in ["sin","cos","atan2","tanh"]:
        directory=f"artifacts/{function}"
        run(["npx","--no-install","tsx","ts/trace.ts",directory,"73","1","91",function],f"cordic-{function}-ts.json")
        run([binary,"decode",f"{directory}/trace.amvk"],f"cordic-{function}-rust-ops.json")
        assert load(f"cordic-{function}-ts.json")==load(f"cordic-{function}-rust-ops.json")
        assert (ROOT/f"{directory}/trace.amvk").read_bytes()==(ROOT/f"fixtures/{function}.amvk").read_bytes()
        run([binary,"kernel-judge",directory],f"cordic-{function}.json")
        run([binary,"run",f"{directory}/trace.amvk",f"{directory}/inputs.json"],f"cordic-{function}-receipt.json")
        assert load(f"cordic-{function}-receipt.json")["failure"] is None
    run([sys.executable,"judges/cordic.py"],"cordic-accuracy.json")
    # Step 4: verify receipts AND real native log records from this exact process.
    pid = run([binary, "runtime-judge"], "step4-receipts.jsonl")
    receipts = [json.loads(line) for line in (EVIDENCE / "step4-receipts.jsonl").read_text().splitlines()]
    assert len(receipts) == 24 and len({r["job_id"] for r in receipts}) == 24
    assert all(r["failure"] is None and r["returned_before_completion"] and r["signpost_enabled"] for r in receipts)
    for attempt in range(3):
        run(["/usr/bin/log", "show", "--last", "5m", "--style", "json", "--signpost", "--predicate", 'subsystem == "engineering.atoma.rust-server"'], "step4-os-log.json")
        logs = [r for r in load("step4-os-log.json") if r.get("processID") == pid]
        pairs = {(int(r.get("signpostID", 0)), r.get("signpostType")) for r in logs if r.get("signpostName") == "Kernel Job"}
        if all((r["signpost_id"], kind) in pairs for r in receipts for kind in ("begin", "end")):
            break
        time.sleep(1)
    else:
        raise RuntimeError("missing native signpost records")
    (EVIDENCE / "step4-os-log.json").write_text(json.dumps(logs, indent=2))
    (EVIDENCE / "step4-summary.json").write_text(json.dumps({"jobs": 24, "successful": 24, "native_signpost_pairs": 24, "process_id": pid, "all_submissions_returned_before_completion": True}, indent=2))
    flight=json.loads((ROOT/"artifacts/runtime-flight.json").read_text())
    assert flight["limit"]==2 and flight["completed"]==flight["submitted"]==24 and flight["active"]==0
    assert flight["peak"]<=2 and flight["peak_outstanding"]<=2 and flight["acquired"]==flight["released"]
    (EVIDENCE/"runtime-flight-2.json").write_text(json.dumps(flight,indent=2))
    run([binary,"runtime-judge"],"runtime-limit3.jsonl",env=dict(ENV,RUST_SERVER_IN_FLIGHT="3"))
    flight3=json.loads((ROOT/"artifacts/runtime-flight.json").read_text())
    assert flight3["limit"]==3 and flight3["completed"]==flight3["submitted"]==24 and flight3["active"]==0
    assert flight3["peak"]<=3 and flight3["peak_outstanding"]<=3 and flight3["acquired"]==flight3["released"]
    (EVIDENCE/"runtime-flight-3.json").write_text(json.dumps(flight3,indent=2))
    assert "wait_until_completed" not in (ROOT/"src/gpu.rs").read_text()
    # Step 5: raw integers checked by independent arbitrary-precision oracle.
    for shape, directory in [("square", "artifacts"), ("rectangular", "artifacts/rectangular")]:
        run([sys.executable, "judges/benchmark.py", directory, f"evidence/step5-{shape}-rust.json"], f"step5-{shape}-comparison.json")
    run([sys.executable, "judges/benchmark.py", "artifacts/elementwise", "evidence/elementwise-rust.json"], "elementwise-integer-comparison.json")
    run([binary,"capabilities"],"capabilities.json")
    for name,directory in [("square","artifacts"),("rectangular","artifacts/rectangular"),("elementwise","artifacts/elementwise")]:
        run([binary,"backend-judge",directory],f"resources-{name}.json")
    run([sys.executable,"judges/resources.py"],"resources-summary.json")
    # Complete adversarial suite and quality checks after the step ladder.
    run(["cargo", "test", "--locked", "--", "--nocapture"], "adversarial-tests.log")
    run(["cargo", "test", "--locked", "--features", "debug-validation", "--", "--nocapture"], "debug-validation-tests.log")
    run(["npm", "test"], "ts-tests.log")
    run(["cargo", "clippy", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"], "clippy.log")
    run(["cargo", "fmt", "--check"], "format.log")
    run(["npx", "--no-install", "tsrs", "-p", "tsconfig.json"], "final-tsrs.log")
    run(["npx", "--no-install", "tsc", "--noEmit", "-p", "tsconfig.json"], "final-ts71.log")
    run([binary, "run", "artifacts/trace.amvk", "artifacts/inputs.json"], "cli-receipt.json")
    receipt = load("cli-receipt.json")
    assert receipt["failure"] is None and len(receipt["output"]["values"]) == 128*128
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        bad = bytearray((ROOT / "fixtures/square.amvk").read_bytes())
        bad[4] = 0
        (tmp / "bad.amvk").write_bytes(bad)
        (tmp / "empty-inputs.json").write_text("{}")
        (tmp / "oversized.amvk").write_bytes(bytes(65537))
        checks = []
        for label, args, expected_stage in [
            ("malformed_trace", [binary,"run",str(tmp / "bad.amvk"),"artifacts/inputs.json"], "decode"),
            ("missing_inputs", [binary,"run","artifacts/trace.amvk",str(tmp / "empty-inputs.json")], "inputs"),
            ("oversized_trace", [binary,"decode",str(tmp / "oversized.amvk")], None),
        ]:
            result = subprocess.run(args, cwd=ROOT, env=ENV, text=True, capture_output=True, timeout=30)
            assert result.returncode != 0, label
            if expected_stage:
                assert json.loads(result.stdout)["failure"]["stage"] == expected_stage
            checks.append({"case":label,"returncode":result.returncode,"stage":expected_stage})
        for value in ["0","1","4","invalid"]:
            result=subprocess.run([binary,"run","artifacts/trace.amvk","artifacts/inputs.json"],cwd=ROOT,env=dict(ENV,RUST_SERVER_IN_FLIGHT=value),text=True,capture_output=True,timeout=30)
            assert result.returncode!=0
            assert "in-flight limit" in result.stderr or "RUST_SERVER_IN_FLIGHT" in result.stderr
            checks.append({"case":"in_flight_limit","value":value,"returncode":result.returncode})
        (EVIDENCE / "cli-rejections.json").write_text(json.dumps(checks, indent=2))
    run([binary,"run-chain","artifacts/rectangular/inputs.json","artifacts/rectangular/trace.amvk","artifacts/sin/trace.amvk","artifacts/cos/trace.amvk","artifacts/tanh/trace.amvk"],"chain-receipt.json",env=dict(ENV,RUST_SERVER_IN_FLIGHT="3"))
    chain=load("chain-receipt.json")
    assert chain["failure"] is None and chain["kernel"]=="chain"
    assert chain["output"]["metrics"]["kernel_dispatches"]==4
    assert chain["output"]["metrics"]["validation_performed"] is False
    assert chain["output"]["metrics"]["max_abs_error"] is None
    assert chain["output"]["metrics"]["validation_ms"]==0
    assert chain["flight"]["limit"]==3 and chain["flight"]["completed"]==chain["flight"]["submitted"]==1
    # Native backend has no interpreter/JSON transport. Legacy adapter is opt-in only.
    run([binary,"native-mlx"],"native-mlx-smoke.txt")
    for command in [["native-device-smoke"],["native-device-smoke","fixtures"],["native-device-smoke","unused","extra"],["model-run","fixtures/qwen-serving.json","fixtures/qwen-prompts.json","1"],["config","check","--config","fixtures/qwen-serving.json"],["native-mlx","--unknown"],["native-layer-reference"],["native-layer-reference","fixtures"],["native-layer-reference","unused","extra"]]:
        rejected=subprocess.run([binary,*command],cwd=ROOT,env=ENV,capture_output=True,timeout=10)
        assert rejected.returncode!=0, command
    run([binary,"cache-judge","fixtures/cache-config-bench.json"],"cache-judge.json")
    cache=load("cache-judge.json")
    assert cache["pass"] and cache["production_qualified"] is False
    assert cache["payload_source"]=="SYNTHETIC-NONMODEL" and cache["policy"]=="DETERMINISTIC-MODE"
    assert cache["audit"]["declared_tokens"]==cache["audit"]["resident_declared_tokens"]==32
    assert cache["validation"]["passed"]==cache["validation"]["scheduled"]==1
    assert cache["cache"]["active_leases"]==0
    assert cache["receipt"]["validation_policy"]=="DETERMINISTIC-MODE"
    capture_path = ROOT / "artifacts" / f"judge-{time.time_ns()}.gputrace"
    run([binary, "kernel-judge", "artifacts", str(capture_path)], "capture-judge.json", env=dict(ENV, METAL_CAPTURE_ENABLED="1"))
    assert capture_path.is_dir() and any(capture_path.iterdir())
    (EVIDENCE / "capture-path.txt").write_text(str(capture_path) + "\n")
    source = [p for directory in ["src", "ts", "native", "schema", "tests", "judges", "fixtures", "backend", "examples"]
              for p in (ROOT / directory).rglob("*") if p.is_file() and "__pycache__" not in str(p)]
    source += [ROOT / p for p in ["Cargo.toml", "Cargo.lock", "build.rs", "package.json", "package-lock.json", "tsconfig.json", "run.sh", "requirements-bench.txt"]]
    hashes = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source)}
    (EVIDENCE / "source-sha256.json").write_text(json.dumps(hashes, indent=2))
    (EVIDENCE / "judge-summary.json").write_text(json.dumps({"pass": True, "arithmetic_contract": "q1_31_i32_v3", "cordic_functions": ["sin","cos","atan2","tanh"], "async_completion_limits": [2,3], "default_in_flight": 3, "sample_interval":1000, "sampling_policy":"DETERMINISTIC-MODE", "production_sampling_claims":False, "spec15_cache_control":"opaque payload adapter; nonproduction", "model_serving_foundation":"native Rust MLX tensor executor with joint completion ledger; native hybrid forward implemented; strict held-out model gate failed; Python adapter disabled by default", "chain_max_stages":16, "validation_policy":"production sampled; full CPU reference test/debug-validation feature", "resource_backend_stages": [1,2], "steps": list(range(6)), "native_capture": str(capture_path), "source_manifest": "source-sha256.json", "unix_time": time.time()}, indent=2))
    print("PASS: steps 0–5, persistent resources, six kernel families including CORDIC, adversarial suites, compiler gates, CLI, native signposts and GPU capture")


if __name__ == "__main__":
    main()
