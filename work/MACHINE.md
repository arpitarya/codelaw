---
type: Reference
description: "Environment and tooling quirks of wherever this runs — why a command fails on THIS surface."
---

# MACHINE — the surfaces this runs on

One section per surface (a laptop, a CI runner, a Cowork bridge shell). A row
is added when something fails for a reason the code did not cause.

| surface | symptom | cause | what to type instead |
|---|---|---|---|
| macOS laptop (zsh) | `timeout: command not found` | GNU coreutils is not installed | leave `timeout` out; use the tool's own timeout |
| macOS laptop (zsh) | a script using 3.10+ syntax fails | `python3` is 3.9.6 here (the L6 floor of 3.11 is for consumers) | keep repo scripts to Python 3.9 syntax |
| GitHub Windows runner | a repo script printing `✔` dies with `UnicodeEncodeError` | the console is cp1252 | `PYTHONUTF8=1` (set in `ci.yml`); the scripts also reconfigure stdout to UTF-8 |
| Claude Code cloud session | `gh api …/jobs/<id>/logs` and `…/actions/caches` refused | the session's proxy blocks Actions log blobs and the cache API | read a failure from the job's check-run annotations (`gh api repos/<r>/check-runs/<id>/annotations`); steps print their own `::error::` |
| Claude Code cloud session | `No space left on device` with several agents building at once | each cargo target dir is ~3 GB with debug info; the session's disk allowance is ~40 GB | one target dir per lane with `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`; delete a lane's target dir when it finishes |

## Disk budget — law L9: this section is always followed

**This section binds** ([SR-LAW-9](../records/0011_LAW-9-machine-budget.md)): every session and every
agent follows it — Claude Code or Cowork, main agent or sub-agent. A number here
changes only by the decider's ruling. `.claude/hooks/disk-budget.py` reads the
`budget` row below; keep its `<n> GB` spelling.

| what | value |
|---|---|
| budget | 5 GB for the whole project — main session + every sub-agent + every worktree + build output, combined |
| past the budget | every further GB needs the decider's approval, asked before it is used: Claude Code raises a permission prompt; a Cowork session asks in chat. One yes approves one GB |
| new sub-agent or worktree | asks from 90 % of the approved ceiling |
| approvals reset | when the project is back under the budget |
| not counted | shared caches outside the repo: `~/.cargo`, the pnpm store, pip's cache |
| see it | `python3 .claude/hooks/disk-budget.py fresh` |
| codelaw's cargo | one shared `CARGO_TARGET_DIR` (`.claude/settings.local.json`) and no debug info for dev and test |
| at filing (2026-10-24) | codelaw is 15 GB — past the budget; `target/` alone is 15 GB, rebuildable with `cargo clean` (the installer offers it) |
