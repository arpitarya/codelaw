---
type: Handoff
name: W-136
description: "Arpit, 2026-10-30 (voice): a finding about a plain-text config file (.nvmrc and the like) shows on that file's line as well as on the manifest — L3 amended for findings only."
item: W-136
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5) — closed"
ball: agent
---

# W-136 — L3: findings on plain-text config files

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L3: a closed list of plain-text config files may carry findings — never linted or formatted as a language | a warning can sit on the `.nvmrc` line | ✅ 2026-10-30 |
| 2 | The engine reports a finding at a line of such a file (CLI, SARIF, LSP, the hook) | the editor squiggles it | ✅ 2026-10-30 |
| 3 | W-128, W-129, W-132 report in both places | the manifest and the text file both show it | ↗ each item's own rule, through `NativeRule::elsewhere` (on main 2026-10-30) |

**Arpit, 2026-10-30 (voice), on Q-A:** *"I want the error to be displayed in both the places, in package.json as well as in nvmrc."*

**Model:** Opus (Claude Code): a law amendment plus the reporting path across every output format.

## Goal

The files: `.nvmrc`, `.node-version`, `.python-version`, `.tool-versions`, `Dockerfile`, `CODEOWNERS` (every location), yarn classic `yarn.lock`. Each is read as input by a rule; a finding about one is reported **twice** — on the L3 file it concerns (root manifest, workflow YAML) and on the text file's line. They do **not** become a tenth language: no lint rules of their own, no formatter, no rule may target them except through this list. Open to the agent: whether `requirements*.txt` joins the list (B-051) — say so to the decider; do not add it unasked.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · law | SR-LAW-3 amended on this ruling; CLAUDE.md's L3 line (SR-WORK-DOCS decision 7a) | ✅ 2026-10-30 — SR-LAW-3 decision 10; CLAUDE.md's L3 line; register row (02548f0) |
| 2 · engine | a location in a listed text file is valid for a finding; `check` text/json/sarif/github, LSP diagnostics and the hook carry it | ✅ 2026-10-30 — `codelaw_core::TEXT_CONFIG_FILES`, `NativeRule::elsewhere` and the engine's copy (SR-ARCHITECTURE decision 15d, 02548f0); the LSP publishes the copy on the text file's URI and checks an open text file (SR-EDITOR decision 2a); the hook takes an edited text file (SR-AGENT-SURFACES); the extension's selector opens them |
| 3 · tests | a fixture with `.nvmrc` ≠ `engines.node` shows both findings in every format | ✅ 2026-10-30 — `crates/codelaw-cli/tests/text_config.rs` (text, JSON, SARIF, GitHub, LSP, hook) and `crates/codelaw-engine/tests/text_config.rs`, on the test-only rule `test/node-version` |

## Decided by the agent (2026-10-30)

- **`requirements*.txt` is not on the list** (B-051): not added unasked; put to the decider. SR-LAW-3 decision 10 says so.
- **`Dockerfile` is matched by exact name**; `Dockerfile.<name>` variants are not on the list (a closed list; adding them is a ruling).
- **A copy is `manual` unless the rule gives it edits in the text file**, and names the finding's `file:line:column`; `codelaw fix` never writes a text file — a copy's edits are the editor's quick fix.
- **The editor re-checks an open text file on open and save**, not per keystroke: its copies come from the L3 files on disk.
- **A rule names the text file's place after `check`** (`NativeRule::elsewhere`, like `severity_of` and `data_of`), so no rule's `NativeFinding` literal changes under W-128, W-129 or W-132.

## Blockers

None. W-129 and W-132 need it for the second location; they may land the first location before it.

## Record affected

[SR-LAW-3](../../records/0005_LAW-3-languages.md) · SR-ARCHITECTURE (where a finding may sit) · SR-EDITOR (the extension must open diagnostics on these files).

## Hazards

- The VS Code extension may not activate on a plain-text file — check its `activationEvents` and document selector.
- The list is closed: adding a file is a decider's ruling, as any law change.

## Reference

[Proposal](../../work/proposals/deps-flags-markers-reviews.md) §8 · [W-129](../../work/open/W-129-deps-hygiene-rules.md) · [W-132](../../work/open/W-132-complexity-and-codeowners.md)
