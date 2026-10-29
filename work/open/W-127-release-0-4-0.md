---
type: Handoff
name: W-127
description: "Arpit, 2026-10-29: \"upgrade 0.4.0\" — release 0.4.0 through CI: every name on npm, every PyPI project (all exist since W-119), the extension as codelaw.codelaw-vscode."
item: W-127
filed: 2026-10-29
filed_by: "Claude Code (Opus 5.5)"
updated: 2026-10-29
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-127 — Release 0.4.0 through CI

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `scripts/version.py --set 0.4.0` (Cargo, the eighteen packs, the extension, the Action's default), CHANGELOG's 0.4.0 section | the release commit | ✅ 2026-10-29 |
| 2 | Push `main` to `origin` (it also carries the 17 commits before it — W-95, W-119, W-126, …) | CI runs on the release commit | ✅ 2026-10-29 — pushed `6553a6d..7accdae` |
| 3 | `ci.yml` green on that commit, on `main` (the first run of W-126's pipeline) | `publish.yml`'s gate passes | 🟡 run 40927104004: all green but `lint` (the guard replay; RULE-SINCE moved past 2c7fee8) — re-run on the fix |
| 4 | Arpit: notes from CHANGELOG, then `gh release create v0.4.0 --title "v0.4.0 — codelaw complete on PyPI, and the editor extension under its own name" --notes-file notes.md` on the green commit (RELEASING.md §Every release step 3) | CI publishes npm, PyPI and the extension | ⏳ after 3 |
| 5 | Check: npm every `@codelaw/*` at 0.4.0, PyPI every project at 0.4.0, the Marketplace `codelaw.codelaw-vscode` 0.4.0; W-126 step 4 takes the release's times | the release, verified | ⏳ |

**Arpit, 2026-10-29:** *"upgrade 0.4.0"*.

**Model:** Opus.

## What 0.4.0 is

The engine and the rules are 0.3.0's (no change under `crates/` or `packs/` but the version since v0.3.0). What is new: every PyPI project exists (W-119), so the `pypi` job uploads every wheel; the extension is `codelaw.codelaw-vscode` (W-95); the pipeline of W-126. **Before step 4:** RELEASING.md step 0 — the npm trusted publishers, re-checked.

## Hazards

- W-126's `ci.yml` and `publish.yml` run for the first time on this release; a red `ci.yml` is fixed on `main` and the bump stays.
- W-125 (revert `only=pypi-new`) is open; it does not block the release.

**2026-10-29, Arpit:** *"commit everything and publish the version. If any stage fails, fix it and then push it. Make sure everything is green and the packages and even the VS Code extension gets published."* — Claude Code cuts the Release (step 4) once `ci.yml` is green.
