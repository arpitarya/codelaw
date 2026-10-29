---
type: Handoff
name: W-136
description: "Arpit, 2026-10-30 (voice): a finding about a plain-text config file (.nvmrc and the like) shows on that file's line as well as on the manifest — L3 amended for findings only."
item: W-136
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-136 — L3: findings on plain-text config files

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L3: a closed list of plain-text config files may carry findings — never linted or formatted as a language | a warning can sit on the `.nvmrc` line | ⏳ |
| 2 | The engine reports a finding at a line of such a file (CLI, SARIF, LSP, the hook) | the editor squiggles it | ⏳ |
| 3 | W-128, W-129, W-132 report in both places | the manifest and the text file both show it | ⏳ |

**Arpit, 2026-10-30 (voice), on Q-A:** *"I want the error to be displayed in both the places, in package.json as well as in nvmrc."*

**Model:** Opus (Claude Code): a law amendment plus the reporting path across every output format.

## Goal

The files: `.nvmrc`, `.node-version`, `.python-version`, `.tool-versions`, `Dockerfile`, `CODEOWNERS` (every location), yarn classic `yarn.lock`. Each is read as input by a rule; a finding about one is reported **twice** — on the L3 file it concerns (root manifest, workflow YAML) and on the text file's line. They do **not** become a tenth language: no lint rules of their own, no formatter, no rule may target them except through this list. Open to the agent: whether `requirements*.txt` joins the list (B-051) — say so to the decider; do not add it unasked.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · law | SR-LAW-3 amended on this ruling; CLAUDE.md's L3 line (SR-WORK-DOCS decision 7a) | ⏳ |
| 2 · engine | a location in a listed text file is valid for a finding; `check` text/json/sarif/github, LSP diagnostics and the hook carry it | ⏳ |
| 3 · tests | a fixture with `.nvmrc` ≠ `engines.node` shows both findings in every format | ⏳ |

## Blockers

None. W-129 and W-132 need it for the second location; they may land the first location before it.

## Record affected

[SR-LAW-3](../../records/0005_LAW-3-languages.md) · SR-ARCHITECTURE (where a finding may sit) · SR-EDITOR (the extension must open diagnostics on these files).

## Hazards

- The VS Code extension may not activate on a plain-text file — check its `activationEvents` and document selector.
- The list is closed: adding a file is a decider's ruling, as any law change.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §8 · [W-129](W-129-deps-hygiene-rules.md) · [W-132](W-132-complexity-and-codeowners.md)
