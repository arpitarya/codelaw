---
type: Handoff
name: W-146
description: "Arpit, 2026-10-30 (voice): pull-request hygiene — commit messages (Conventional Commits by default, fully configurable), branch names, PR size and PR description — locally, in CI and for agents."
item: W-146
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-146 — Pull-request hygiene: commits, branches, size, description

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Widen SR-ARCHITECTURE decision 14: git may also be read for commit messages in a range and the current branch name — read-only, offline | the data to check | ⏳ ruled |
| 2 | `git/commit-message` — Conventional Commits by default; `[commits]` configurable | readable history | ⏳ |
| 3 | `git/branch-name` — `[branches]` pattern, ticket, length | findable branches | ⏳ |
| 4 | `git/pr-size` — warn above 400 changed lines, lockfiles and generated files excluded | reviewable PRs | ⏳ |
| 5 | `git/pr-description` — title, ticket link, template sections, from the CI's own event data | complete PRs | ⏳ |
| 6 | A `commit-msg` hook (`init --git-hooks commit-msg`), CI over the PR's commits, the agent hook | checked before, during and after | ⏳ |

**Arpit, 2026-10-30 (voice):** *"Pull request hygiene is something I would go for."* · *"want the commit convention to be configurable."* · on the example `[commits]` and `[branches]` keys: *"All this looks good."* · on widening the git rule and the defaults (Conventional Commits, a 400-line warning): *"Yes."* → *"file it."*

**Model:** Opus (Claude Code).

## Goal — the configuration (the keys Arpit approved, final names settled in the record)

```toml
[commits]
preset = "conventional"          # conventional | angular | gitmoji | custom | off
types = ["feat", "fix", "docs", "refactor", "test", "chore", "perf", "ci", "build"]
scopes = []                      # empty = any scope
require_scope = false
subject_max = 72
subject_case = "lower"           # lower | sentence | any
require_body = false
breaking = ["!", "BREAKING CHANGE:"]
pattern = ""                     # a regex, only with preset = "custom"

[commits.ticket]
required = false
pattern = "[A-Z]+-[0-9]+"
place = "footer"                 # subject | footer | either

[commits.skip]
merges = true
reverts = true
fixups = true
authors = ["dependabot[bot]", "renovate[bot]"]

[branches]
pattern = ""                     # empty = branch names not checked
max_length = 60
protected = ["main", "develop", "release/*"]
skip_authors = ["dependabot[bot]", "renovate[bot]"]

[branches.ticket]
required = false
pattern = "[A-Z]+-[0-9]+"
match_commits = false

[pr]
max_lines = 400                  # warn above; 0 = off
exclude = ["**/*.lock", "**/package-lock.json", "**/pnpm-lock.yaml", "**/generated/**"]
title_pattern = ""
require_ticket = false
template_sections = []           # headings the description must fill
```

Defaults ruled: Conventional Commits on, a 400-line PR-size warning. Defaults chosen by the agent (say so in the record): ticket checks off, branch names unchecked until a pattern is set, PR description unchecked until a section or pattern is set.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · git | SR-ARCHITECTURE decision 14: from six uses to eight — `git log --format` over the `--changed-since` range, and the current branch (or the CI's branch variable) | ⏳ |
| 2 · rules | the four rules, each `manual` with steps (a message is never rewritten silently); the error shows the message, what is wrong and a corrected example | ⏳ |
| 3 · where | `init --git-hooks commit-msg` (template in `templates/git-hooks/`, L10); `codelaw check --changed-since` checks the PR's commits; the agent hook checks a commit an agent makes | ⏳ |
| 4 · PR data | GitHub (`GITHUB_EVENT_PATH`), GitLab (`CI_MERGE_REQUEST_TITLE`, `CI_MERGE_REQUEST_DESCRIPTION`); Bitbucket and Azure DevOps expose no description without an API call — title or branch only there, the rest reported as "not checked here", never fetched (L5) | ⏳ |
| 5 · config | SR-CONFIG `[commits]`, `[branches]`, `[pr]`; shareable through a pack | ⏳ |
| 6 · tests | each preset; skips; the hook; each CI's data, from fixtures | ⏳ |

## Blockers

None — both open points ruled 2026-10-30.

## Record affected

[SR-ARCHITECTURE](../../records/0108_architecture.md) decision 14 · SR-CONFIG · SR-CLI-MCP decision 11 (git hooks) · SR-RULE-PACKS · SR-AGENT-SURFACES (the hook).

## Hazards

- **A commit is not a file:** findings need a location — a logical one (`commit:<sha>`, SARIF `logicalLocations`; the hook's message file in hook mode). Decide in SR-ARCHITECTURE; no L3 change.
- Never network (L5): the PR description is read only from what the CI already put on disk or in the environment.
- Corrected at voice: the architecture record allows git for **six** things today, not three.

## Reference

W-50 (git hooks) · W-131 (ticket patterns, the same `[A-Z]+-\d+` shape)
