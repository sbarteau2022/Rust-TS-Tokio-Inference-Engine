"""Real-model regular/TurboQuant A/B through the Rust scheduler. Run with Rapid's Python.
This is a small diagnostic/congruence benchmark, not broad model-quality qualification.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import time
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
MODES = ['regular', 'rapid-turboquant-v4', 'rapid-turboquant-k8v4']


def distribution(logits):
    x = logits.astype(np.float64)
    x -= x.max()
    probabilities = np.exp(x)
    return probabilities / probabilities.sum()


def compare(reference, candidate):
    comparisons = []
    matches = total = 0
    for left, right in zip(reference['generations'], candidate['generations'], strict=True):
        a, b = left['tokens'], right['tokens']
        matches += sum(x == y for x, y in zip(a, b))
        total += max(len(a), len(b))
        if not a or not b or a[0] != b[0]:
            raise ValueError('first resumed-decode contexts differ; cannot compare logits')
    left_records = {r['sequence']: r for r in reference['details']['records'] if r['first_decode']}
    right_records = {r['sequence']: r for r in candidate['details']['records'] if r['first_decode']}
    for sid, record in left_records.items():
        other = right_records[sid]
        a = np.load(record['logits_probe'], allow_pickle=False)
        b = np.load(other['logits_probe'], allow_pickle=False)
        if a.shape != b.shape or not np.isfinite(a).all() or not np.isfinite(b).all():
            raise ValueError('invalid logits probe')
        p, q = distribution(a), distribution(b)
        comparisons.append({'sequence': sid, 'rmse': float(np.sqrt(np.mean((a.astype(np.float64)-b)**2))), 'max_absolute_error': float(np.max(np.abs(a-b))), 'kl_reference_to_candidate': float(np.sum(p*np.log(np.maximum(p, 1e-300)/np.maximum(q, 1e-300)))), 'total_variation': float(np.abs(p-q).sum()/2), 'argmax_match': bool(a.argmax() == b.argmax())})
    return {'generated_token_position_match': matches/total, 'matched': matches, 'positions': total, 'first_decode_logit_comparisons': comparisons}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--config', default=ROOT/'fixtures/qwen-serving.json', type=Path)
    parser.add_argument('--prompts', default=ROOT/'fixtures/qwen-prompts.json', type=Path)
    parser.add_argument('--repetitions', type=int, default=3)
    parser.add_argument('--max-new', type=int, default=16)
    args = parser.parse_args()
    if not 1 <= args.repetitions <= 5 or not 2 <= args.max_new <= 32:
        parser.error('bounded diagnostic runs require repetitions 1..5, max-new 2..32')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    # Declared acceptance gates are intentionally stricter than a simple "it ran" result.
    gates = {'minimum_generated_token_match': .99, 'maximum_first_decode_kl': .01, 'minimum_wall_speed_ratio': .95, 'minimum_total_state_compression': 1.1}
    all_runs = {mode: [] for mode in MODES}
    for repetition in range(args.repetitions):
        order = MODES[repetition % len(MODES):] + MODES[:repetition % len(MODES)]
        for mode in order:
            stem = f'{repetition}-{mode}'
            command = [str(ROOT/'target/release/rust-server'), 'model-run', str(args.config), str(args.prompts), str(args.max_new), '--kv-codec', mode, '--probe-directory', str(out/(stem+'-probes'))]
            with (out/(stem+'.stderr')).open('w') as err:
                run = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=err, text=True, timeout=300)
            (out/(stem+'.json')).write_text(run.stdout)
            if run.returncode:
                raise RuntimeError(f'{mode} failed; see {stem}.stderr')
            data = json.loads(run.stdout)
            if not data['pass'] or data['stats']['memory_used'] or data['details']['active_sessions'] or data['stats']['failed']:
                raise ValueError('failed run or undrained state')
            all_runs[mode].append(data)
    summaries = {}
    regular = all_runs['regular']
    for mode, runs in all_runs.items():
        if any(run['readiness']['identity'] != regular[0]['readiness']['identity'] for run in runs):
            raise ValueError('model/backend identity changed within A/B')
        comparisons = [compare(a, b) for a, b in zip(regular, runs, strict=True)]
        wall = statistics.median(r['rust_boundary_seconds'] for r in runs)
        speed = statistics.median(r['rust_boundary_seconds'] for r in regular)/wall
        state_ratio = statistics.median(max(v['uncompressed_state_bytes']/v['state_bytes'] for v in r['details']['records']) for r in runs)
        kl = max(v['kl_reference_to_candidate'] for c in comparisons for v in c['first_decode_logit_comparisons'])
        match = min(c['generated_token_position_match'] for c in comparisons)
        summaries[mode] = {'median_rust_boundary_seconds': wall, 'wall_speed_ratio_vs_regular': speed, 'total_state_compression_at_prefill': state_ratio, 'minimum_generated_token_match': match, 'maximum_first_decode_kl': kl, 'comparisons': comparisons, 'passes_candidate_gates': mode!='regular' and match>=gates['minimum_generated_token_match'] and kl<=gates['maximum_first_decode_kl'] and speed>=gates['minimum_wall_speed_ratio'] and state_ratio>=gates['minimum_total_state_compression']}
    result = {'completed': True, 'scope': 'DIAGNOSTIC-AB; greedy; prefix-storage codec, not packed attention', 'production_qualified': False, 'full_integer_qwen_comparison_available': False, 'repetitions': args.repetitions, 'prompt_count': len(json.loads(args.prompts.read_text())), 'diagnostic_probe_io_in_rust_wall_time': True, 'acceptance_gates_declared_before_runs': gates, 'modes': summaries, 'default_remains': 'regular', 'default_reason': 'Candidates require broader held-out quality/SLO and sustained-load qualification even if diagnostic gates pass.', 'binary_sha256': hashlib.sha256((ROOT/'target/release/rust-server').read_bytes()).hexdigest(), 'worker_sha256': hashlib.sha256((ROOT/'backend/rapid_worker.py').read_bytes()).hexdigest(), 'config_sha256': hashlib.sha256(args.config.read_bytes()).hexdigest(), 'prompts_sha256': hashlib.sha256(args.prompts.read_bytes()).hexdigest(), 'unix_time': time.time()}
    (out/'summary.json').write_text(json.dumps(result, indent=2))
    print(json.dumps({k: v for k, v in result.items() if k!='modes'}))


if __name__ == '__main__':
    main()
