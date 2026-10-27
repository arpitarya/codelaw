---
type: Handoff
name: W-95
description: "Arpit, 2026-10-27 (W-36 gap G2): publish the built VS Code extension on the Marketplace and Open VSX."
item: W-95
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-95 — Publish the VS Code extension — Marketplace and Open VSX

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Package `packaging/vscode` per platform with its bundled binary (or the npm binary lookup); `vsce package` / `ovsx` dry runs in CI | publishable artifacts | ⏳ |
| 2 | Arpit creates the Marketplace publisher and the Open VSX namespace, and stores the tokens (or a trusted-publishing equivalent) as GitHub secrets in the `vscode` environment | the right to publish | ⏳ then 🔴 Arpit |
| 3 | `publish.yml` publishes the extension on a Release, after the engine | one release, every surface | ⏳ |
| 4 | README / USAGE: install from the Marketplace; the preview note drops "the editor extension's Marketplace listing comes later" | users can install it | ⏳ |

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: Biome, Ruff and oxc each ship first-party extensions (VS Code, Zed, JetBrains); codelaw's is built and unpublished ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## Definition of done

- The extension installs from both registries, finds the engine, and shows diagnostics and fix-on-save in a clean VS Code and VSCodium.

## Record affected

SR-EDITOR · SR-PACKAGES (a new published artifact) · RELEASING.md.

## Hazards

- Marketplace tokens are long-lived secrets; scope them to the `vscode` environment with a `v*` tag rule, as `npm`/`pypi` are.
- Zed and JetBrains are later items, not this one.
