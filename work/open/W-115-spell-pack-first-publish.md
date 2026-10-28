---
type: Handoff
name: W-115
description: "The spell pack has no registry name yet: @codelaw/rules-spell on npm and codelaw-rules-spell on PyPI need their first version by hand and their trusted publishers at the next release (RELEASING.md)."
item: W-115
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-115 — The spell pack's first publish

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | At the next release, take `@codelaw/rules-spell`'s tarball and `codelaw-rules-spell`'s wheel from that release's `dist` | the exact files CI built | ⏳ the next release |
| 2 | Put the commands to Arpit as an inbox row: `npm publish` by hand, then `scripts/npm-trust.sh`; `twine upload` by hand (inside PyPI's four-new-projects-a-day limit), then the trusted publisher | both names exist and CI can publish them | ⏳ |
| 3 | `publish.yml` publishes later versions; `npm view` and PyPI's JSON show the version | the pack is installable | ⏳ |

**npm done, 2026-10-29:** `@codelaw/rules-spell@0.2.0` is on npm and trusts `arpitarya/codelaw` · `publish.yml` · `npm` (checked 01:10 IST with every other npm name — 44 of 44 trusted). Left: PyPI `codelaw-rules-spell`, and `@codelaw/rules-spell-fux-index` (absent on npm).

Found 2026-10-29 (Cowork, reconciling the queue): `npm view @codelaw/rules-spell` → not found; `pypi.org/pypi/codelaw-rules-spell/json` → not found. W-103 built the pack and RELEASING.md names its first publish, but no queue item held it, and PyPI days 1–4 (W-86–W-89) do not include it.

**Model:** Haiku.

## Definition of done

- `@codelaw/rules-spell` on npm and `codelaw-rules-spell` on PyPI at the release's version, each trusting `arpitarya/codelaw` · `publish.yml` · its environment.

## Record affected

No SR affected — RELEASING.md's steps are followed as written.

## Hazards

- PyPI allows one pending publisher per repository · workflow · environment (RELEASING.md step 4), and `codelaw` holds it until the v0.2.0 Release; the first PyPI version goes up by hand.
