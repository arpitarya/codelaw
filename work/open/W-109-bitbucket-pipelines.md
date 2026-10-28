---
type: Handoff
name: W-109
description: "Arpit, 2026-10-27 (voice docket): codelaw in Bitbucket Pipelines as well as GitHub — a pipe, init --ci bitbucket, findings in Code Insights, the SARIF part configurable."
item: W-109
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-109 — Bitbucket Pipelines

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first: how codelaw runs in Bitbucket Pipelines, where its findings go, which outputs are switches | the spec before the code | ⏳ |
| 2 | A Bitbucket pipe (or a documented step) that installs codelaw at a pinned version and runs `check` | one step in `bitbucket-pipelines.yml` | ⏳ |
| 3 | Findings to Bitbucket **Code Insights** — a report and its annotations on the pull request | the Bitbucket twin of GitHub's annotations | ⏳ |
| 4 | **The SARIF part configurable:** SARIF output on or off, and the Code Insights upload on or off, each its own variable | teams pick what their pipeline needs | ⏳ |
| 5 | `codelaw init --ci bitbucket` writes the pipeline from a template; USAGE documents it | adoption | ⏳ |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** on W-97, asked whether every CI has `gh`: *"Make sure it works with Bitbucket as well. I don't want any problems with it."* — told a GitHub Action runs only on GitHub, so Bitbucket needs its own piece — *"File it as a work item and make it configurable. The serif part."*

**Model:** Opus — records first, and an L11 question likely.

Why: W-97's Action covers GitHub only; Bitbucket Pipelines has no Actions and reads findings through Code Insights, not SARIF code scanning.

## Definition of done

- A pipeline in a throwaway Bitbucket repo, from `codelaw init --ci bitbucket`, fails on one `debugger`, shows a Code Insights report with its annotation, and writes a SARIF file only when switched on.
- With the Code Insights upload off, the step still checks and fails the build.

## Open questions (for step 1)

- **Pipe or step:** a Bitbucket pipe is a Docker image (`pipe.yml` + `Dockerfile`), and its base image would be a dependency — an L11 row the decider ratifies. A documented step that installs codelaw with `npx`/`pip` in the pipeline's own image needs none. Put the fork to the decider before building.
- **The upload:** Code Insights' REST API (reports and annotations) with `curl` and the pipeline's own auth proxy — no tool, as W-95's `curl` ruling.
- **The template:** under `templates/ci/bitbucket/` (L10), beside W-97's `templates/ci/github/`.

## Record affected

SR-PACKAGES (the pipe or step, beside decision 15's Action) · SR-CLI-MCP (`init --ci bitbucket`, decision 12a) · L11 if a pipe's base image is chosen.

## Hazards

- Code Insights caps annotations per report; a large first run must truncate honestly and say so in the report.
