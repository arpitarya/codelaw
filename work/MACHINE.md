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
