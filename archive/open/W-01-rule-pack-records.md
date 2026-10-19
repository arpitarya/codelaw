---
type: Handoff
name: W-01
description: "Arpit, 2026-10-19: publish the built-in rules as separate packages (JS/TS, HTML, CSS, Python, data) apart from the engine; amend L6 and the records before anything is built."
item: W-01
filed: 2026-10-19
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-19
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-01 — Rewrite the records so rules ship as five separate packages, apart from the engine

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules how rule packs are published | the shape every record follows | ✅ ruled 2026-10-19 |
| 2 | Amend law L6 with the new package names | packs become legal on npm and PyPI | ✅ 2026-10-19 |
| 3 | Rewrite SR-RULE-PACKS around five first-party packs | one home for what each pack holds | ✅ 2026-10-19 |
| 4 | Update SR-PACKAGES, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-ARCHITECTURE, SR-AGENT-SURFACES | every record agrees with the split | ✅ 2026-10-19 |
| 5 | Register, SR-LAWS, `CLAUDE.md` L6 line, glossary | the bookkeeping the change owes | ✅ 2026-10-19 |

**Arpit, 2026-10-19:** *"research the rules preset in ruff, eslint, tslint, prettier, and other and create as separate packages, like 1 for js/ts/node/jsx/tsx, 1 for html, 1 for css, 1 for python and engine will be separate"*

**Rulings, same session (Cowork):**
- **Separately published,** not just separate in source.
- **Mirror every pack on both registries:** npm `@codelaw/rules-<name>`, PyPI `codelaw-rules-<name>`.
- **A fifth pack, `rules-data`,** for JSON, YAML, TOML and Markdown.
- **`codelaw init` installs the packs for the languages it detects;** the engine package itself carries no rules.

**Model:** Opus: it amends a law and rewrites the reasoning of a record that previously rejected this exact option; no test catches a bad rewrite.

## Goal

The records describe five separately published rule packs plus a rule-free engine, consistently, before W-02 or W-03 builds anything.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · L6 | SR-LAW-6 law text and decision 10 name the packs: npm `codelaw` + `@codelaw/cli-*` + `@codelaw/rules-{js,html,css,python,data}`; PyPI `codelaw` + `codelaw-rules-{js,html,css,python,data}`. Decision 3 ("complete alone") narrows to the engine: every language, formatter, MCP, LSP; rules come from packs. The ruling is named. | ✅ 2026-10-19 |
| 2 · versions | One version for the engine and all five packs, released together (L6 decision 9 extended); a pack declares `min_codelaw`. | ✅ 2026-10-19 |
| 3 · SR-RULE-PACKS | Rewritten: the five packs, their languages and presets (table below), resolution of `extends = ["codelaw:js/recommended"]` from either registry, third-party packs unchanged. The old "embedded in the binary" choice moves to *Alternatives considered* with the ruling as the reason. | ✅ 2026-10-19 |
| 4 · SR-PACKAGES | Distribution table gains the 10 pack packages; crate `codelaw-rules-builtin` is removed; packs live in `packs/<name>/` in the source repo. | ✅ 2026-10-19 |
| 5 · SR-CONFIG / SR-CONSUMER-LAYOUT | `extends` syntax for first-party presets; installed packs shown in the consumer tree; `init` adds the detected packs as dev dependencies. | ✅ 2026-10-19 |
| 6 · SR-RULE-FORMAT | New frontmatter `ported_from:` (e.g. `[eslint/no-dupe-keys, ruff/F601]`); new `codelaw` block form `native: <id>` for rules the engine implements in Rust. | ✅ 2026-10-19 |
| 7 · SR-ARCHITECTURE | Adds the per-file **semantic model** (scopes, bindings, references) for JS/TS and Python, the `native:` rule hook, and pack data tables (CSS properties, HTML elements). | ✅ 2026-10-19 |
| 8 · SR-AGENT-SURFACES | `codelaw-setup` installs packs; `codelaw-migrate` maps ESLint/Ruff/stylelint configs through `ported_from`. | ✅ 2026-10-19 |
| 9 · bookkeeping | Register, SR-LAWS, `CLAUDE.md` L6 line, glossary ("rule pack", "native rule", "semantic model"), worklog, INTERVIEW. | ✅ 2026-10-19 |
| 10 · accept | SR-RULE-PACKS set to `accepted` once rewritten (ruled 2026-10-19: "go with the recommendations" — accept after the rewrite). | ✅ 2026-10-19 |

