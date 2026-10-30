---
type: Handoff
name: W-152
description: "Arpit, 2026-10-30 (voice): W-97 and W-126 sat green with `agent` while their next step was his — the queue checks never compare a row's ball with what it says. Add that check."
item: W-152
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-152 — A row's ball must match what it says

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `scripts/check-open-work-inbox.py` (or `check-work-system.py`): refuse a 🟢/🟡 `agent` row, or an item file whose `ball:` is `agent`, when its row or its *At a glance* names the decider as the next step — "needs Arpit", "Arpit's allow", "waits on Arpit", "to the decider", "🔴 Arpit" on the first unfinished step | a wrong colour fails the check, not a person's eye | ⏳ |
| 2 | The same check: an item whose first unfinished step is an unruled choice ("offered, not ruled", "if a step needs one", options listed with no ruling) is 🔴 with an inbox row | choices reach the decider | ⏳ |
| 3 | Cases for both (a row that should fail, one that should pass), run in CI with the other queue checks; SR-WORK-OPEN-QUEUE gains the rule | it stays fixed | ⏳ |

**Arpit, 2026-10-30 (voice):** *"So it is blocked on me, so it should be red. That is the first thing. Right now it is green. Why is it green?"* · *"Same thing goes for W 126."* · *"There was nothing that could have been done if I wouldn't have made the call, right? So that should also have been red."* · then, in text: *"file it and check on all the work items. Are they actually blocked or not? And correct their balls."*

**Model:** Sonnet (Claude Code): one check with cases.

## Why it happened

- The ball is set by hand by whichever session last edits the row; the checks verify length, links and format, not that the ball agrees with the text.
- A session marks an item green when any small task in it is possible, though the step that moves it is the decider's; two sessions editing the same rows copy a wrong colour forward (Cowork's merge did, for W-97).

## Corrected 2026-10-30 (Cowork, every item read)

| item | was | now | why |
|---|---|---|---|
| W-97 | 🟢 `agent` | 🔴 `arpit` + inbox | both halves need a public repo created — Claude Code's check refused `gh repo create` |
| W-126 | 🟢 `agent` (the `perf` choice unasked) | 🟢 `agent` | ruled the same day; the matrix and non-blocking `perf` are agent work now |
| the other twelve | — | unchanged | each one's next step checked: W-95, W-102, W-151 🔴 Arpit; W-142, W-143 🟡 W-139; W-103, W-105, W-129, W-139, W-141, W-145, W-146 🟢 with agent work ready |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | both rules in the check, with cases; today's queue passes | ⏳ |
| 2 | SR-WORK-OPEN-QUEUE names the rule | ⏳ |

## Blockers

None.

## Record affected

[SR-WORK-OPEN-QUEUE](../../records/0051_WORK-open-queue.md) (the ball rules).

## Hazards

- A phrase list misses phrasings; prefer the structure (the first non-✅ step's state cell) over prose where the file has one.

## Reference

[W-97](W-97-github-action.md) · [W-126](W-126-fast-pipeline.md)
