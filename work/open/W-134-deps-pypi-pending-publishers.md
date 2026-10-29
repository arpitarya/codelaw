---
type: Handoff
name: W-134
description: "Arpit, 2026-10-30 (voice): the deps pack's two PyPI names as pending trusted publishers, as their own item, done just before the deps release."
item: W-134
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-134 — PyPI: pending trusted publishers for the `deps` pack

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | 🟡 waits on W-128 |
| 2 | Claude, in Chrome: pypi.org → *Your account → Publishing → Add a pending publisher*, filled twice; Arpit checks and submits | `publish.yml` can create both projects | 🔴 Arpit submits |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** Cowork with Claude in Chrome — a browser form on Arpit's signed-in account; he submits.

## The form, twice

| field | value |
|---|---|
| PyPI project name | `codelaw-rules-deps`, then `codelaw-rules-deps-fux-index` |
| Owner | `arpitarya` |
| Repository | `codelaw` |
| Workflow | `publish.yml` |
| Environment | `pypi` |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both pending publishers saved on PyPI | ⏳ |
| 2 | The first release with the `deps` pack creates both projects | ⏳ |

## Blockers

🟡 W-128 (the pack must exist and be releasable).

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

## Hazards

- A pending publisher reserves nothing: someone else can take the name first — do it close to the release, as ruled, but not after it.
- B-063: the Fux-index packages are not uploaded by `publish.yml` yet; the second name may stay pending until B-063 lands.

## Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [proposal](../proposals/deps-flags-markers-reviews.md) §2.1 · [W-135](W-135-deps-npm-first-publish.md)
