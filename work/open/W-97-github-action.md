---
type: Handoff
name: W-97
description: "Arpit, 2026-10-27 (W-36 gap G3): a first-party GitHub Action for codelaw."
item: W-97
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-97 — A first-party GitHub Action

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `action.yml` (composite) at this repo's root: install codelaw at a pinned version (npm or PyPI), run `check --format github`, optionally `--format sarif` + upload to code scanning, with `--changed-since` on pull requests | `uses: arpitarya/codelaw@v0.x` in one line | ⏳ |
| 2 | Only ratified actions inside (D-30); pinned by SHA (W-18) | L11 | ⏳ |
| 3 | `init --ci github` writes a workflow that uses it; docs in USAGE | adoption | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: Biome (`setup-biome`) and Ruff (`ruff-action`) ship one ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## Definition of done

- A workflow in a throwaway repo using the Action shows annotations and uploads SARIF; a version input pins the engine.

## Record affected

SR-PACKAGES (a new distribution surface) · SR-CLI-MCP (`init --ci`).

## Hazards

- An Action in the main repo is versioned by the repo's tags — the release tags must stay the Action's contract.
