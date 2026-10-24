---
type: Handoff
name: W-21
description: "Arpit, 2026-10-20: engine features some ported rules need."
item: W-21
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), docket 2026-10-24"
ball: closed
---

# W-21 — Close the engine gaps that hold rules out of the packs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Lint `` css`…` `` / `` html`…` `` tagged templates as embedded regions | CSS-in-JS and lit templates checked | ✅ 2026-10-22, on branch `W-21-pack-gaps` |
| 2 | Give native rules the file's path, raw bytes and neighbours | `md/relative-links`, `html/no-utf8-bom` and Python package checks become possible | ✅ 2026-10-22, on branch `W-21-pack-gaps` |
| 3 | Handle modern CSS that tree-sitter-css rejects (range media queries, `layer()`, fractional keyframe percentages) | `css/media-feature-range-notation` can be `auto` | ✅ 2026-10-22: an in-tree fork of tree-sitter-css 0.25.0; the upstream patch is prepared and checked against upstream's corpus (2026-10-24); the fork stays, the PR is backlog [B-046](../../work/BACKLOG.md) (ruled 2026-10-24) |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Arpit, 2026-10-24:** *"Then let's keep using the patch copy and put the PR in backlog."*

**Model:** Opus: a region extractor and a grammar fix both risk off-by-one locations, the main injection risk SR-LINTER names.

## Goal

The three gaps are closed and the rules the packs list as waiting on them are ported.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · templates | Tagged templates are regions when `[lint] tagged_templates = true` (SR-LINTER decision 3). | ✅ 2026-10-22 |
| 2 · context | Native rules receive path, bytes and a read-only view of sibling files. | ✅ 2026-10-22 |
| 3 · CSS | **Restated by the decider, 2026-10-24:** modern CSS parses through the fork, and the engine parses every CSS file and region and verifies every CSS fix with it; the CSS rules keep their text reader (`css_syntax.rs`). | ✅ 2026-10-24 (restated). Moving the rules onto a real tree is [B-047](../../work/BACKLOG.md) |
| 4 · ports | The waiting rules ship with `codelaw test` green. | ✅ 2026-10-22: `md/relative-links`, `html/no-utf8-bom`, `node/hashbang` ship green; `css/media-feature-range-notation` is `auto`, 16 checks green |

## What was built (2026-10-22)

