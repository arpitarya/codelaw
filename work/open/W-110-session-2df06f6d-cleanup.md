---
type: Handoff
name: W-110
description: "Arpit, 2026-10-27 (voice docket): clean what Claude Code session 2df06f6d left on the Mac — no worktree and no process left behind."
item: W-110
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-110 — Clean session 2df06f6d's leftovers

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Remove the five worktrees whose branch is in `main` (W-62, W-93, W-94, W-103, W-104) with `worktree-cleanup.py --apply` | disk back, no stale branches | ⏳ — a Claude Code session on the Mac (a Cowork session cannot clean, SR-WORK-CLEANUP decision 11) |
| 2 | Remove `.claude/worktrees/W-{58,82,85}` — three folders holding only a `.fux` index, no git worktree | the last ~9 MB | ⏳ |
| 3 | Check no codelaw, cargo or rust-analyzer process from a dead session still runs | no memory held | ⏳ |
| 4 | Drop `work/NOW.md`'s resource rows for what was removed | NOW shows only what is live | ⏳ |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** on `NOW.md`'s stale session row: *"It's not running on my Mac, but make sure uh, no session is active. If active meaning there shouldn't be any working trees left or extra memory that is being consumed. Make sure that it gets cleaned up."*

**Model:** Haiku.

## Found (2026-10-28, Cowork, read-only)

- `NOW.md`'s *In flight* is empty: the old `c0826a95` row was already removed by session `2df06f6d`, which then ended and listed seven worktrees under *Resources*.
- `git worktree list`: seven `lm/*` worktrees under `.claude/worktrees/`. Branch in `main` (`git merge-base --is-ancestor`): **W-62, W-93, W-94, W-103, W-104** — safe by decision 1(a); (b)–(d) are checked by the script on the Mac. **W-90 and W-96** hold commits `main` lacks — red, [W-111](W-111-two-unmerged-worktrees.md).
- Sizes: W-62 32 MB, W-93 57 MB, W-94 53 MB, W-103 33 MB, W-104 58 MB; the three `.fux`-only folders 9.1 MB, 8 KB and 8 KB.
- Processes on the Mac cannot be seen from the Cowork VM.

## How (on the Mac, from the main checkout)

```bash
python3 .claude/hooks/worktree-cleanup.py          # the verdicts
python3 .claude/hooks/worktree-cleanup.py --apply  # removes only the safe ones
rm -rf .claude/worktrees/W-{58,82,85}
pgrep -fl 'codelaw|cargo|rust-analyzer'            # anything left from a dead session
```

## Definition of done

- `git worktree list` shows only `main` and whatever W-111's ruling keeps; `.claude/worktrees/` holds nothing else; no process from session `2df06f6d` runs; NOW's resource rows match.

## Hazards

- **A live session may own a worktree again.** The `lm/W-103` and `lm/W-62` branches moved after this item was filed (W-103 landed at 08:12 and 08:15 IST, 2026-10-28), so live sessions are building in those worktrees. Run the script's report first; it refuses a worktree whose lock a live process holds (SR-WORK-CLEANUP decision 1(d)) — never remove one by hand while NOW lists a session on it.

## Record affected

No SR affected — SR-WORK-CLEANUP is followed as written.
