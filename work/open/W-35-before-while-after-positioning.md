---
type: Handoff
name: W-35
description: "Arpit, 2026-10-25 (voice): market codelaw as the tool that helps an AI agent before, while and after it writes code — the README and public docs lead with it."
item: W-35
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), 2026-10-25: step 2 landed"
ball: agent
---

# W-35 — Lead the README with "before, while and after the AI writes"

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Agree the line and the three beats | the story | ✅ 2026-10-25, below |
| 2 | Put it at the top of the README: before and after as they are today, while marked "in design" with a link | the pitch on the front page, true on the day it lands | ✅ 2026-10-25: README's top section. **Arpit, 2026-10-25:** *"Say what ships"* — Before reads "gets the rules for it" until `brief_for_task` (W-05) narrows them; the "handful, not a wall" line returns with it |
| 3 | When the while-writing check ships, drop the "in design" label; when `brief_for_task` ships, restore Before's "the handful of rules that matter" | the full story | ⏳ waits on W-05 |

**Arpit, 2026-10-25 (voice):** *"I want it as a marketing thing where codelaw can
help agents before writing a code, while writing a code and after writing a
code … I believe it will be a good way to sell codelaw."* → on the draft below:
*"Looks good."*

**Model:** Sonnet — copy into the README, no code.

## The copy (agreed 2026-10-25)

**Line:** *codelaw: your rules, before, while and after the AI writes.*

| beat | what the agent gets | ships in |
|---|---|---|
| **Before** | it asks what applies to this file and gets the handful of rules that matter, not a wall of 132 | today: `rules_for_path`, the skills (W-08, W-10); narrowed by W-05's `brief_for_task` |
| **While** | every edit is checked before it reaches disk; a bad line is refused with the reason and the fix | W-05's `hook --pre` — **not built** |
| **After** | anything that slips through is fixed on save, in the hook and in CI | today: `codelaw hook`, `check`, `fix` |

**The kicker:** *no AI inside codelaw — the same verdict on every machine, with
or without an agent* ([SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md)).

Alternative lines offered, not chosen: *"Rules your AI follows, not just
reads."* · *"Guide it. Guard it. Fix it."*

## Hazards

- **The README must not claim the while beat until it ships** — step 2 labels it.
- **Parts of the README are generated** by `scripts/docs-sync.py` (W-26): write
  the section outside its regions, or into the source it renders from, and run
  `docs-sync.py --check`.

**Records affected:** no SR affected — README only
([SR-WORK-DOCS](../../records/0055_WORK-docs.md) decision 10, item 6).
