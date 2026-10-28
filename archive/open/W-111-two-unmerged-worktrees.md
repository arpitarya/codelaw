---
type: Handoff
name: W-111
description: "SR-WORK-CLEANUP decision 3: two worktrees of session 2df06f6d hold commits main lacks — the decider rules keep or drop."
item: W-111
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-111 — Two unmerged worktrees: keep or drop

## At a glance

| worktree | branch commits not in `main` | is its work in `main`? | recommend | state |
|---|---|---|---|---|
| `.claude/worktrees/W-90` (49 MB) | `52be338`, `71e631a` — *"W-90: L11 is "dependencies" and holds the list; DEPENDENCIES.md archived"* | yes — W-90 is closed and archived; `archive/DEPENDENCIES.md` and L11 §3 are in `main` | drop | 🔴 the decider |
| `.claude/worktrees/W-96` (56 MB) | `2858069` (W-96's change), `0211a57` (a merge of `main` into it) | yes — W-96 is closed and archived; `crates/codelaw-report/src/agent_env.rs` is in `main` | drop | 🔴 the decider |

Filed from W-110 on Arpit's ask, 2026-10-27 (voice docket): *"Make sure that it gets cleaned up."* SR-WORK-CLEANUP decision 3 still needs a ruling per worktree before `--force` or `-D`.

**Model:** Haiku.

## Why they read as unmerged

Each item landed through lockmaster as other commits than its branch tip, so the tip is not an ancestor of `main`. Files the branches touched now differ from `main` because `main` moved on (the `.fux/index` shards, later edits), not because work is missing — a byte-level check of every file is the agent's before dropping.

## On "drop" (on the Mac)

```bash
git worktree remove --force .claude/worktrees/W-90 && git branch -D lm/W-90
git worktree remove --force .claude/worktrees/W-96 && git branch -D lm/W-96
git worktree prune
```

**Closed 2026-10-29 (Cowork): merged into [W-114](../../work/open/W-114-session-cleanup.md)**, which the session-end hook filed for the same session with these two worktrees and three more; one ruling covers all five.

## Definition of done

- The decider has ruled per worktree, and each is removed (drop) or merged and then cleaned (keep).

## Record affected

No SR affected.
