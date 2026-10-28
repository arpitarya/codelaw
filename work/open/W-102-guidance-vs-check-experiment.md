---
type: Handoff
name: W-102
description: "Arpit, 2026-10-27 (W-36 R4): measure whether an agent with Fux and the rule docs still breaks rules — the evidence for or against the README's 'you need a check' line."
item: W-102
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-102 — Experiment — does good guidance make the check unnecessary?

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | 20 small tasks in a fixture repo with `js/recommended`, `py/recommended`, three custom rules and two configured `use-instead` rules | a fair task set | ✅ built — `experiments/guidance-vs-check/repo/` and `tasks/t01`–`t20` |
| 2 | Three arms, same model and prompt, 3 runs each (180 runs): (A) AGENTS.md only; (B) + codelaw MCP and the rule pages; (C) + a Fux index | comparable runs | ✅ built; approved 2026-10-27; ✅ **pilot run 2026-10-28** (60 sessions, $10.47); ⏳ the 180 — the decider's go-ahead (inbox) |
| 3 | Measure `codelaw check` on each diff (new findings by rule), with `hook` and `hook --pre` off; record tokens and turns | the data | ✅ scorer and table built, proven by `harness.py dry-run` (180/180 again after the pilot's fixes); the pilot's table: [`results/2026-10-w102-pilot/`](../../experiments/guidance-vs-check/results/2026-10-w102-pilot/RESULTS.md) |
| 4 | Decide: arm C gives zero new `error`s in all 60 runs → the README's "rules it can't skip" line goes; otherwise it stays with the measured rate | a README line backed by data | ⏳ |

**Arpit, 2026-10-29 (voice docket; written on *"file it."*):** recommended redesigning before the full 180: *"Let's post a research document on how do you want to design it again. With the downsides and details on why the current one failed, or the previous one failed, and what needs to be improved in next use case. Or what is something that you need to figure out in the next use case. So probably create a research work item."* — [W-118](W-118-guidance-experiment-redesign.md); the 180 waits on it.

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"This is in my local, right?"* — yes: on the Mac, from the main checkout, with his Claude Code login, one session at a time (9–18 hours in all; nothing in CI) — then *"Approved."*: (1) the CLI is Claude Code 2.1, `claude -p … --setting-sources project --strict-mcp-config` (`experiment.toml` `[agent.claude]`); (2) `fux` (D-34) runs in arm C outside CI; (3) run it — the pilot first, `harness.py run --agent claude --runs 1 --out experiments/guidance-vs-check/runs/2026-10-w102` (60 sessions, ~$29), then the same `--out` without `--runs` for the full 180 (~$86).

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Claude Code (Opus) on the Mac.

Design: [proposal](../proposals/linter-for-ai-positioning.md) §5. Needs a real agent — a Claude Code session on the Mac.

## Pilot (2026-10-28, Claude Code)

`harness.py run --agent claude --runs 1 --out experiments/guidance-vs-check/runs/2026-10-w102`, Claude Code
2.1.310, `claude-opus-5-5`, codelaw 0.2.0 (a release build pinned in the run folder's `bin/`). The table:
[`results/2026-10-w102-pilot/RESULTS.md`](../../experiments/guidance-vs-check/results/2026-10-w102-pilot/RESULTS.md).

| arm | runs | broke a targeted rule | any new error | turns (mean) | tokens in / out (mean) | cost (mean · total) |
|---|---|---|---|---|---|---|
| A | 20 | 2/20 | 2/20 | 4.3 | 82.0k / 1.1k | $0.11 · $2.21 |
| B | 20 | 2/20 | 2/20 | 9.3 | 153.8k / 1.9k | $0.20 · $3.96 |
| C | 20 | 2/20 | 2/20 | 8.6 | 161.6k / 1.9k | $0.21 · $4.30 |

- **Cost: $10.47 against the ~$29 estimate (0.36×); 26 minutes, not 3–6 hours.** Re-estimated from the
  pilot: the full 180 ~$31 and ~80 min; resumed on the pilot's folder (its r1 kept), the 120 left ~$21, ~55 min.
- **Both breaks are in every arm**, on the same tasks: t02 (debug logging — `console.log`) and t04 (parameters
  kept unused in a signature). In t02 and C t04 the agent said the check would fail; A t04 edited `codelaw.toml`
  instead (restored, scored broken). The other 18 tasks tempted no arm: at one run a cell the arms do not separate.
- **Every arm can run the check.** `codelaw` is on every run's `PATH` and `AGENTS.md` names `codelaw check`;
  the agents ran it. Arm A is "guidance + a check it may choose to run", which the design should name or remove.

**What the pipeline got wrong, and the fixes (in `harness.py`, landed with the pilot):**

1. **A relative `--out` found no Python pack** — `CODELAW_SITE_PACKAGES` was relative and every check runs inside
   the repo. `--out` is resolved now.
2. **Inside `runs/` the check read no file.** `runs/` is git-ignored in this checkout, and `codelaw check` applies
   an enclosing repository's ignore files to a project that is its own git repository (`ancestor_ignores` in
   `crates/codelaw-engine/src/discover.rs` starts above the root, `ancestors().skip(1)`, so the root's own `.git`
   never stops it — git does not do this). Every run scored clean, and the agent would have read codelaw's own
   `CLAUDE.md`. Caught after 5 sessions (~$1), which were dropped. The repos are now built in `--work`, outside
   every repository (default `<tmp>/codelaw-w102/<out's name>`), refused under a `.git`, `CLAUDE.md`, `AGENTS.md`
   or `codelaw.toml`; a prepared repo whose check reads no file stops the run. The engine behaviour is not fixed
   here (not this item's) — it is worth an item of its own.
3. **An agent error scored clean.** Claude Code 2.1.207 answers `claude-opus-5-5` with an API error (2.1.280+
   needed); the session scored clean. Agent errors are now counted apart and kept out of every rate. The pilot
   used the VS Code extension's 2.1.310, linked as the run folder's `bin/claude` (first on every run's `PATH`).
4. **Every run read `tampered`** — the agent's own `codelaw check` writes `.codelaw/cache/`. The scorer ignores the
   cache now, after the pilot; the pilot's `tampered` column is wrong (findings unaffected).

## Built (2026-10-27, Claude Code)

The experiment is one command, under [`experiments/guidance-vs-check/`](../../experiments/guidance-vs-check/README.md)
(owned by this item in the register's Ownership table until a decision owns it; outside the engine, L5;
standard library only, L11):

- **The repo and 20 tasks.** A TS client and a Python API; `js/recommended` + `py/recommended`; three custom
  rules (`acme/no-datetime-now`, `acme/no-sql-fstring`, `acme/no-inline-style`, each passing `codelaw test`)
  and `js/use-instead` (`fetch` → `apiFetch`) and `py/use-instead` (`requests.get`/`post` → `http_get`/`http_post`),
  all at `error`. Each task is a prompt that tempts one or two named rules (18 rule ids across the 20), a seed
  where the temptation is "follow the existing style", and a scripted violating and compliant solution.
- **The arms** as this item defines them, C cumulative on B ("the best guidance"); the prompt is the task's text
  alone in every arm. An optional arm **D** (A + `codelaw hook --pre`, the check) is wired to W-93's real command — the
  `PreToolUse` group `sync-agents` writes, from `templates/agents/claude/pre-hook-group.json` — and refused on a
  codelaw built before W-93 landed (`4296fe9`). Dry-run for D, and the hook refused a `fetch` write in a D repo with
  `js/use-instead`. It is not in the 180: `--arms A,B,C,D` adds 60 sessions (~$22).
- **The scorer** runs `codelaw check --format json --no-adapters --no-cache --as-of 2026-10-27` before and after
  the agent; a new finding is a rise per rule, file and severity. `codelaw.toml` and `.codelaw/` are restored
  before scoring (the run is marked `tampered`); added `codelaw-ignore` comments are counted.
- **The table** (`RESULTS.md`, `results.json`): per arm the rate of runs that broke a targeted rule and that
  added any new error, with run counts, tokens, turns and cost; per task × arm; and step 4's verdict on arm C.
- **The dry run**: `harness.py dry-run` drives `fake_agent.py` through all 180 runs and fails unless every score
  is the one its script implies — which also proves every violating solution breaks its targets and every
  compliant one adds no error. 180 runs: A 60/60 broke a targeted rule, B 20/60, C 0/60, exactly as
  scripted; the output is 7.7 MB (each repo is deleted after scoring, its diff kept).

**Estimate** (`harness.py plan`): 180 sessions of `claude-opus-5-5`; per session roughly 250k–450k input tokens
(mostly cache reads), 30k–50k cache writes, 6k–9k output; **~$86 in all, a range of ~$45–$170**, and 9–18 hours
run one at a time. A guess, not a measurement: the first runs replace it.

**Not recorded in SR-WORK-GOVERNANCE:** the experiment governs no work, so the map has no row for it; the
register's Ownership table is where it is recorded.

**Left:** (1) ~~Arpit approves the spend and names the agent CLI~~ (2026-10-27); ~~the pilot~~ (2026-10-28); (2) the decider: the 180 as built, or a redesign first (inbox); the 180 runs; (3) the write-up and
step 4's README line (W-35's rule); (4) on close, the Ownership row passes to a record or the folder is archived.

## Definition of done

- The 180 runs, their inputs and the per-arm counts written up; the README line kept or removed per step 4.

## Record affected

No SR affected — an experiment. README by W-35's rule.

## Hazards

- Agent runs cost tokens and time; budget them before starting (L9's disk budget applies to the fixture repos).
- Non-determinism: report rates with the run count, never a single run.
