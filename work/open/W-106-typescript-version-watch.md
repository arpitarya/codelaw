---
type: Handoff
name: W-106
description: "Keep type rules working on every TypeScript release (W-101 R8): a tested range with a warning outside it, a CI matrix (latest 7.1 blocking, typescript@next nightly reporting), and a scheduled bump job that opens a PR or an issue."
item: W-106
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-106 — TypeScript version watch

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A tested TS range in SR-TYPES and the helper; outside it, type rules run with a warning (the adapter pattern) | users on newer TS keep working | ⏳ |
| 2 | `ci.yml`: the type-rule fixtures against the latest 7.1 on every change (blocking) | no regressions | ⏳ |
| 3 | Nightly: the same against `typescript@next`, report only | early warning | ⏳ |
| 4 | `typescript-bump.yml`, modelled on `fux-bump.yml`: a new TS release → run the fixtures → PR widening the range, or an issue naming what broke | nobody has to watch | ⏳ |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Sonnet — CI wiring on a known pattern.

## Definition of done

- A dispatch with an older and a newer TS version each produces the right PR or issue (as W-91 proved `fux-bump.yml`).

## Record affected

SR-TYPES (W-104) for the range; SR-PACKAGES if `ci.yml` jobs are named there.

## Hazards

- The nightly must never block `main`.
