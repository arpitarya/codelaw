---
type: Handoff
name: W-116
description: "Arpit, 2026-10-29 (voice): release 0.3.0 through CI — every package on npm; on PyPI only codelaw and the packs that already exist there, the rest skipped until their first upload by hand."
item: W-116
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-116 — Release 0.3.0 through CI

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first: `publish.yml` skips a PyPI project that does not exist yet, naming it, instead of failing the job (L6 decision 9, SR-PACKAGES, RELEASING.md) | a release that never half-fails on a name PyPI lacks | ✅ |
| 2 | `publish.yml`'s `pypi` job uploads only wheels whose project exists (or holds the pending publisher, `codelaw`); the skipped names are printed in the job summary | the change | ✅ |
| 3 | npm: every name at 0.3.0 — the new spell pack's first version by hand first ([W-115](W-115-spell-pack-first-publish.md)), or the job skips it the same way | npm complete — every name is already on npm at 0.2.0, the spell pack included; nothing by hand | ✅ |
| 4 | Bump to 0.3.0 (`scripts/version.py`), CHANGELOG's 0.3.0 section; `ci.yml` green on `main` | the release commit | ✅ 2026-10-29 — 0.3.0 from `457764c`, `ci.yml` green |
| 5 | Arpit: `gh release create v0.3.0 …` (an inbox row with the exact command when step 4 is green) | CI publishes | ✅ 2026-10-29 01:28 IST — v0.3.0 released |
| 6 | Check: npm shows every `@codelaw/*` at 0.3.0; PyPI shows `codelaw` (created through its pending publisher) and the six packs at 0.3.0 | the release, verified | ✅ 2026-10-29 10:05 — npm `codelaw` 0.3.0 (every name); PyPI `codelaw` 0.3.0 (created through its pending publisher) and the six packs 0.3.0 |

**Arpit, 2026-10-29 (voice docket):** offered the v0.2.0 Release (W-86): *"Let's do this. Let's publish version 0.3.0 for all the packages through CI/CD pipeline."* Told CI can publish only what exists and trusts `publish.yml`: *"We can skip the one pending for pipe here. The ones for NPM, it is being built and it should be ready. In few minutes."* ("pipe here" is PyPI) — then *"Next."* to filing it.

**Model:** Opus — a release and a `publish.yml` change.

## What 0.3.0 reaches

- **PyPI, through CI:** `codelaw` (its pending publisher, registered 2026-10-25) and `codelaw-rules-{css,data,html,js,python,react}` (trusted). **Skipped:** `codelaw-rules-{scss,security-js,security-html,security-python,spell}`, `codelaw-skills`, the Fux index projects not yet created — they come by hand (W-87–W-89, W-115), then CI from the next release.
- **npm:** every name already published and trusted; `@codelaw/rules-spell` is new (W-115).

**Changed 2026-10-29 01:17:** `codelaw` 0.2.0 now goes to PyPI by hand on day 2 (W-87, W-86), with a normal trusted publisher after; v0.3.0's CI uploads it as an existing project, not through the pending publisher.

**2026-10-29 09:59:** `codelaw` is now on PyPI and trusted (W-86/W-87), and PyPI holds **no pending publisher** — the one slot for `arpitarya/codelaw` · `publish.yml` · `pypi` is free again, so one new project (e.g. `codelaw-rules-spell`) could be created by CI through it, if `publish.yml`'s skip rule allows a pending name.

**Closed 2026-10-29 (Cowork, reconciling):** v0.3.0's `publish.yml` run 40759085164 — gate, trust, npm, pypi, attach green (IMPLEMENTATION, 2026-10-29). Its `vscode` job reached the Marketplace through trusted publishing but the upload was refused (HTTP 400, content type) — fixed on `main`; the extension goes on with [W-95](../../work/open/W-95-publish-vscode-extension.md). The DoD is met.

## Definition of done

- v0.3.0's `publish.yml` run is green; npm and PyPI show the versions in step 6; W-86 closes (the Release step done), W-38's check runs.

## Record affected

[SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 (by-hand first uploads; a missing project skipped) · [SR-PACKAGES](../../records/0107_packages.md) · RELEASING.md.

## Hazards

- **W-86's Release:** it was v0.2.0's; now this item's. A session (`c6c222e8`) was on it at 00:31 on 2026-10-29 — do not run both.
- **Same day:** W-97's Marketplace tick and W-95's Visual Studio Marketplace upload by hand both happen on this release.
- PyPI's pending publisher allows one project per repository · workflow · environment (RELEASING.md step 4): only `codelaw` can be created by CI.