- **Tagged templates** (`crates/codelaw-lang/src/regions.rs`): `` css`…` `` / `` html`…` `` with a plain identifier tag become CSS / HTML regions when `[lint] tagged_templates = true`. Each `${…}` is a *hole*: a placeholder chosen by position keeps the region parseable; a finding inside a placeholder is dropped and an edit that would touch a hole is refused. Nested templates are regions of their own. SR-LINTER decision 3.
- **File context** (`crates/codelaw-engine/src/files.rs`): native rules get the file's path, its bytes (BOM included) and a read-only view of the project (exists, is-dir, read, list), from disk or from the in-memory set for `check_files`/`codelaw test`; paths with `..` answer no. Every question is cached with its answer's hash and a cached result is reused only while all answers hold. `check_source`/`fix_source` read and keep a leading BOM. `codelaw test` checks a file-scope snippet with a file header as a set of files. SR-ARCHITECTURE decision 15b, SR-RULE-AUTHORING decision 4.
- **Rules:** `md/relative-links` (data, `md/recommended`), `html/no-utf8-bom` (html, `html/recommended`), `node/hashbang` (js, `node/recommended`, reads the nearest `package.json`'s `bin`; `suggest`). SR-RULE-PACKS and the pack READMEs updated.
- **Verified:** `cargo clippy --workspace --all-targets -D warnings` clean; `cargo test --workspace` 393 passed, 0 failed; `codelaw test packs/{data,html,python}` 0 failed; `packs/js` and `packs/css` each have one failure that is on `main` already (`js/no-irregular-whitespace`, `css/media-query-no-invalid`: W-23's layout refusal), not this item's. End-to-end: a `css` template's `colr` and `b {}` are reported at host positions and `fix` removes the empty block; a relative link to a missing file stops firing once the file exists, through the cache.

## What was built (2026-10-22, step 3)

- **The fork** (`crates/codelaw-lang/grammars/tree-sitter-css`): upstream 0.25.0 (MIT, its `LICENSE` kept), put in place by the workspace's `[patch.crates-io]`, version `0.25.0+codelaw.1`. `grammar.js` adds `range_query` (`(width >= 600px)`, `(400px <= width <= 700px)`, ratios, functions), `layer` / `layer(name)` in `@import`, and `float_value` keyframe selectors (`12.5%`; integer percentages already parsed). `src/` regenerated with tree-sitter-cli 0.25.10, which reproduces upstream's `parser.c` byte for byte from the unmodified grammar. Its README carries the credit, the change and the manual regeneration step.
- **Tests:** `test/corpus/modern.txt` (6 corpus tests, `tree-sitter test` green); `codelaw_lang::tests::modern_css_parses` (no `ERROR`, fails if the patch stops applying) and `fingerprint_names_the_css_fork` (`GRAMMARS_FINGERPRINT` names the fork's version, so a grammar change invalidates the cache).
- **No regressions found:** 564 stylesheets (bootstrap, bulma, normalize.css, animate.css, pico, open-props, every CSS example in `packs/`) parsed with 0.25.0 and the fork: all 555 error-free trees identical; 6 of the 9 that had errors now parse clean.
- **Rule:** `css/media-feature-range-notation` is `auto`; a `.cases.md` covers pairs, lists, ratios and range syntax left alone. `codelaw fix` rewrites `(min-width: 600px) and (max-width: 900px)` end to end.
- **Records:** SR-ARCHITECTURE decision 3 (the fork, the ruling, a reopen trigger in the veto), SR-PACKAGES decision 1 (the second allowed `[patch.crates-io]`), SR-LINTER decision 1, SR-LAW-2 Consequences, SR-RULE-PACKS decision 2, the register's ownership note.
- **Upstream:** `grammar.patch` and `PR.md` are prepared against 0.25.0 (grammar, highlights, corpus; `src/` to be regenerated) and handed to the parent session; nothing is filed.

## DoD 3, settled (2026-10-24)

**As written, DoD 3 does not hold, and an agent cannot make it hold inside this item.**

- **What the rules read.** All 70 rules in `packs/css/rules/css/` are `native:`, and every one is registered by one macro in `crates/codelaw-native/src/css_syntax.rs` that declares the single node kind `stylesheet` and re-reads the stylesheet's text with that file's 1,028-line reader. No CSS rule looks at a tree node below the root. The rule bodies are about 2,800 lines in `css_{atrules,declarations,media,misc,selectors,values}.rs`.
- **Why they cannot move onto the tree yet.** Probed with the fork's grammar (`tree-sitter parse`, 2026-10-24). These still give `ERROR`: a nesting selector after a compound (`a { .b & {} }`), `@page :first`, `@container name (…)`, `@custom-media --x (…)`, `@import … supports(…)`, `@font-feature-values`, `!ie`-style annotations, named-range keyframe selectors (`entry 0%`), an unclosed block. Worse than an `ERROR`: a custom property holding a block, `:root { --x: { color: red; } }`, parses as a *nested rule* `--x` whose `color: red` looks like a real declaration, so tree-based `declaration-block-no-duplicate-properties`, `no-unknown-property` and the value rules would report findings that are not there. The text reader handles every one of these today, and the pack's cases depend on it.
- **Some rules report what the grammar rejects.** `css/media-query-no-invalid` and `css/at-rule-prelude-no-invalid` say *what* is wrong in a prelude the grammar turns into a bare `ERROR`. On the tree they could only say "syntax error".
- **A partial port does not meet the DoD either.** The pack would then read CSS two ways, and the side parser would stay.
- **What does hold:** the fork reads the three forms, the engine parses every CSS file and region with it, and the fix verifier (SR-ARCHITECTURE decision 5) refuses any CSS fix that adds an `ERROR`. SR-ARCHITECTURE decision 15b now records this under *The CSS rules read text, not the tree*.
- **The decider's choice:** (a) restate DoD 3 as what holds (*modern CSS parses through the fork and every CSS fix is verified by it; the rules keep their text reader*) and close W-21 once the patch is filed; or (b) keep DoD 3 and file a new item: *extend the fork for the forms above, then port the 70 rules onto the tree*. That means a grammar change per form plus a rewrite of every CSS rule.

## Left

- **The upstream patch: backlog [B-046](../../work/BACKLOG.md)** (Arpit, 2026-10-24: keep the fork, put the PR in the backlog). Opening a PR on tree-sitter/tree-sitter-css is outward-facing, so no agent does it. Also ready: `upstream/0001-tree-sitter-css-modern.patch`, one commit with `src/` regenerated (byte-identical to the fork, re-checked 2026-10-24; `PR.md` says so). Ready: [`upstream/grammar.patch`](../../crates/codelaw-lang/grammars/tree-sitter-css/upstream/grammar.patch) (apply on the `v0.25.0` tag with `patch -p1`, then `tree-sitter generate`) and [`upstream/PR.md`](../../crates/codelaw-lang/grammars/tree-sitter-css/upstream/PR.md) (title on its first line, body ready to paste). **Checked 2026-10-24** on upstream's `v0.25.0` tarball with tree-sitter-cli 0.25.10. Unpatched, `generate` reproduces the released `parser.c` byte for byte and `tree-sitter test` passes 40/40. Patched, it applies cleanly, the corpus passes 46/46, the highlight test passes (37 assertions), `eslint grammar.js` is clean, and the regenerated `src/parser.c`, `grammar.json` and `node-types.json` match the fork byte for byte. When a release carries it, drop the fork (its README says how; SR-ARCHITECTURE's veto condition is the trigger).
- **Still `ERROR` in the fork, not filed:** `@import … supports(…)`, a name before the query in `@custom-media` / `@container` (`open-props`' media file), named-range keyframe selectors (`entry 0%`).
- **The CSS rules still read text themselves** (`css_syntax.rs`): see *DoD 3, settled*.
- Python package checks (e.g. a missing `__init__.py`) are now possible but no rule is filed for them.

## Blockers

None. Closed 2026-10-24: **Arpit, 2026-10-24 (docket):** *"(a) engine-only, close W-21"*. The upstream PR is backlog B-046; the CSS-tree port is B-047. The fork-vs-upstream question is already ruled: *Arpit, 2026-10-22: fork in-tree + upstream patch; drop the fork when upstream releases.*

## Record affected

[SR-LINTER](../../records/0106_linter.md) decisions 1 and 3 · [SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 3, 15b (the CSS rules' text reader, 2026-10-24), 16 · [SR-PACKAGES](../../records/0107_packages.md) decision 1 · [SR-LAW-2](../../records/0004_LAW-2-foss-dependencies.md) · [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 2.

## Hazards

- **A grammar fork** is a dependency change: L2 and a record (both done, 2026-10-22).
- **A stale fork:** if `ast-grep-language` moves off `tree-sitter-css` 0.25, the patch silently stops applying; `modern_css_parses` and `fingerprint_names_the_css_fork` fail when it does.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