**The five packs and the engine:**

| Package (npm · PyPI) | Languages | Presets |
|---|---|---|
| `codelaw` · `codelaw` | none: the engine; ships only `codelaw/*` meta diagnostics and `graph/no-deprecated-usage` | — |
| `@codelaw/rules-js` · `codelaw-rules-js` | JS, JSX, TS, TSX, incl. Node, React, Stencil | `js/recommended`, `ts/recommended`, `react/recommended`, `node/recommended`, `stencil/recommended`, `js/strict` |
| `@codelaw/rules-html` · `codelaw-rules-html` | HTML (and HTML in Markdown/templates) | `html/recommended`, `html/a11y`, `html/document` |
| `@codelaw/rules-css` · `codelaw-rules-css` | CSS | `css/recommended`, `css/standard` |
| `@codelaw/rules-python` · `codelaw-rules-python` | Python | `py/recommended`, `py/strict` |
| `@codelaw/rules-data` · `codelaw-rules-data` | JSON, YAML, TOML, Markdown | `data/recommended`, `md/recommended`, `package-json`, `tsconfig`, `pyproject`, `github-actions` |

Graph rules live with their language: `graph/no-unused-exports` in js and python, `graph/no-unused-css-classes` in css.

## Blockers

None. The records it amends were accepted on 2026-10-19 with this rewrite foreseen; the amendment names the ruling.

## Outcome (2026-10-19)

Done in one session. Where the result differs from the plan above:

- **`rules-data` presets are namespaced** `data/package-json`, `data/tsconfig`, `data/pyproject` and `data/github-actions`, so every first-party preset maps to its pack by namespace (SR-RULE-PACKS decision 4). `json`, `yaml` and `toml` map to `rules-data` too.
- **Each pack gains an opt-in `conventions` preset** for the v0 starter rules that match no source rule (SR-RULE-PACKS decision 8; placed by W-03).
- **Crates: 20.** `codelaw-rules-builtin` is out; `codelaw-semantic` and `codelaw-native` are in (SR-PACKAGES decision 1).
- **SR-ARCHITECTURE gains decisions 15a–15c,** lettered so that no existing reference is renumbered. SR-RULE-PACKS decisions were renumbered; every citation of them (SR-ARCHITECTURE 18, SR-CONFIG, SR-RULE-FORMAT 9, SR-RULE-AUTHORING 6) was repointed.
- **`native:` is reserved** for first-party packs and ejected copies of their rules (SR-RULE-PACKS decision 10).
- **SR-RULE-PACKS accepted** on the 2026-10-19 ruling (DoD 10).

## Record affected

SR-LAW-6 (law text, decisions 3, 9, 10) · SR-RULE-PACKS · SR-PACKAGES decisions 1–5 · SR-CONFIG decisions 3, 8 · SR-CONSUMER-LAYOUT §1 tree · SR-RULE-FORMAT decisions 2, 3 · SR-ARCHITECTURE groups D and E · SR-AGENT-SURFACES skill catalogue.

## Hazards

- **L6 decision 3 completeness:** after the split, `pip install codelaw` alone lints nothing. The DoD must say so plainly and make `init` the documented path.
- **L3 independence still holds:** a Python-only repo installs `codelaw` and `codelaw-rules-python` from PyPI and never touches npm. Keep that sentence in L3's enforcement section.
- **The SR-PACKAGES crate table** loses `codelaw-rules-builtin`; check every record that cites it (`grep -rn rules-builtin records docs`).

## Reference

- [W-03](W-03-rule-packs.md): the research this item's tables come from.
- [SR-RULE-PACKS](../../records/0101_rule-packs.md), [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md).
