---
type: Handoff
name: W-97
description: "Arpit, 2026-10-27 (W-36 gap G3): a first-party GitHub Action for codelaw."
item: W-97
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: arpit
---

# W-97 — A first-party GitHub Action

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `action.yml` (composite) at this repo's root: install codelaw at a pinned version (npm or PyPI), run `check --format github`, optionally `--format sarif` + upload to code scanning, with `--changed-since` on pull requests | `uses: arpitarya/codelaw@v0.x` in one line | ✅ 2026-10-27 |
| 2 | Only ratified actions inside (D-30); pinned by SHA (W-18) | L11 | ✅ 2026-10-27 — it uses **no** action; the SARIF upload is the REST API via `gh` (the decider confirms the route) |
| 3 | `init --ci github` writes a workflow that uses it; docs in USAGE | adoption | ✅ 2026-10-27 |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: Biome (`setup-biome`) and Ruff (`ruff-action`) ship one ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## Definition of done

- A workflow in a throwaway repo using the Action shows annotations and uploads SARIF; a version input pins the engine. ⏳ — waits on the decider (below): the released codelaw 0.2.0 has no `--format github`, so the proof needs the next release tag.

## Built (2026-10-27, Claude Code)

- **`action.yml`** — composite; inputs `version` (default written by `scripts/version.py`, so tag `v<x>` installs `<x>`), `registry` npm|pypi, `package`, `working-directory`, `args`, `changed-since`, `sarif`, `upload-sarif`, `fail-on-error`, `token`; outputs `exit-code`, `sarif-file`. Annotation and SARIF paths are rewritten from the project folder to the repo root. No `uses:` inside (L11; `crates/codelaw-cli/tests/ci_action.rs` holds it).
- **CI:** `ci.yml` job `action` runs `uses: ./` on `fixtures/github-action` (one local rule, one `debugger`), installed from the run's linux wheel; asserts exit `1` and one SARIF result at `fixtures/github-action/src/app.js`.
- **`init --ci github`** (no `--autofix`) writes `.github/workflows/codelaw.yml` from `templates/ci/github/codelaw.yml` (SR-CLI-MCP decision 12a); `--ci gitlab` alone is refused.
- Records: SR-PACKAGES decision 15, SR-CLI-MCP 12a; USAGE, RELEASING, CHANGELOG.

## Left — the decider (inbox row)

1. **SARIF route:** keep the REST API via `gh` (built), or ratify `github/codeql-action/upload-sarif` in L11 §3.
2. **The DoD proof:** after the next release tag, a throwaway repo running the Action with `sarif: true` — annotations and a code-scanning upload.
3. **Marketplace + `v0`:** accept the Marketplace Developer Agreement and tick *Publish this Action* on the release; say whether a moving `v0` tag is kept.

## Record affected

SR-PACKAGES (a new distribution surface) · SR-CLI-MCP (`init --ci`).

## Hazards

- An Action in the main repo is versioned by the repo's tags — the release tags must stay the Action's contract.
