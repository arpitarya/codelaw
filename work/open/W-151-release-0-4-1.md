---
type: Handoff
name: W-151
description: "Arpit, 2026-10-30 (Claude Code): release v0.4.1 from CI, so every package behind 0.4.0 — PyPI's eleven Fux index wheels, npm's having been published by hand — catches up, and the deps pack ships."
item: W-151
filed: 2026-10-30
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-151 — Release v0.4.1

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `publish.yml` uploads every package a release builds: the dev-server plugins (W-150), the Fux index set (B-063), `codelaw-rules-deps` through its pending publisher (W-134) | nothing is left behind again | ✅ 2026-10-30 (W-150's change) |
| 2 | Agent: `python3 scripts/version.py --set 0.4.1`; CHANGELOG's *Unreleased* becomes *0.4.1*; `ci.yml` green on that commit on `main` | a releasable commit | 🔄 bumped 2026-10-30 (`version.py --set 0.4.1`: Cargo, 19 packs, the extension, the Action); CHANGELOG *0.4.1 — 2026-10-30* written; `ci.yml` on it pending |
| 3 | **Arpit:** `scripts/npm-trust.sh --fux-index` (W-85's *Pending validation* configurations lapse after ~2 days, and no CI publish ever validated the index names); then **W-135** — `npm publish` `@codelaw/rules-deps` and `@codelaw/rules-deps-fux-index` at 0.4.1 by hand from step 2's run, and `scripts/npm-trust.sh` for both | npm trusts every name, or `trust` stops the release before anything is written | 🔴 Arpit |
| 4 | **Arpit:** `gh release create v0.4.1` with CHANGELOG's section as notes (RELEASING.md step 3) | every package at 0.4.1 on both registries | 🔴 Arpit |
| 5 | Agent: every name checked at 0.4.1 on npm and PyPI; `v0` moved to `v0.4.1` (W-97); the run timed for W-126; W-134 step 3 | proof | ⏳ |

**Arpit, 2026-10-30 (Claude Code), asked how the packages behind 0.4.0 should catch up — CI cannot upload them at 0.4.0 (the `npm`/`pypi` environments take `v*` tags only; v0.4.0's own `publish.yml` has no tier for them) and this machine has no npm login:** *"Release v0.4.1 from CI"* — over *"You log in, I publish"* and *"Wire CI only, skip catch-up"*. Asked the same turn: *"close out item 149, item 126, item 95, item 97. Make sure the pipeline is green completely. And publish the packages that haven't been upgraded to .4.0 version."*

**Model:** Sonnet (Claude Code) for steps 2 and 5; steps 3–4 are the decider's hand.

## Behind 0.4.0 (checked 2026-10-30 on both registries)

| registry | packages | at | since |
|---|---|---|---|
| npm | `@codelaw/vite-plugin`, `@codelaw/webpack-plugin`, `@codelaw/next` | 0.2.0 | ✅ 0.4.0 — Arpit, by hand, 10-30 10:30 IST (W-150 step 1) |
| npm | `@codelaw/rules-<name>-fux-index` × 16 | 0.2.0; `spell`'s 0.3.0 | ✅ 0.4.0 — Arpit, by hand, 10-30 (all 16 on the registry, rechecked 10:40) |
| PyPI | `codelaw-rules-<name>-fux-index` × 11 | 0.2.0; the security three and `spell`'s 0.3.0 | ⏳ this release |

Every other name is at 0.4.0. v0.4.0's CI run (40932052080) holds the PyPI wheels at 0.4.0 until 2026-11-28, but no workflow may upload them at that version. **So this release now carries:** PyPI's eleven index wheels, the `deps` pack and its index (new), and everything else at 0.4.1.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | npm and PyPI show every codelaw name at 0.4.1, `deps` and its index included (PyPI's `deps` index once W-134 step 3 adds its pending publisher) | ⏳ |
| 2 | `publish.yml`'s run green, its time recorded in W-126 | ⏳ |

## Blockers

Steps 3–4: the decider's npm login and the Release.

## Record affected

None beyond W-150's change (SR-PACKAGES decisions 3a, 11, 14; L6 decision 9; RELEASING.md).

## Hazards

- `trust` checks every npm name before either registry is written: one lapsed or missing name (the `deps` pair, a lapsed index name) stops the whole release — step 3 first.
- 0.4.1 carries everything since 0.4.0 (the `deps` pack, flags, markers, complexity, W-137's editor), not only the catch-up.

## Reference

[W-150](W-150-publish-dev-server-plugins.md) · [W-134](W-134-deps-pypi-pending-publishers.md) · [W-135](W-135-deps-npm-first-publish.md) · [W-126](W-126-fast-pipeline.md) · [W-97](W-97-github-action.md) · RELEASING.md
