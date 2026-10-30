---
type: Handoff
name: W-147
description: "Arpit, 2026-10-30 (voice): the testing and nextjs packs W-139 builds are npm only; their names and Fux indexes are first published by hand, just before the release that carries them."
item: W-147
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-147 — npm: first publish of the `testing` and `nextjs` packs' names

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-139 has the two packs ready to release | the names are bound when they are needed | 🟡 waits on W-139 |
| 2 | Arpit publishes the four names below once by hand, as RELEASING.md does for a new pack | the names exist on npm | 🔴 Arpit |
| 3 | Add `arpitarya/codelaw` · `publish.yml` · `npm` as each package's trusted publisher | every later release publishes them from CI | 🔴 Arpit |

**Arpit, 2026-10-30 (voice):** testing pack — *"Let's create it."* · *"The next JS pack. Let's create it."*

**Model:** NONE for steps 2–3 — npm has no pending publisher; only Arpit's npm login can publish a new name.

## The names

| pack | npm | its Fux index (L6 decision 12) |
|---|---|---|
| testing (Jest, Vitest, Testing Library, Playwright) | `@codelaw/rules-testing` | `@codelaw/rules-testing-fux-index` |
| Next.js | `@codelaw/rules-nextjs` | `@codelaw/rules-nextjs-fux-index` |

**npm only:** Python's test rules live in `python`; Next.js is JavaScript. Under the `@codelaw` scope, so no outside name can take them.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | All four names on npm with a trusted publisher | ⏳ |
| 2 | The next release publishes both packs from `publish.yml` | ⏳ |

## Blockers

🟡 W-139.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · L6 decision 11 (W-139 writes the amendment) · RELEASING.md.

## Hazards

- B-063: `publish.yml` does not upload Fux-index packages yet; the fux-index names are bound by hand regardless.
- Until both names have a trusted publisher, `publish.yml`'s trust check stops a release carrying the packs.

## Reference

[W-139](W-139-rule-parity-build.md) · [W-144](../../archive/open/W-144-rule-parity-round-2.md) · [W-135](W-135-deps-npm-first-publish.md) (the same steps for `deps`) · [W-142](W-142-framework-pack-names.md) (PyPI's five)
