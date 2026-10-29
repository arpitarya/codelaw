---
type: Handoff
name: W-132
description: "Arpit picked 2026-10-29 (Cowork, voice) component size and complexity limits, and CODEOWNERS coverage."
item: W-132
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-132 — Complexity and component-size limits; CODEOWNERS coverage

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add complexity and nesting limits for JS/TS and Python | hard-to-read functions flagged | ⏳ |
| 2 | Add React component limits: lines, props, JSX depth | components stay small | ⏳ |
| 3 | Check every file has an owner in CODEOWNERS, and no pattern is stale | reviews always reach someone | ⏳ |

**Arpit, 2026-10-29:** *"Then component size and complexity. I like that."* · *"Then code owners coverage. I want that as well."*

**Model:** Sonnet (Claude Code): metric rules beside W-58's size rules, and one file reader.

## Goal

[The proposal](../proposals/deps-flags-markers-reviews.md) §5–6.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · complexity | `js/complexity`, `py/complexity` (20), `js/max-depth`, `py/max-depth` (4); in `js/size`, `py/size` | ⏳ |
| 2 · components | `react/component-max-lines` (200), `react/max-props` (8), `react/max-jsx-depth` (6); `react/size` opt-in | ⏳ |
| 3 · codeowners | GitHub and GitLab paths, `[codeowners] path`; unowned files, stale patterns, empty owners unless `allow_unowned`; stale patterns on the CODEOWNERS line, unowned paths on the root manifest — both places, Q-A ruled B (W-136) | ⏳ |
| 4 · records, tests, docs | SR-RULE-PACKS, SR-CONFIG `[codeowners]`; manual fixes with a skill; corpus counts; L8 | ⏳ |

## Blockers

None. The CODEOWNERS-line location waits on W-136.

## Record affected

SR-RULE-PACKS · SR-CONFIG (`[codeowners]`).

## Hazards

- CODEOWNERS is not an L3 language; findings on its lines come only through W-136's amendment.
- Which pack holds `codeowners-coverage` is the agent's call (`data` fits: a repo-level file); say so when it lands.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §5–6 · W-58 (`archive/open/`)
