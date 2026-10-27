---
type: Handoff
name: W-91
description: "Arpit, 2026-10-27 (voice docket): fux-engine always at its latest release — ratified with no fixed version; a scheduled job bumps the pin and opens a PR."
item: W-91
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-91 — fux-engine at its latest, every commit still pinned

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A scheduled workflow (daily) asks PyPI for `fux-engine`'s latest version and compares it with `FUX_VERSION` in `scripts/fux-index.py` | a new fux is noticed within a day | ⏳ |
| 2 | On a newer one: bump `FUX_VERSION`, re-run `fux-index.py assemble` + `check`, commit on `deps/fux-<v>`, open or update one PR | the bump arrives as a reviewable change | ⏳ |
| 3 | The PR's CI builds the indexes with the new fux and checks their digests reproduce | a bump that breaks the index is red, not merged | ⏳ |
| 4 | D-34's row says "unversioned; each commit pins `FUX_VERSION`, bumped by this job" | the record matches | ✅ 2026-10-27 (`DEPENDENCIES.md`, D-34) |

**Arpit, 2026-10-27 (voice docket):** *"For Fux, what I want is whenever there is a latest version available, we should be using that."* On the trade-off (CI grabbing "latest" would break W-34's reproducible digest): *"Go with the recommendation."* — ratify fux-engine with no fixed version; pin per commit; a scheduled job bumps the pin and opens a PR.

## Definition of done

- The job runs on a schedule and by hand (`workflow_dispatch`); with no new fux it changes nothing and is green.
- A run against a faked newer version opens one PR; a second run updates that PR rather than opening another.
- The PR's own CI runs (a PR opened with `GITHUB_TOKEN` triggers no workflows — use a token or `workflow_dispatch` on the branch, and say which).

## Record affected

`DEPENDENCIES.md` D-34 (done) — moves into L11 with W-90; [SR-PACKAGES](../../records/0107_packages.md) decision 14 if it names the pin.

## Hazards

- No new third-party action (L11): use `gh` and `git` on the runner, not a marketplace PR action.
- The repository setting *Allow GitHub Actions to create pull requests* must be on — the decider's, if it is off.
- L11 decision 4 ("the tree as pinned that day") — W-90 states the unversioned ratification in the law.

**Model:** Sonnet.
