---
type: Handoff
name: W-107
description: "Type rules today, until W-105 ships (W-101 R1): prove and document that oxlint's type-aware mode (oxlint-tsgolint 7.x, options.typeAware in .oxlintrc.json) flows through codelaw's existing oxlint adapter, typescript/* rule ids included."
item: W-107
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-107 — The gap filler: oxlint type-aware through the adapter

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A fixture with `oxlint-tsgolint` 7.x and `options.typeAware: true` in `.oxlintrc.json`; check the adapter's parser reads `typescript/no-floating-promises` and friends | type rules work today | ⏳ |
| 2 | Check whether oxlint's `--format=json` carries fix edits for them (unverified in W-101's research); if not, they stay `manual` with docs links, as today | the fix class, named | ⏳ |
| 3 | USAGE: "type-aware rules today" — install, the one config line, and that codelaw's own rules replace it via `ported_from` once W-105 ships | people know how | ⏳ |

**Arpit, 2026-10-27 (voice), W-101:** rulings R1–R8 in [the proposal](../proposals/type-aware-rules.md) §3 — TypeScript 7.1's own API through the project's installed copy, Node at runtime, rules in the `ts` pack, preview on the 7.1 RC, *"Go with the recommendation"* on version checks.

**Model:** Sonnet — a fixture and a doc on an existing adapter.

## Definition of done

- The fixture runs in `crates/codelaw-adapters/tests`; USAGE has the section.

## Record affected

SR-LINTER decision 2 (the oxlint adapter's note), if the tested range changes; otherwise no SR affected.

## Hazards

- `tsgolint` needs TypeScript 7.0+ in the fixture; legacy `baseUrl` is unsupported.
