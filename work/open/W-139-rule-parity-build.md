---
type: Handoff
name: W-139
description: "Arpit, 2026-10-30 (voice): build into codelaw's packs every rule W-138's research marks worth porting, once he has reviewed it."
item: W-139
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-139 — Build: port the rules W-138 picks

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait for W-138's research and Arpit's ruling on its port list | a decided list | 🟡 waits on W-138 |
| 2 | Write each rule fresh in its pack, with cases, a fix (auto, suggest or manual + skill) and its doc | the depth the single-language tools have | ⏳ |
| 3 | `codelaw test` on every pack, the corpus counts, the L8 budgets | nothing slower, nothing wrong | ⏳ |
| 4 | Counts per pack before and after, in the proposal and the README | the gap closed, measured | ⏳ |

**Arpit, 2026-10-30 (voice):** *"One big item for Opus to research it and once the research is done, then one big work item to build it."*

**Model:** Opus (Claude Code): hundreds of rules across every pack; split into parallel worktrees per pack only within L9's disk budget.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rules | every rule on the ruled port list, in its pack, with valid and invalid cases and its fix class | ⏳ |
| 2 · presets | each new rule placed in a preset (or off by default), as the ruling says; B-048's presets if ruled in | ⏳ |
| 3 · checks | `codelaw test` 0 failed on every pack; the layout check clean; corpus counts; L8 budgets measured, not loosened | ⏳ |
| 4 · docs | `scripts/rule-docs.py` pages, `docs-sync.py` counts, CHANGELOG, the proposal's before/after table | ⏳ |

## Blockers

🟡 waits on **W-138** (the research and Arpit's ruling on it).

## Record affected

[SR-RULE-PACKS](../../records/0101_rule-packs.md) (the catalogue) · SR-FORMATTER if a `layout` rule exposes a formatter gap.

## Hazards

- Rewrite, never copy upstream text or cases (L2, L11); no new dependency.
- A rule that needs types waits for W-105's helper — do not fake it syntactically.
- Rule ids must not collide with names W-128–W-132 reserve.

## Reference

[W-138](W-138-rule-parity-research.md) · W-03 · W-105
