---
type: Handoff
name: W-09
description: "Arpit, 2026-10-20: the LSP and editor extension SR-EDITOR and L6 decision 3 require."
item: W-09
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-09 — Build the language server and the VS Code extension

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Add `codelaw-lsp` and `codelaw lsp`: diagnostics, code actions for fixes, formatting | the editor sees what `check` sees | ✅ diagnostics, quick fixes, fix all, hover · ⏳ formatting (W-07) |
| 2 | Convert positions to UTF-16 at the LSP boundary only | SR-ARCHITECTURE decision 13 | ✅ |
| 3 | Publish the thin `codelaw.codelaw` extension (Marketplace + Open VSX) | one-click install | ✅ built in `packaging/vscode` · ⏳ packaging and publishing (release pipeline, decider) |
| 4 | Extend the CLI/MCP parity test to the LSP | three doors, one answer | ✅ CLI vs LSP · ⏳ MCP joins when W-08 lands |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: incremental re-parse, UTF-16 mapping and the never-write rule are easy to get subtly wrong, and the editor is where users notice.

## Goal

`codelaw lsp` gives the same diagnostics and fixes as `check`/`fix`, never writes files itself, and the extension runs the project's own codelaw first.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · server | `codelaw lsp`: publishDiagnostics, codeAction (auto/suggest fixes), formatting via W-07. | ✅ diagnostics and code actions · ⏳ formatting: `crates/codelaw-lsp/src/format.rs` is the one function to wire when W-07 lands; the capability is not advertised until then |
| 2 · positions | A test on multi-byte and astral characters round-trips UTF-16 positions. | ✅ `position.rs` unit tests and `tests/server.rs` |
| 3 · no writes | The server returns `WorkspaceEdit`s only (SR-ARCHITECTURE decision 12). | ✅ tested: the repo is byte-identical after fixes; the crate has no write path |
| 4 · extension | `codelaw.codelaw` finds `node_modules/.bin`, then `.venv/bin`, then a bundled binary (SR-EDITOR). | ✅ `packaging/vscode/src/binary.ts`, tested under `node --test` |
| 5 · gate | `scripts/law-gate.py` no longer lists `lsp`. | ✅ |

## Left, and why

- **Formatting** waits for W-07's `codelaw-format`.
- **Not built yet:** pull and workspace diagnostics, project rules on save (no graph yet), "Ignore here with reason", the warm socket (SR-ARCHITECTURE decision 21), incremental re-parse (W-15; every change re-checks the document, full sync).
- **Release:** the platform-specific `.vsix` packages (bundled binary in `bin/`, `LICENSE`, the one version stamped over `0.0.0`) and their Marketplace/Open VSX publish join the release pipeline; publishing is the decider's.

## Blockers

None for the server. Formatting on save needs W-07.

## Record affected

[SR-EDITOR](../../records/0114_editor.md) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 12, 13, 23, 24 · [SR-PACKAGES](../../records/0107_packages.md) decisions 1 and 6.

## Hazards

- **The extension is a new published artifact:** it joins the release pipeline and its version (L6 decision 9).

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
