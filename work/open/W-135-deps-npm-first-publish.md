---
type: Handoff
name: W-135
description: "Arpit, 2026-10-30 (voice): the deps pack's two npm names, first published by hand, as their own item, done just before the deps release."
item: W-135
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-135 — npm: first publish of the `deps` pack's names

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | ✅ W-128 built and closed 2026-10-30; the next release (after 0.4.0) carries `deps` |
| 2 | Arpit publishes `@codelaw/rules-deps` and `@codelaw/rules-deps-fux-index` once by hand, as RELEASING.md does for a new pack | the names exist on npm | 🔴 Arpit |
| 3 | Add `arpitarya/codelaw` · `publish.yml` · `npm` as each package's trusted publisher | every later release publishes them from CI | 🔴 Arpit |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** NONE for steps 2–3 — npm has no pending publisher; only Arpit's npm login can publish a new name.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both names on npm with a trusted publisher | ⏳ |
| 2 | The next release publishes both from `publish.yml` | ⏳ |

## Blockers

None — W-128 closed 2026-10-30. Do it just before the next release.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

## Hazards

- [W-150](W-150-publish-dev-server-plugins.md) (was B-063): `publish.yml` does not upload Fux-index packages yet; the fux-index name is bound by hand regardless.

## Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [W-134](W-134-deps-pypi-pending-publishers.md) · RELEASING.md
