---
type: Handoff
name: W-45
description: "Arpit, 2026-10-25: findings inline on PRs without a SARIF upload: --format github, gitlab, junit and checkstyle."
item: W-45
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5) — docket"
ball: agent
---

# W-45 — CI report formats — GitHub, GitLab, JUnit, Checkstyle

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add the four formats to SR-CLI-MCP decision 4 | the record is the spec | ✅ |
| 2 | `github`: workflow commands `::error file=…,line=…,col=…,title=<rule>::<msg>` (warn → `::warning`, info → `::notice`) | annotations on the PR diff | ✅ |
| 3 | `gitlab`: Code Quality JSON (`check_name`, `description`, `severity`, `fingerprint`, `location.path`, `lines.begin`) | the MR widget | ✅ |
| 4 | `junit` and `checkstyle` XML | Jenkins, Azure DevOps, Bitbucket, SonarQube | ✅ |
| 5 | Docs: one CI snippet per platform in USAGE | copy-paste setup | ✅ |

**Arpit, 2026-10-25:** *"Watch mode, let's build it. Then CI report formats, let's build it. Exception report, that is code law exceptions, expiring 30 days, group added by … build it. Baseline burn down over commits, build it. Code law migrate from ESLint, flat config, Ruff, style lint, as well as biome … Remove OXLint. Dev server plugin for RSpack, Vite, Webpack, Next.js. Pre-commit hook, post-commit hook, create work items for all these."*

**Model:** Sonnet — four fixed output schemas with tests against each schema.

## Goal

- `--format github|gitlab|junit|checkstyle` on `check`, `fix` and `format --check`.
- Every format carries the fix class and steps where the schema has room (L4).
- **Fingerprints** are stable: a hash of rule, file, message and the occurrence's index among that rule's findings in the file — never the line, so moving code doesn't re-open findings.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | each format validated against its published schema/XSD in a test | ✅ |
| 2 · | GitHub: a real run on this repo shows annotations on a PR | ⏳ PR open: [#3](https://github.com/arpitarya/codelaw/pull/3), waiting for Arpit's look |
| 3 · | the same input gives byte-identical output (L5) | ✅ |
| 4 · | exit codes unchanged | ✅ |

## What is left

- DoD 2: **PR [#3](https://github.com/arpitarya/codelaw/pull/3) is open** (draft; its run did put the 5 annotations on `a.js`/`a.py` per the check-runs API; branch `demo/W-45-annotations-v2`, workflow `w45-annotations-demo`). The one step left is Arpit's: open Files changed, confirm the 2 errors and 3 warnings show inline on `w45-demo/a.js` and `a.py`, say so here; then close the PR and delete the branch. (Was: push, open a PR with `codelaw check --format github` in the workflow, and look at the annotations.)
- JUnit/Checkstyle schema checks need `xmllint` (libxml2); the test skips loudly without it. Checkstyle publishes no schema, so `checkstyle.xsd` is its format description written down.


## Rulings

**Arpit, 2026-10-26 (docket):** *"Claude Code opens the PR"*. Claude Code, on the Mac with `gh`, pushes a branch whose workflow runs `codelaw check --format github` on a file with known findings, opens the PR, and links it in this file; Arpit looks at the annotations and says so — then DoD 2 is met.

**Model:** Claude Code, Sonnet — it needs `git push` and `gh`, which this Cowork bridge lacks.

## Blockers

**The decider's move (2026-10-25):** DoD 2 needs a real pull request with `codelaw check --format github` in a workflow, so the annotations can be seen on GitHub — an agent cannot push or open one here.

None.

## Record affected

SR-CLI-MCP decision 4 (amended: the formats, the `format --check` mapping, the fingerprint, the tests).

## Hazards

- GitHub caps annotations per step (about 10 per type shown inline): print a summary line too.
- GitLab severity names differ (`info|minor|major|critical|blocker`): map error → major, warn → minor, info → info.

## Reference

[GitHub workflow commands](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands) · [GitLab Code Quality](https://docs.gitlab.com/ci/testing/code_quality/)
