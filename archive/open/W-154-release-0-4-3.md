---
type: Handoff
name: W-154
description: "Arpit, 2026-10-30 (Claude Code): release v0.4.3 so that publish.yml creates codelaw-rules-deps-fux-index on PyPI through its pending publisher."
item: W-154
filed: 2026-10-30
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: none
---

# W-154 — Release v0.4.3: the `deps` index on PyPI

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `version.py --set 0.4.3`; CHANGELOG *0.4.3*; `ci.yml` green on that commit on `main` | a releasable commit | ✅ `5b38accd`, `ci` 41035668356 |
| 2 | `gh release create v0.4.3` → `publish.yml` | `codelaw-rules-deps-fux-index` created on PyPI by its pending publisher; every package at 0.4.3 | ✅ [v0.4.3](https://github.com/arpitarya/codelaw/releases/tag/v0.4.3), `publish` 41037110590 green first time, 4m59 |
| 3 | every name at 0.4.3 on npm (47), PyPI (26) and the Marketplace (8); `v0` → `v0.4.3`; `PENDING` drops the name; README at 0.4.3 | proof | ✅ npm 47/47, PyPI 26/26, Marketplace 8/8 (10:04 UTC); `PENDING` back to `codelaw`; README at 0.4.3 — `v0` → v0.4.3 (`5b38accd`) pushed by Arpit, 2026-10-30 |

**Arpit, 2026-10-30 (Claude Code):** *"publish the pending pypi package"* — asked how, since a run from v0.4.2 uses v0.4.2's `publish.yml`, whose `PENDING` lacks the name, and PyPI is never uploaded from a laptop (W-134's ruling): *"Release v0.4.3 now"* (over a PyPI-only run from a v0.4.3 tag, or waiting).

**Model:** Sonnet (Claude Code).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | `codelaw-rules-deps-fux-index` 0.4.3 on PyPI; every other name at 0.4.3 | ✅ |
| 2 | `ci` and `publish` green | ✅ |

## Closed 2026-10-30 (Claude Code)

v0.4.3 released from CI; every codelaw package on every registry (npm 47, PyPI 26 — `codelaw-rules-deps-fux-index` created by its pending publisher — Marketplace 8); `PENDING` back to `codelaw`; `v0` at v0.4.3, moved by Arpit.

## Blockers

None.

## Record affected

None — a version bump (SR-PACKAGES decision 7). `PENDING`'s list in SR-PACKAGES decision 11 and L6 decision 9 changes after, when the name leaves it.

## Hazards

- Sigstore (Rekor) failed twice on v0.4.2's run; a failed upload job is re-run, not re-released (RELEASING).
