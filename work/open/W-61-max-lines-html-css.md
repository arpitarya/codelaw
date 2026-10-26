---
type: Handoff
name: W-61
description: "Arpit, 2026-10-26: max lines per file for HTML and for CSS — including SCSS and CSS Module files — with the same options shape as W-58's JS and Python rules."
item: W-61
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-61 — `html/max-lines` and `css/max-lines` (CSS, SCSS, CSS Modules)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write `html/max-lines` and `css/max-lines` pages with options and cases | the spec | ⏳ |
| 2 | Implement as `native:` rules on the counter W-58 builds (one shared line counter, per-language comment syntax) | one implementation for six languages | ⏳ |
| 3 | Add both to the `size` presets: `codelaw:html/size`, `codelaw:css/size` | one line to turn them on | ⏳ |
| 4 | Docs: USAGE's size-limits section gains HTML and CSS rows | users find it | ⏳ |

**Arpit, 2026-10-26:** *"I want max lines rule for HTML CSS files as well. Research and create a work item for it. The CSS pack should work with SCSS, and CSS modules as well."*

**Model:** Sonnet — two counting rules with written options; the counter is W-58's.

## Goal

**Prior art:** none of stylelint, html-validate, HTMLHint or Biome has a file-length rule for CSS or HTML (ESLint's `max-lines` is JS-only), so codelaw sets its own defaults.

### `css/max-lines` — `.css`, `.scss` (W-62), `*.module.css`/`*.module.scss` (W-63)

| option | type | default | meaning |
|---|---|---|---|
| `max` | int | 300 | most lines a stylesheet may have |
| `skip_blank_lines` | bool | true | |
| `skip_comments` | bool | true | `/* */` and, in SCSS, `//` lines |
| `skip_imports` | bool | false | `@import`, `@use`, `@forward` lines don't count |

### `html/max-lines` — `.html`

| option | type | default | meaning |
|---|---|---|---|
| `max` | int | 400 | most lines a page or template may have |
| `skip_blank_lines` | bool | true | |
| `skip_comments` | bool | true | `<!-- -->` lines |
| `skip_embedded` | bool | false | lines inside `<script>` and `<style>` don't count (their content is checked by the JS/CSS rules) |

- Scope tables (W-59) set per-file-pattern limits, e.g. `[scope.partials] files = ["**/_*.scss"]` at 150.
- Reported once at the first line over the limit; `fix: manual` (split into partials / components), steps on the page.
- **Companions already in the CSS pack:** none on size. Worth adding later (stylelint names): `max-nesting-depth` (matters most in SCSS), `selector-max-compound-selectors`, `selector-max-id`, `selector-max-specificity`.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | cases: blank, comment, SCSS `//`, embedded `<script>`/`<style>`, every option | ⏳ |
| 2 · | `codelaw test packs/html packs/css` green | ⏳ |
| 3 · | a scope gives `_*.scss` partials their own limit | ⏳ |

## Blockers

None. Uses W-58's counter if it has landed; otherwise this item writes it and W-58 reuses it.

## Record affected

SR-RULE-PACKS (the html and css `size` presets).

## Hazards

- Markdown and HTML embedded regions: `max-lines` counts the host file only (as in W-58).

## Reference

[stylelint rules](https://stylelint.io/user-guide/rules) · [ESLint `max-lines`](https://eslint.org/docs/latest/rules/max-lines) · [W-58](W-58-size-limits.md)
