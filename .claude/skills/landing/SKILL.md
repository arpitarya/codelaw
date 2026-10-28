---
name: landing
description: How a change reaches main in a repository that uses lockmaster — from Claude Code or from Cowork. Use whenever you have edited files and they need to be committed, merged or landed on main; when a commit is refused or git says "Operation not permitted"; when work in a worktree is done; or before telling the decider anything about commits, worktrees, lock files or merging.
---

# Landing on main — lockmaster

`main` moves only through lockmaster. You never need the decider for the
mechanics of committing: lockmaster either lands your edits or hands them to
a session that can. **Do not ask the decider about commits, lock files,
worktrees, `index.lock` or "Operation not permitted". Do not try another git
command to get around a refusal.**

`lockmaster` below means `lockmaster` if it is on the PATH, else
`PYTHONPATH=<the lockmaster checkout>/src python3 -m lockmaster` (beside this
repository, e.g. `../lockmaster/src`).

## Cowork, or any session editing main's checkout directly

When your edits are done, name the files you changed and run, from the
repository's root:

```bash
lockmaster save -m "<item>: what changed and why" <path> <path> …
```

- **This surface can commit** → it lands now, like any change (checks run,
  one commit on `main`).
- **It cannot** (the Cowork bridge cannot remove files, or sees the Mac's
  repository from another path) → it prints `handed off …` and stops. That is
  success: the next Claude Code session in this repository lands it at its
  start or at the end of its next turn. Say one line — "handed to Claude Code
  to land" — and move on.

Name only the files *you* changed; another session's uncommitted edits may sit
in the same checkout. `lockmaster can-commit` says which case you are in.
Never run `git commit`, `git worktree`, `git worktree prune` or `rm` on git's
files from the Cowork bridge.

## Claude Code

- Start work as a change: `lockmaster start <item> --paths <path> …`, edit in
  its worktree. **At the end of every turn the kit's `auto-land.py` hook lands
  what the change holds** — the change stays open, its worktree follows `main`,
  so conflicts stay small. `lockmaster submit -m "…"` when the item is finished
  lands it and removes the worktree.
- A `claude --worktree` session's worktree is adopted and landed the same way.
- If a landing is held (a check failed, a conflict), the hook tells you why:
  fix it in the worktree; it lands at the end of the next turn. It is not the
  decider's problem unless you cannot fix it.
- Edits made straight in main's checkout: the hook reminds you once —
  `lockmaster save -m "…" <paths>`.
- Handoffs from Cowork are landed by the hook (`lockmaster pickup`). If one
  goes red, the hook says why: fix the files where they are, in main's
  checkout — the end of the next turn retries the same change with them.

`lockmaster queue` shows what is waiting, held, handed off, landed or red.
`[land] auto = "off"` in `lockmaster.toml` turns end-of-turn landing off.
