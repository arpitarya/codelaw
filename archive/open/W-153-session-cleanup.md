---
type: Handoff
name: W-153
description: "Worktrees left by session 3ae9bb46-993d-4181-bf61-71e189463e09 would lose data if removed"
item: W-153
filed: 2026-10-30
filed_by: "now-ledger.py (Claude Code)"
updated: 2026-10-30
updated_by: "now-ledger.py (Claude Code)"
ball: none
---

# W-153 — Worktrees left by session 3ae9bb46-993d-4181-bf61-71e189463e09 would lose data if removed

## At a glance

| what | state |
|---|---|
| Worktrees left by session 3ae9bb46-993d-4181-bf61-71e189463e09 would lose data if removed | ✅ moot — landed |

Filed at the end of session `3ae9bb46-993d-4181-bf61-71e189463e09`.

These worktrees were left untouched. Rule per worktree: **keep** (merge it or commit the files; the next cleanup removes it) or **drop** (only then may an agent force the removal and delete the unmerged branch).

- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/handoff-20261030-050518` — HEAD 99ecf4c4 is not in main

## Closed 2026-10-30 (Claude Code, session 3ae9bb46)

The one worktree, `handoff-20261030-050518`, was Cowork's unlanded handoff (W-152, W-153's own row, the W-126 ruling, the queue regroup). `main` was merged into it, its two conflicts (`work/NOW.md`, the open-items count) resolved, and it landed as `4afabc5b`; lockmaster then removed the worktree. Nothing was dropped, so there is nothing for the decider to choose.

## Record affected

No SR affected — the rule is SR-WORK-CLEANUP.
