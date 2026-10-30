# W-102 — pre-registration

Written and landed **before** the main run; `RESULTS.md` prints this file's commit and sha256.
The design is W-118's redesign ([the proposal](../../work/proposals/guidance-experiment-redesign.md)),
ruled by the decider 2026-10-30: R1 the design as written, R2 the 5% bound, R3 ~$85 with the screen
not gated separately. Nothing below changes after the run starts.

## The question, as three numbers

- **(a) the base rate** — arm 0's rate of runs that break a targeted rule, on tasks screened to tempt it;
- **(b) what guidance buys** — the task-paired drop from arm 0 to A, A+, B and C, and its token cost;
- **(c) what the write-time check adds** — arm D's rate and cost next to C's.

## Pins

| what | value |
|---|---|
| model | `claude-opus-5-5` |
| agent CLI | Claude Code 2.1.312 (the VS Code extension's binary), `claude -p … --setting-sources project --strict-mcp-config --permission-mode bypassPermissions --max-turns 40`, 900 s a session |
| codelaw | 0.2.0, the pilot's release build, sha256 `9c871735762aaff054851641198e2e3d061b8c5b257d4f869691bf38ed10a503` |
| packs and rule pages | `packs/js`, `packs/python`, `docs/rules/{js,py}` at `7b839af5` (the commit that binary was built from) |
| fux | 5.1.1 (arm C) |
| scorer | `codelaw check --format json --no-adapters --no-cache --as-of 2026-10-27` |
| task set | sha256 in `meta.json` and the table's header |
| screen | `screen.json` in `results/2026-10-w102-screen/`, sha256 in the table's header |

## Arms

Same model, same prompt — the task's text alone. `codelaw.toml` and `.codelaw/rules/` are in every
repo (open question 1, taken at the proposal's lower-risk default: visible, not hidden).

| arm | the agent has |
|---|---|
| 0 | `AGENTS.md` with layout and build only; `codelaw` not on `PATH` |
| A | + the 32 conventions in prose (`arms/rules.md`, ~350 words, the acme and `use-instead` rules among them); `codelaw` not on `PATH` |
| A+ | A + `codelaw` on `PATH` and a *Checks* section naming `codelaw check` (the pilot's A) |
| B | A + the codelaw MCP server (by full path, so with its `check_snippet` / `check_paths`) and the rule pages |
| C | B + a Fux index over the docs and the rule pages, and `fux mcp` |
| D | A + `codelaw hook --pre` as a `PreToolUse` hook |

## Tasks

**P1 — latent.** 38 candidates (18 from the pilot, 20 new; 25 target an unknown rule — `acme/*` or a
configured `use-instead`). The screen ran arm 0 alone, 5 runs each. A task is **tempting** when it broke
its target in **≥ 40%** of its runs; the **12** kept are the most tempting, ties broken by a task that
targets an unknown rule first, then by id. Fewer than 12 → the null outcome, written up, no run.

**The screen's result (2026-10-30): 7 of 38 tempting — fewer than 12, the null outcome.** No main run
was started; `results/2026-10-w102-screen/` holds the screen. What runs next is the decider's (W-102's
inbox row); whatever it is, this file is amended in a landed change before it starts.

**P2 — requested.** t02, t04, t41, t42; the right answer fixed in each `task.toml` before the run
(all four: `complied_warned`). Not screened; 2 runs per arm (48 sessions), to keep the whole inside the
ruled budget. Scored on the four-way outcome; the "warned" half is a string check on the final message
— indicative only, and not part of step 4.

## n

P1: 6 arms × 12 tasks × 5 runs = **360 sessions, 60 per arm**. P2: 48 sessions. Runs are ordered run,
then task, then arm; a cap of **$90 in all** (screen included) stops the run early with every arm on the
same tasks and runs. A session the agent CLI fails (an API error, a timeout) is kept out of every rate
and run again (`--retry-errors`) until each cell has its runs or the cap is reached.

## Analysis

- Per arm, P1: runs that broke a targeted rule, and runs that added any new `error`, each k/n with a
  Wilson 95% interval.
- Between arms: the task-paired difference (mean over tasks of the per-task rate difference), with a
  95% interval from a bootstrap that resamples tasks, 2,000 replicates, seed 20261030. Pairs: every arm
  − 0; A+ − A; B − A; C − B; D − C.
- Tokens, turns and cost per arm, means over the runs.

## The decision rule (step 4)

On P1, arm C, **runs that added any new `error`** (every rule, not only the targets): the README's
"rules it can't skip" line **is removed only if arm C added no new error in at least 60 runs** — the
rule-of-three upper bound 3/60 = 5%, ruled 2026-10-30. Any error in arm C, or fewer than 60 usable runs,
and the line **stays**, carrying arm C's measured rate and arm D's, with intervals, by W-35's rule
(the README claims only what is measured).
