---
type: Handoff
name: W-137
description: "Arpit, 2026-10-30 (Cowork): make the VS Code extension show codelaw's findings more beautifully — badges, gutter icons, TODO chips, hover cards, a codelaw sidebar and status bar — from the design he picked."
item: W-137
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-137 — The extension's look: inline badges, hover cards, a sidebar and a status bar

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The server sends structured data with each finding, and adds inlay hints and CodeLens | every LSP editor gets hints, the hover card and the summary line | ⏳ |
| 2 | The extension draws gutter icons, row tints, end-of-line badges and TODO/ticket/date chips | the status of every line at a glance | ⏳ |
| 3 | The extension adds a codelaw sidebar (Dependencies, Deadlines, Tests, Ownership, scan banner) | one list of everything to act on | ⏳ |
| 4 | Status bar counts and coloured tab names | the totals always visible | ⏳ |
| 5 | Ordinary source files: **squiggles only**, plus one well-designed hover card (board H) | code stays clean; the card explains and fixes | ⏳ ruled 2026-10-30 |

**Arpit, 2026-10-30 (Cowork):** *"I want CodeLaw extension, the VS Code extension, to be a bit more beautiful on how it highlights the to-do or whether a package published or not."* → on the designs: *"I like the second design for package JSON, PyProject, TOML, test and to do, as well as I like the sidebar and status bar."* → *"On package.json and pyproject toml, I want the hover functionality that we have on first design as well."* → *"Okay, let's go with this design. file it."*

**Arpit, 2026-10-30, on source files (boards E–G):** *"All the three designs are way too much. … I liked the pop-up on the E … but the inline badges and all that, that's way too much. I want to keep it very simple, like squiggly line kind of things … The pop-up, we can design it well, but other than highlighting part, we just wanted squiggly lines, that's all."* → *"the pop-up shouldn't be there by default. It should come up when we try to hover over it. And there should be a squiggly line … And make badges as an optional thing … two kind of views which are configurable. The default one would be squiggly lines."*

**Design of record:** board **D · Final** for manifests, tests and TODO/flag markers; boards **H** (default view), **J** (every code use case, squiggles only) and **K** (the hover card for each of twelve use cases) for ordinary code — *"Yes, I like the squiggly line. Build the squiggly line for all the use cases that we have, as well as pop up with different use cases within the code. For package.json and pyproject.toml, keep the design that we have in D."* (Arpit, 2026-10-30) · board **L**, the three defaults together — *"keep badges for to do and fix me … I want badge and squiggly line configuration in three categories: for code, for manifest files like package.json, pyproject.toml, and if there are any others. And then the third one is for to do comments, skip comments, fix me comments. … For code, it is going to be squiggly lines. For others, it is going to be badge."* on the canvas *codelaw editor look* — <https://claude.ai/artifact/Ufrc5yN8MKfGqpAwaBi7qy> (A, B, C are the options it was drawn from).

**Model:** Opus (Claude Code): server capabilities in Rust and a decoration layer in the extension, theme-aware.

## Goal

Build board D. The split:

| surface | where | every LSP editor? |
|---|---|---|
| hover card (advisories, safe version, latest, data source; Change to / Quick Fix / Ignore with expiry) | server `hover` | yes |
| quick fixes, inlay hints, CodeLens summary ("2 vulnerable · Fix vulnerable (2)") | server | yes |
| gutter icons, row tints, end-of-line badges, TODO/ticket/date chips | extension decorations, from each diagnostic's `data` | VS Code and forks |
| codelaw sidebar (tree view), status bar items, tab colours (file decorations) | extension | VS Code and forks |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · data | each LSP diagnostic carries `data`: rule, grade, fix class, safe/latest version, advisory list, due date — one documented shape | ⏳ |
| 2 · server | `inlayHintProvider`, `codeLensProvider`; the hover card's sections as markdown | ⏳ |
| 3 · decorations | icons per kind (vulnerable, deprecated, outdated, clean, due, expired, flag, focused/skipped test, runtime); badges; chips in TODO/SKIP comments; colours as `contributes.colors` so light and high-contrast themes work | ⏳ |
| 4 · sidebar | tree view `codelaw`: Dependencies (Vulnerable, Deprecated, Licence denied, Outdated, Runtime mismatch), Deadlines (Expired, Due soon, Missing ticket), Tests, Ownership; the incomplete-scan banner with Refresh / turn the source off | ⏳ |
| 5 · status bar and tabs | counts (vulnerable, outdated, expired · due); a warning item while the scan is incomplete; `FileDecorationProvider` colours and counts | ⏳ |
| 6 · settings | three categories, each `"squiggles"` \| `"badges"`: `codelaw.view.code` (findings in JS/TS/Python code) default **`squiggles`**; `codelaw.view.manifests` (`package.json`, `pyproject.toml`, their lockfiles, `.nvmrc`/`.python-version`/`.tool-versions`) default **`badges`**; `codelaw.view.markers` (TODO, FIXME, HACK, SKIP comments and skipped-test calls) default **`badges`** — board L shows the defaults together. Plus `codelaw.sidebar` (default on), `codelaw.badges.clean` (show "latest") | ⏳ |
| 6b · source files | no badges, gutter icons, tints or CodeLens on ordinary code — the severity squiggle only — red error, yellow warn, blue info, green "fixes itself on save"; baseline findings a dotted grey underline; the hover card **only while hovering**, never pinned open: a plain title, severity and fix-class chips, one-sentence why, Don't/Do (or the fix as a before/after), rule id · CWE · new-or-baseline, then *Apply fix* or *Ask agent to fix*, *Rule page*, *Ignore with expiry…* | ⏳ |
| 6c · hover cards | one card per use case, as board K: security (needs your call), bug risk (one-click, before → after), fixes itself on save, baseline, vulnerable package imported here (dep's safe version, opens the manifest), TODO expired, TODO without a ticket, flag expiring (one-click, keeps the registry's branch), flag expired (pick the branch), focused test, skipped test without reason, too complex — each: plain title, severity and fix chips, one-line why, example, rule id · meta, one primary action | ⏳ |
| 7 · tests, docs | extension tests for the decoration mapping; screenshots in the extension README; SR-EDITOR | ⏳ |

## Blockers

None for the presentation. Its content arrives as W-128 (vulnerable), W-129, W-130 (flags), W-131 (TODO/skip) and W-132 (ownership) land; build against `deps/outdated` and fixtures until then. Step 5 waits on Arpit's pick.

## Record affected

[SR-EDITOR](../../records/0114_editor.md) · SR-CLI-MCP (LSP capabilities) · SR-PACKAGES (extension contributions).

## Hazards

- **Not chosen:** the "Fix both" pop-up notification (board C) — left out; ask before adding.
- **L11:** the extension's only runtime dependency is `vscode-languageclient` 9.0.1 — draw everything with the VS Code API; no UI library.
- Decorations must follow edits without flicker: recompute from the latest `publishDiagnostics`, visible editors only.
- Colour is never the only signal (icons and words on every badge); check contrast in Light+ and High Contrast.
- The mock-up's version numbers are illustrative.

## Reference

The canvas above · [proposal](../proposals/deps-flags-markers-reviews.md) · W-95 (the extension) · `packaging/vscode/`
