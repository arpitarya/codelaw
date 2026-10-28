---
type: Handoff
name: W-109
description: "Arpit, 2026-10-27/28: Bitbucket Code Insights output for codelaw — the report and SARIF each switchable; no init --ci bitbucket, no pipe (ruled 2026-10-28)."
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
| 1 | Records first: how codelaw runs in Bitbucket Pipelines, where its findings go, which outputs are switches | the spec before the code | ✅ [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decisions 4 (`--format bitbucket`) and 12b; [SR-PACKAGES](../../records/0107_packages.md) decision 15a |
| 2 | A Bitbucket pipe (or a documented step) that installs codelaw at a pinned version and runs `check` | one step in `bitbucket-pipelines.yml` | 🔄 ruled 2026-10-28: **no pipe**, and no generated step — USAGE shows the step to copy, with the two switches as its variables |
| 3 | Findings to Bitbucket **Code Insights** — a report and its annotations on the pull request | the Bitbucket twin of GitHub's annotations | ✅ `--format bitbucket` + `curl` through the pipeline proxy; proven locally, live run left |
| 4 | **The SARIF part configurable:** SARIF output on or off, and the Code Insights upload on or off, each its own variable | teams pick what their pipeline needs | ✅ `CODELAW_SARIF` (default off), `CODELAW_CODE_INSIGHTS` (default on) |
| 5 | `codelaw init --ci bitbucket` writes the pipeline from a template; USAGE documents it | adoption | ❌ ruled 2026-10-28: **remove** — the `init --ci bitbucket` code, `templates/ci/bitbucket/`, its tests and records lines come out |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** on W-97, asked whether every CI has `gh`: *"Make sure it works with Bitbucket as well. I don't want any problems with it."* — told a GitHub Action runs only on GitHub, so Bitbucket needs its own piece — *"File it as a work item and make it configurable. The serif part."*

**Arpit, 2026-10-28 (voice; written on *"file it."*):** *"No need to build out that Bitbucket CI itself. Remove that. Just have serif and code and sites configurable."* (the transcript's "serif" is SARIF, "code and sites" Code Insights) · *"Remove that whole work item."* — asked whether that meant closing W-109 or taking the Bitbucket output out too: *"It should just be code insights output."* · asked whether the live Bitbucket proof stays: *"Perfect."* (read as yes). Earlier, in text: *"codelaw init --ci bitbucket no need to build this part, but do build out the configurable insights part."*

**Model:** Sonnet — a removal, records first.

Why: W-97's Action covers GitHub only; Bitbucket Pipelines has no Actions and reads findings through Code Insights, not SARIF code scanning.

## Definition of done

- A pipeline in a throwaway Bitbucket repo, from the step USAGE shows, fails on one `debugger`, shows a Code Insights report with its annotation, and writes a SARIF file only when switched on.
- With the Code Insights upload off, the step still checks and fails the build.

## Built (2026-10-28, Claude Code)

- **`--format bitbucket`** (`crates/codelaw-report/src/ci.rs`, on `check`, `fix` and `format --check`): line 1 the report, each line after it one `POST` body of at most 100 annotations; errors first; past Code Insights' 1000-annotation cap the report's `details` says `Showing 1000 of <n> findings` (the hazard below). Every field and enum checked against Bitbucket Cloud's published schema (`report`, `report_annotation`, `report_data` in `api.bitbucket.org/swagger.json`) by `ci.rs`'s tests.
- **`codelaw init --ci bitbucket`** (`crates/codelaw-cli/src/ci.rs`) writes `bitbucket-pipelines.yml` from `templates/ci/bitbucket/bitbucket-pipelines.yml` (L10): `node:22`, `npm install --global codelaw@<version>`, `codelaw check`, then `--format bitbucket` sent with `curl` through `http://localhost:29418` (no token): `DELETE`, `PUT` the report `codelaw`, `POST` each batch. Switches: `CODELAW_CODE_INSIGHTS`, `CODELAW_SARIF`, `CODELAW_FAIL_ON_ERROR`, `CODELAW_DIR`, `CODELAW_ARGS`, `CODELAW_CHANGED_SINCE` (a PR passes `--changed-since origin/<destination>`). A foreign `bitbucket-pipelines.yml` is never overwritten; `--autofix` with it is refused.
- **Proven locally** (`crates/codelaw-cli/tests/ci_bitbucket.rs`): the step's own script, `curl` and `npm` stubbed, on a project with one `debugger` — exit 1, the report `FAILED` and its one annotation (`proj/src/app.js:2`, `HIGH`) through the proxy, no SARIF; with `CODELAW_SARIF=true` and the upload off: one SARIF result, no `curl`, still exit 1; `CODELAW_FAIL_ON_ERROR=false`: exit 0.

## Left (ruled 2026-10-28)

1. **Agent — remove the generator:** `codelaw init --ci bitbucket` (its branch in `crates/codelaw-cli/src/ci.rs`), `templates/ci/bitbucket/` and its `templates/README.md` row, the init half of `crates/codelaw-cli/tests/ci_bitbucket.rs`; records first — SR-CLI-MCP decision 12b and SR-PACKAGES decision 15a rewritten, USAGE §CI shows the step as a snippet to copy. **Keep** `--format bitbucket` and its tests.
2. **The switches stay configurable** — `CODELAW_CODE_INSIGHTS` and `CODELAW_SARIF` as the snippet's variables, a repository variable overriding each (the agent's reading of *"just have serif and code [insights] configurable"*; say so to the decider if it needs a codelaw-side switch instead).
3. **The live proof (Arpit, after v0.3.0):** the DoD's throwaway Bitbucket repo, from USAGE's snippet; an inbox row once v0.3.0 is out.

## Open questions (for step 1) — answered

- **Pipe or step:** built as a step; the pipe is the decider's (inbox). A Bitbucket pipe is a Docker image (`pipe.yml` + `Dockerfile`), and its base image would be a dependency — an L11 row the decider ratifies. A documented step that installs codelaw with `npx`/`pip` in the pipeline's own image needs none. Put the fork to the decider before building.
- **The upload:** done — Code Insights' REST API (reports and annotations) with `curl` and the pipeline's own auth proxy — no tool, as W-95's `curl` ruling.
- **The template:** built, then ruled out 2026-10-28 — codelaw writes no Bitbucket file, so L10 needs none.

## Record affected

SR-CLI-MCP decisions 4 (`--format bitbucket`, kept) and 12b (removed) · SR-PACKAGES decision 15a (no pipe, no generated step).

## Hazards

- Code Insights caps annotations per report; a large first run must truncate honestly and say so in the report.
