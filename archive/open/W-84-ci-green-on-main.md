---
type: Handoff
name: W-84
description: "Arpit, 2026-10-27: bring OPEN-WORK up to date — CI on main has been red since 2026-10-26 11:33 UTC and no item tracked it; this one makes main green again."
item: W-84
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-84 — CI green on `main` again

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Fix `lint`'s step *the SR guard, replayed over history* — red on every run since 2026-10-26 11:33 UTC (9d0bfa0 onward) | the commit guard holds over history again | ✅ eight pushed commits since `fedf2f0` failed the replay (CI showed only the newest, 791b189); `records/RULE-SINCE` moved to `791b189`, each named (eb0e970) |
| 2 | Fix `test` (`cargo test --workspace --locked`) — red on `3f3d054` (run 40423534120), green on `fdd8345` | the workspace tests pass on `main` | ✅ run 40437239647 |
| 3 | Fix `smoke` — all eight legs fail *npm (no Python on PATH) and PyPI (no Node on PATH) give byte-equal output*; the two musl legs also fail their Alpine step | the release smoke passes on every platform | ✅ 175a328 (BusyBox `diff -x`, win32 `--prefer-warm`); all eight legs green on run 40437239647 |
| 4 | Confirm `dev-servers (vite)` — it refused the `Origin` header on `3f3d054`; `f419f32` sends it to webpack-dev-server hosts only | W-70's five hosts judged | ✅ all five green on run 40437239647 |
| 5 | One `ci.yml` run on `main` completes with every job green; its run id goes in `IMPLEMENTATION.md` | W-44, W-70 and W-71 can close | ✅ run 40437239647 on `eb0e970`: 27/27 jobs |

**Arpit, 2026-10-27:** *"Update the OpenWork file. Bring it to the latest updates."* — the check found CI red on `main` with no item tracking it.

**Model:** Opus (diagnosis from the job logs), on the Mac with `gh` — the run logs need an authenticated `gh run view --log-failed`.

## Evidence (2026-10-27, from the public Actions API)

| run | commit | failing jobs |
|---|---|---|
| 40315989524 | 9d0bfa0 | lint, corpus, perf, build win32-x64, build win32-arm64 |
| 40382552494 | a5c2a5b | lint, build win32-x64, build win32-arm64 |
| 40388001095 | fdd8345 | lint, smoke win32 ×2 (`dev-servers` first run here) |
| 40423534120 | 3f3d054 | lint, test, smoke ×8, dev-servers (vite) |
| 40434117921 | f419f32 | lint (others still running at 07:15 IST) |

The last green run on `main` was 40241607191 (683eb72, 2026-10-25 22:44 UTC).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | a `ci.yml` run on `main`: `completed/success`, every job | ✅ run 40437239647 |
| 2 · | each fix is a row in `IMPLEMENTATION.md` naming the run that proved it | ✅ 2026-10-27 row, run 40437239647 |
| 3 · | W-44 DoD 5, W-70 step 2 and W-71 DoD 10 re-read against that run | ✅ handed to change W-70 (session c0826a95), which closes them citing this run |

## Record affected

No SR affected, unless step 1's cause is in a record or `records/RULE-SINCE` — then that record, in the same change.

## Hazards

- Another session (`c0826a95`, NOW.md) has been landing CI fixes for W-44/W-70; check NOW.md and `git log` before starting, so two sessions do not fix the same job.
- A loaded Mac reproduces timing failures that a runner does not (LESSONS); judge by the runner's log.
