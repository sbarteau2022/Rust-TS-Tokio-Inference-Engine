#!/usr/bin/env python3
"""Serial discovery-only sweep; immutable output directory and explicit argv ledger."""
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'judges'))
from native_dspark_report import validate, FROZEN

CONFIGS = [('w2-none', 2, 'none', False), ('w4-none', 4, 'none', False),
           ('w8-none', 8, 'none', False), ('w4-c90', 4, '0.9', False),
           ('w4-c95', 4, '0.95', False), ('w8-c90', 8, '0.9', False),
           ('w8-c95', 8, '0.95', False), ('adaptive8-c90', 8, '0.9', True),
           ('adaptive8-c95', 8, '0.95', True)]


def main():
    if len(sys.argv) != 5:
        raise SystemExit('run_dspark_tuning.py BINARY PLAN DRAFTER NEW_DIRECTORY')
    binary, plan, drafter, directory = map(lambda value: Path(value).resolve(), sys.argv[1:])
    if not all(path.is_file() for path in [binary, plan, drafter]):
        raise SystemExit('binary/plan/drafter must exist')
    directory.mkdir(parents=True, exist_ok=False)
    frozen = json.loads(FROZEN.read_text())
    summaries, commands = [], []
    for name, width, cutoff, adaptive in CONFIGS:
        report = directory / (name + '.json')
        command = [str(binary), str(plan), str(drafter), str(report), 'bf16',
                   f'--width={width}', f'--confidence={cutoff}']
        if adaptive:
            command.append('--adaptive')
        commands.append(command)
        (directory / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n')
        with (directory / (name + '.log')).open('x') as log:
            subprocess.run(command, cwd=ROOT, stdout=log, stderr=log,
                           check=True, timeout=180)
        raw = json.loads(report.read_text())
        summary = validate(raw, frozen)
        summary.update(name=name, tuning=raw['tuning'],
                       proposal_seconds=raw['proposal_seconds'])
        summaries.append(summary)
        (directory / 'summary.json').write_text(json.dumps(summaries, indent=2) + '\n')
        print(name, flush=True)


if __name__ == '__main__':
    main()
