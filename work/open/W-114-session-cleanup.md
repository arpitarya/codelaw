---
type: Handoff
name: W-114
description: "Worktrees left by session 2df06f6d-4680-4c59-a610-65a9126be306 would lose data if removed"
item: W-114
filed: 2026-10-28
filed_by: "now-ledger.py (Claude Code)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-114 — Worktrees left by session 2df06f6d-4680-4c59-a610-65a9126be306 would lose data if removed

## At a glance

| what | state |
|---|---|
| Worktrees left by session 2df06f6d-4680-4c59-a610-65a9126be306 would lose data if removed | ⏳ |

**Arpit, 2026-10-29 (voice docket; written on *"file it."*):** *"Drop all five."* — a Claude Code session on the Mac runs the commands below (`--force`, `-D`: permitted by this ruling, SR-WORK-CLEANUP decision 3).

Filed at the end of session `2df06f6d-4680-4c59-a610-65a9126be306`.

These worktrees were left untouched. Rule per worktree: **keep** (merge it or commit the files; the next cleanup removes it) or **drop** (only then may an agent force the removal and delete the unmerged branch).

- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/W-109-ruling` — HEAD cac52d1e is not in main
- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/W-110` — 491 modified, staged, deleted or untracked files
- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/W-112` — HEAD 99db9f2b is not in main
- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/W-90` — HEAD 52be3382 is not in main; ignored `.claude/.disk-usage` is not build output
- `/Users/arpitarya/my_programs/codelaw/.claude/worktrees/W-96` — HEAD 0211a57e is not in main

## Checked 2026-10-29 (Cowork, read-only) — nothing would be lost

| worktree | why the script held it | what it actually holds | recommend |
|---|---|---|---|
| `W-109-ruling` (64 MB) | HEAD `cac52d1` not in `main` | `8f83229`, the removal of `init --ci bitbucket`, landed on `main` as `ab040e8`; files differ only where `main` moved on | drop |
| `W-110` (65 MB) | 491 changed files | all 491 under `.fux/index/` — the regenerated Fux index, not work; its branch is in `main` | drop |
| `W-112` (47 MB) | HEAD `99db9f2` not in `main` | `e401b13` landed as `8e5f9b9`; one file differs, where `main` moved on | drop |
| `W-90` (49 MB) | HEAD `52be338` not in `main` | W-90 closed and archived; `archive/DEPENDENCIES.md` and L11 §3 are in `main` | drop |
| `W-96` (45 MB) | HEAD `0211a57` not in `main` | W-96 closed and archived; `crates/codelaw-report/src/agent_env.rs` is in `main` | drop |

Also under `.claude/worktrees/`: `worklog-2df06f6d` (8 KB, no git worktree) — remove with the rest.

**W-111 is folded in here** (closed 2026-10-29): it asked the same question for `W-90` and `W-96`.

**On "drop all five" (on the Mac, main checkout):**

```bash
for w in W-109-ruling W-110 W-112 W-90 W-96; do git worktree remove --force .claude/worktrees/$w && git branch -D lm/$w; done
rm -rf .claude/worktrees/worklog-2df06f6d && git worktree prune
```

## Record affected

No SR affected — the rule is SR-WORK-CLEANUP.
