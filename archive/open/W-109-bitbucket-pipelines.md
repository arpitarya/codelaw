---
type: Handoff
name: W-109
description: "Arpit, 2026-10-27/28: Bitbucket Code Insights output for codelaw — the report and SARIF each switchable; no init --ci bitbucket, no pipe (ruled 2026-10-28)."
item: W-109
filed: 2026-10-28
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Claude Code (Opus 5.5), 2026-10-28: closed on the decider's ruling, the live proof to BACKLOG B-061"
ball: closed
---

# W-109 — Bitbucket Pipelines

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first: how codelaw runs in Bitbucket Pipelines, where its findings go, which outputs are switches | the spec before the code | ✅ [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decisions 4 (`--format bitbucket`) and 12b; [SR-PACKAGES](../../records/0107_packages.md) decision 15a |
| 2 | A Bitbucket pipe (or a documented step) that installs codelaw at a pinned version and runs `check` | one step in `bitbucket-pipelines.yml` | ✅ ruled 2026-10-28: **no pipe**, no generated step — [USAGE](../../USAGE.md#bitbucket-pipelines) §Bitbucket Pipelines shows the step to copy (`codelaw@0.3.0` pinned), the switches as its variables; `crates/codelaw-cli/tests/usage_bitbucket.rs` runs that block's script |
| 3 | Findings to Bitbucket **Code Insights** — a report and its annotations on the pull request | the Bitbucket twin of GitHub's annotations | ✅ `--format bitbucket` + `curl` through the pipeline proxy; proven locally, live run left |
| 4 | **The SARIF part configurable:** SARIF output on or off, and the Code Insights upload on or off, each its own variable | teams pick what their pipeline needs | ✅ `CODELAW_SARIF` (default off), `CODELAW_CODE_INSIGHTS` (default on) |
| 5 | `codelaw init --ci bitbucket` writes the pipeline from a template; USAGE documents it | adoption | ✅ removed 2026-10-28 (ruled): the `init --ci bitbucket` code, `templates/ci/bitbucket/` and its registry rows, the init test, SR-CLI-MCP 12b / SR-PACKAGES 15a / SR-LAW-10 3 rewritten; `--ci bitbucket` is now an unknown value (exit 2) |

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** on W-97, asked whether every CI has `gh`: *"Make sure it works with Bitbucket as well. I don't want any problems with it."* — told a GitHub Action runs only on GitHub, so Bitbucket needs its own piece — *"File it as a work item and make it configurable. The serif part."*

**Arpit, 2026-10-28 (voice; written on *"file it."*):** *"No need to build out that Bitbucket CI itself. Remove that. Just have serif and code and sites configurable."* (the transcript's "serif" is SARIF, "code and sites" Code Insights) · *"Remove that whole work item."* — asked whether that meant closing W-109 or taking the Bitbucket output out too: *"It should just be code insights output."* · asked whether the live Bitbucket proof stays: *"Perfect."* (read as yes). Earlier, in text: *"codelaw init --ci bitbucket no need to build this part, but do build out the configurable insights part."*

**Model:** Sonnet — a removal, records first.

Why: W-97's Action covers GitHub only; Bitbucket Pipelines has no Actions and reads findings through Code Insights, not SARIF code scanning.

## Definition of done

- A pipeline in a throwaway Bitbucket repo, from the step USAGE shows, fails on one `debugger`, shows a Code Insights report with its annotation, and writes a SARIF file only when switched on.
- With the Code Insights upload off, the step still checks and fails the build.

## Built (2026-10-28, Claude Code)

- **`--format bitbucket`** (`crates/codelaw-report/src/ci.rs`, on `check`, `fix` and `format --check`): line 1 the report, each line after it one `POST` body of at most 100 annotations; errors first; past Code Insights' 1000-annotation cap the report's `details` says `Showing 1000 of <n> findings` (the hazard below). Every field and enum checked against Bitbucket Cloud's published schema (`report`, `report_annotation`, `report_data` in `api.bitbucket.org/swagger.json`) by `ci.rs`'s tests. **Kept** by the ruling.
- **The step, documented** ([USAGE.md](../../USAGE.md#bitbucket-pipelines) §Bitbucket Pipelines, after the ruling): `node:22`, `npm install --global codelaw@0.3.0` (pinned; the first release with `--format bitbucket`), `codelaw check`, then `--format bitbucket` sent with `curl` through `http://localhost:29418` (no token): `DELETE`, `PUT` the report `codelaw`, `POST` each batch. Variables: `CODELAW_CODE_INSIGHTS` (default `true`), `CODELAW_SARIF` (default `false`), plus `CODELAW_FAIL_ON_ERROR`, `CODELAW_DIR`, `CODELAW_ARGS`, `CODELAW_CHANGED_SINCE`; a repository variable overrides each. Spec: SR-CLI-MCP decision 12b.
- **Proven locally** (`crates/codelaw-cli/tests/usage_bitbucket.rs`, was `ci_bitbucket.rs`): the YAML block under USAGE's `### Bitbucket Pipelines`, its script run with `curl` and `npm` stubbed, on a project with one `debugger` — exit 1, the report `FAILED` and its one annotation (`proj/src/app.js:2`, `HIGH`) through the proxy, no SARIF; with `CODELAW_SARIF=true` and the upload off: one SARIF result, no `curl`, still exit 1; `CODELAW_FAIL_ON_ERROR=false`: exit 0. And `init --ci bitbucket` exits 2 (`--ci is \`github\` or \`gitlab\``) and writes nothing.
- **Removed (the ruling, 2026-10-28):** `codelaw init --ci bitbucket` (its branch in `crates/codelaw-cli/src/ci.rs`, the refusal in `init.rs`), `templates/ci/bitbucket/bitbucket-pipelines.yml`, its `templates/README.md` row and `template.rs` registry and fixture entries, the init test; SR-CLI-MCP 12b, SR-PACKAGES 15a and SR-LAW-10 decision 3's example rewritten.

## Closed (2026-10-28)

**Arpit, 2026-10-28:** *"close where only you're left"* — every agent step is done; the only step left is the decider's live proof, so the item closes and that proof moves to [BACKLOG](../../work/BACKLOG.md) **B-061**, reopened after the v0.3.0 release.

Evidence: commits `2422496` (`--format bitbucket` and the first step) and `ab040e8` (the ruling applied: `init --ci bitbucket` removed, USAGE §Bitbucket Pipelines the step to copy); the test `crates/codelaw-cli/tests/usage_bitbucket.rs` runs USAGE's block.

## Left (ruled 2026-10-28) — moved to BACKLOG B-061

1. **The live proof (Arpit, after v0.3.0):** the DoD's throwaway Bitbucket repo, from USAGE's step; an inbox row once v0.3.0 is out (0.2.0 has no `--format bitbucket`).
2. **The switches' reading (Arpit, to confirm):** they stay the step's variables, a repository variable overriding each — the agent's reading of *"just have serif and code [insights] configurable"*; say so if it needs a codelaw-side switch instead.

## Open questions (for step 1) — answered

- **Pipe or step:** built as a step; the pipe is the decider's (inbox). A Bitbucket pipe is a Docker image (`pipe.yml` + `Dockerfile`), and its base image would be a dependency — an L11 row the decider ratifies. A documented step that installs codelaw with `npx`/`pip` in the pipeline's own image needs none. Put the fork to the decider before building.
- **The upload:** done — Code Insights' REST API (reports and annotations) with `curl` and the pipeline's own auth proxy — no tool, as W-95's `curl` ruling.
- **The template:** built, then ruled out 2026-10-28 — codelaw writes no Bitbucket file, so L10 needs none.

## Record affected

SR-CLI-MCP decisions 4 (`--format bitbucket`, kept) and 12b (removed) · SR-PACKAGES decision 15a (no pipe, no generated step).

## Hazards

- Code Insights caps annotations per report; a large first run must truncate honestly and say so in the report.
