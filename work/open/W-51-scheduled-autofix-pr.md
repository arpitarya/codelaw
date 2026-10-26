---
type: Handoff
name: W-51
description: "Arpit, 2026-10-25: codelaw init --ci github --autofix weekly writes a workflow that runs fix, format and baseline prune on a schedule and keeps one PR open with the result."
item: W-51
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-51 — A scheduled auto-fix PR — one open PR a week with every safe fix

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Define `init --ci github|gitlab --autofix daily|weekly` in SR-CLI-MCP | the record is the spec | ✅ |
| 2 | The workflow: checkout → install codelaw (pinned version) → `fix` (auto only) → `format` → `baseline prune` → stop if no diff | only safe, verified changes | ✅ |
| 3 | Commit to one branch `codelaw/autofix`, force-push, `gh pr create` or update the open PR | never more than one PR | ✅ |
| 4 | PR body from `fix --format json`: fixes per rule and file, exceptions expiring in 30 days (W-46), the week's burn-down (W-47), the deps snapshot diff (W-52) | reviewers see why | ✅ |
| 5 | `--unsafe` opt-in: `suggest` fixes in a second branch `codelaw/autofix-suggest` | riskier fixes kept apart | ✅ |
| 6 | GitLab twin: a scheduled pipeline and a merge request through the API | same flow on GitLab | ✅ written and tested for content; never run on a GitLab |

**Arpit, 2026-10-25:** *"schedule fix PR, create a work item for it. I like the approach."* (on the design given in chat the same day)

**Model:** Sonnet — a workflow template and an installer, tested on a fixture repo.

## Goal

- Permissions: `contents: write`, `pull-requests: write`; nothing else.
- `--as-of "$(date +%F)"` in the job, so expiries are judged on the run's real day.
- It can only lower baseline counts (SR-BASELINE decision 5); a raised count fails its own CI.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | a fixture repo with fixable findings gets one PR; a second run updates the same PR | ⏳ needs a real GitHub repo: the create-or-edit logic is in the template, unrun |
| 2 · | no fixable findings → no PR, job green | ✅ the job exits 0 before any branch when `git status --porcelain` is empty (template); not run on a runner |
| 3 · | the PR's own CI runs and must pass | ⏳ needs a real repo and `CODELAW_AUTOFIX_TOKEN` |
| 4 · | `suggest` fixes appear only with `--unsafe`, in their own PR | ✅ `tests/ci_autofix.rs` |
| 5 · | `actionlint` clean | ✅ the test runs it on both variants |

## Blockers

None. The burn-down section (W-47) is in. The exceptions section (W-46) is in, unconditional, judged with `--as-of`. The deps-snapshot section (W-52) is not written: the job runs no networked command (L5), so the snapshot cannot change in it.

**Left (human or runner):** run the workflow once in a throwaway GitHub repo with fixable findings (DoD 1, 3).

## Record affected

SR-CLI-MCP (`init --ci … --autofix`).

## Hazards

- `GITHUB_TOKEN`-created PRs don't trigger other workflows by default: document a GitHub App token or `workflow_dispatch` to get CI on the PR.
- Branch protection may refuse force-push to `codelaw/autofix`: exclude that branch.

## Reference

[GitHub: triggering a workflow from a workflow](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow) · [W-46](W-46-exceptions-report.md) · [W-47](W-47-baseline-burn-down.md)
