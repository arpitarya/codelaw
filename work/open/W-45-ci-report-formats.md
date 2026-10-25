---
type: Handoff
name: W-45
description: "Arpit, 2026-10-25: findings inline on PRs without a SARIF upload: --format github, gitlab, junit and checkstyle."
item: W-45
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), parallel merge"
ball: arpit
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
| 2 · | GitHub: a real run on this repo shows annotations on a PR | ⏳ |
| 3 · | the same input gives byte-identical output (L5) | ✅ |
| 4 · | exit codes unchanged | ✅ |

## What is left

- DoD 2: push, open a PR with `codelaw check --format github` in the workflow, and look at the annotations. Nothing else.
- JUnit/Checkstyle schema checks need `xmllint` (libxml2); the test skips loudly without it. Checkstyle publishes no schema, so `checkstyle.xsd` is its format description written down.

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
