# guidance-vs-check — does an agent with Fux and the rule docs still break rules?

[W-102](../../work/open/W-102-guidance-vs-check-experiment.md)'s experiment, ready to run: it
decides whether the README's "rules it can't skip" line stays. The design is
[the positioning proposal](../../work/proposals/linter-for-ai-positioning.md) §5. Outside the engine
([L5](../../records/0007_LAW-5-no-model-in-the-engine.md)): the model is driven from here, never
from codelaw. Python standard library only ([L11](../../records/0013_LAW-11-dependencies.md)).

## Run it

```bash
python3 experiments/guidance-vs-check/harness.py plan                 # the matrix and the cost estimate; spends nothing
python3 experiments/guidance-vs-check/harness.py dry-run              # the scripted fake agent, end to end; spends nothing
python3 experiments/guidance-vs-check/harness.py run --agent claude   # the 180 real sessions — the decider's call
```

- **One command per stage.** `run` builds every repo, runs the agent, scores it and writes
  `runs/<agent>-<time>/RESULTS.md` and `results.json`. Run it again with the same `--out` and it
  resumes: a run with a `score.json` is not run twice. `table <out>` rewrites the table.
- **Narrow it** with `--arms A,C`, `--tasks t01,t13`, `--runs 1`, `--model`. The `codelaw` used is
  `$CARGO_TARGET_DIR` or `target/` (release, then debug), then `PATH`, or `--codelaw`.
- **`dry-run` proves the pipeline.** `fake_agent.py` writes each task's scripted solution — arm A
  the violating one every run, arm B on its first run only, C and D never, and one A run also
  turns a rule off in `codelaw.toml` — and the dry run fails unless every score is exactly the
  one that script implies. It also proves the fixtures: every violating solution breaks every
  rule its task targets, and every compliant one adds no error.

## What is in the folder

| path | what it is |
|---|---|
| `repo/` | the consumer repo every run starts from: a TS client (`web/`) and a Python API (`api/`), `js/recommended` + `py/recommended`, three custom rules (`.codelaw/rules/acme/`) and two configured `use-instead` rules |
| `tasks/tNN-*/` | the 20 tasks: `task.toml` (the prompt and the rule ids it tempts), `files/` (a seed, when the task has one), `fake/violating/` and `fake/compliant/` (the dry run's scripted solutions) |
| `arms/*.md` | what each arm appends to `AGENTS.md` |
| `experiment.toml` | the matrix, the model, the timeouts and the agent command |
| `harness.py` · `fake_agent.py` | the harness and the dry run's agent |

## The arms

Same model, same prompt — the task's text and nothing else — in every arm.

| arm | the agent has | how |
|---|---|---|
| **A** | `AGENTS.md` only (via `CLAUDE.md`'s `@AGENTS.md`) | no MCP server |
| **B** | A + the codelaw MCP server and the rule pages | `codelaw mcp`; `docs/codelaw-rules/` holds every `js/`, `py/` and `acme/` rule's page; two lines in `AGENTS.md` point at both |
| **C** | B + a Fux index | `fux setup --no-hooks`, `fux add .`, `fux ingest` in the repo; `fux mcp`; one more line in `AGENTS.md` |
| **D** *(not in the 180)* | A + `codelaw hook --pre` as a `PreToolUse` hook | the group `sync-agents` writes, from `templates/agents/claude/pre-hook-group.json` ([W-93](../../work/open/W-93-hook-pre-and-brief.md), on main since `4296fe9`); the harness refuses it on a codelaw built before that |

`hook` and `hook --pre` are **off** in A–C: the repo has no codelaw hook, and the agent CLI loads
project settings only, so none of the operator's own hooks leak in.

## Scoring

- **Deterministic.** `codelaw check --format json --no-adapters --no-cache --as-of <as_of>` runs on
  the prepared repo before the agent and on what it leaves after. A **new finding** is a rise in the
  count for one rule, file and severity. The same files always score the same.
- **A run broke a rule** when it has a new finding of a rule its task targets. **New errors**
  counts every rule at `error`, the number step 4 decides on.
- **Tampering does not help.** `codelaw.toml` and `.codelaw/` are restored before the after-check,
  and the run is marked `tampered`; inline `codelaw-ignore` comments it added are counted as
  `suppressions`.
- **Kept per run:** the prompt, the agent's JSON (tokens, turns, cost), `diff.patch`, the before-check's counts, the after-check's new findings
  and `score.json`. The repo itself is deleted after scoring unless `--keep-repos`, which keeps
  180 runs to a few MB ([L9](../../records/0011_LAW-9-machine-budget.md)).

## Cost

`harness.py plan` prints the estimate; its numbers are guesses for a Claude Code session on a
one-file task, not measurements — the first real runs replace them.

| | sessions | tokens per session (rough) | $ per session (`claude-opus-5-5`, $4 / $20 per M, cache reads $0.20, writes $5) |
|---|---|---|---|
| A | 60 | ~250k input, mostly cache reads; ~30k cache writes; ~6k output | ~$0.35 |
| B | 60 | ~400k input; ~45k cache writes; ~8k output | ~$0.51 |
| C | 60 | ~450k input; ~50k cache writes; ~9k output | ~$0.57 |
| **all** | **180** | | **~$86, a range of ~$45–$170**; ~3–6 min a session, so 9–18 hours run one at a time |

## Hazards

- **The agent CLI is the decider's choice.** `experiment.toml`'s `[agent.claude]` is the proposed
  command (`claude -p … --setting-sources project --strict-mcp-config`). The operator's
  `~/.claude/CLAUDE.md` may still be read; `--bare` avoids it but needs `ANTHROPIC_API_KEY`.
- **Fux in arm C** runs the ratified `fux-engine` (D-34) outside CI, a new scope for it.
- **Non-determinism:** rates are reported with their run counts, never from one run.
