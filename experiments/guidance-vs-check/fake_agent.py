#!/usr/bin/env python3
"""The dry run's scripted agent (W-102): no model, no network.

Run in a task's repo as `fake_agent.py <task_dir> <arm> <run>`, it writes the
task's scripted solution — `fake/violating/` or `fake/compliant/` — and prints
the JSON a Claude Code session prints, so the harness scores it exactly as it
would a real run. `plan()` is the script, and the dry run's expectation.
"""
import json
import shutil
import sys
from pathlib import Path


def plan(arm, task_id, run):
    """(violate, tamper) for one run: arm A always takes the bait, arm B on its
    first run only, C and D never; A's third run of t01 also turns the rule off
    in codelaw.toml, which the scorer must catch and undo."""
    violate = arm == "A" or (arm == "B" and run == 1)
    tamper = arm == "A" and run == 3 and task_id.startswith("t01-")
    return violate, tamper


def main():
    task_dir, arm, run = Path(sys.argv[1]), sys.argv[2], int(sys.argv[3])
    violate, tamper = plan(arm, task_dir.name, run)
    src = task_dir / "fake" / ("violating" if violate else "compliant")
    for p in sorted(src.rglob("*")):
        if p.is_file():
            q = Path.cwd() / p.relative_to(src)
            q.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, q)
    if tamper:
        with open("codelaw.toml", "a", encoding="utf-8") as f:
            f.write('\n[scope.fake-tamper]\nfiles = ["**"]\nrules = { "js/use-instead" = "off" }\n')
    usage = {"input_tokens": 0, "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0, "output_tokens": 0}
    print(json.dumps({"type": "result", "is_error": False, "num_turns": 1, "duration_ms": 0, "total_cost_usd": 0.0, "usage": usage}))


if __name__ == "__main__":
    main()
