---
type: Handoff
name: W-50
description: "Arpit, 2026-10-25: codelaw init --git-hooks installs a blocking pre-commit and a non-blocking post-commit, plus a .pre-commit-hooks.yaml and husky/lefthook recipes."
item: W-50
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-50 — Git hooks — pre-commit and post-commit, installed in one command

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define the hooks in SR-CLI-MCP (`init`) and their change set in SR-BASELINE decision 3 (already `--staged`) | the record is the spec | ✅ SR-CLI-MCP decision 11 |
| 2 | **pre-commit** (blocks): `codelaw fix --staged` on fully staged files only, re-stage them, then `codelaw check --staged` | nothing bad is committed | ✅ |
| 3 | **post-commit** (never blocks): `codelaw exceptions --mine --expiring 7d` (W-46) and a one-line burn-down delta (W-47) | a reminder of what you owe | 🟡 hook and extension point (`.codelaw/hooks/post-commit.d/`) done; the two reminders wait on W-46 and W-47 |
| 4 | `codelaw init --git-hooks [pre-commit,post-commit]`: writes into `core.hooksPath` or `.git/hooks`, chains an existing hook, never overwrites | safe install | ✅ |
| 5 | Ship `.pre-commit-hooks.yaml` at the repo root (pre-commit framework) and husky/lefthook snippets in USAGE | whatever the team already uses | ✅ (`language: system`; see DoD 5) |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — shell scripts and an installer with written behaviour.

## Goal

- Works with no agent and no Node (L5 decision 4).
- **Open question for the decider:** does pre-commit run the `local` profile (console.log warns, commit goes through, CI fails later) or `ci` (blocked at commit)? Default proposed: `local`, as asked for laptops; a team flips it with `--profile ci` in the hook.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | a staged `debugger` blocks the commit under the chosen profile | ✅ proved with a staged fixture-rule violation (`fixture/py-no-print`); no profile is passed, see the open question |
| 2 · | a partly staged file is checked but never auto-fixed (no clobbering unstaged hunks) | ✅ |
| 3 · | an existing hook is chained, not replaced | ✅ |
| 4 · | post-commit prints and always exits 0 | ✅ |
| 5 · | pre-commit framework installs codelaw from the repo's tag | ❌ not met: the hooks are `language: system` (`codelaw` on `PATH`). The repo root has no `pyproject.toml`, `package.json` or buildable root crate for the framework to install from; the decider rules whether to add one |

## Blockers

post-commit's two reminders wait on **W-46** and **W-47**. The hook, its extension point and the pre-commit half landed (W-50, 2026-10-25). **Left:** when W-46 and W-47 land, each adds one executable to `.codelaw/hooks/post-commit.d/` that runs `codelaw exceptions --mine --expiring 7d` and the burn-down delta command, and `init --git-hooks` is taught to write them (or the post-commit script calls them directly).

## Record affected

SR-CLI-MCP (`init --git-hooks`) · SR-BASELINE decision 3 (the change set) · SR-WORK-OWNERSHIP is unaffected: this repo's own `commit-msg` guard stays as it is.

## Hazards

- `fix` on a partly staged file would commit unstaged hunks: skip fixing those files.
- Windows Git Bash: the hook is a POSIX `sh` script calling the binary.

## Reference

[pre-commit framework](https://pre-commit.com/#creating-new-hooks) · [lefthook](https://github.com/evilmartians/lefthook)
