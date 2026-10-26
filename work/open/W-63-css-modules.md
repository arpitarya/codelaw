---
type: Handoff
name: W-63
description: "Arpit, 2026-10-26: CSS Modules as a mode of the CSS pack: :global, :local, composes, :export and @value accepted, and graph rules that join a module's classes with the JS/TS that reads styles.x — unused classes, undefined classes, broken composes."
item: W-63
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-63 — CSS Modules — `*.module.css` / `*.module.scss` understood, unused and undefined classes caught

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Detect modules: `*.module.css`, `*.module.scss` (W-62), plus `[css] modules = [<globs>]` | the mode | ⏳ |
| 2 | Parser/rules: accept `:global(…)`, `:local(…)`, `composes:`, `:export {}`, `@value` — no `css/*` false positives | existing rules stay quiet | ⏳ |
| 3 | Project graph: an import of a module stylesheet binds its class names as exports; `styles.foo`, `styles["foo"]` and destructuring are uses | the join the rules need | ⏳ |
| 4 | Rules below, with cases across CSS and TS files | the checks | ⏳ |
| 5 | **A separate pack `css-modules`** — `@codelaw/rules-css-modules`, npm only (CSS Modules are a JS-bundler feature) — holding the `css-modules/*` rules; presets `css-modules/recommended` and `css-modules/strict`; amend L6 with W-62 | only projects using CSS Modules install it | ⏳ |
| 6 | `codelaw init` adds the pack when it finds `*.module.*` files | adoption | ⏳ |

**Arpit, 2026-10-26:** *"I want max lines rule for HTML CSS files as well. Research and create a work item for it. The CSS pack should work with SCSS, and CSS modules as well."*

**Arpit, 2026-10-26:** *"Create separate packages for SCSS and CSS modules."* Parsing `:global`/`composes`/`:export` and the graph join stay in the engine and the `css` pack; the `css-modules/*` rules ship in their own pack.

**Model:** Opus — it extends the project graph across languages; the use-detection decides every finding.

## Goal

| rule | flags | fix |
|---|---|---|
| `css-modules/no-unused-class` | a class in `x.module.css` no JS/TS file reads | manual: delete it (an `auto` fix only with `--unsafe`) |
| `css-modules/no-undefined-class` | `styles.foo` in a JS/TS file where the imported module has no `.foo` — reported in the JS/TS file | manual |
| `css-modules/composes-valid` | `composes: a from "./b.module.css"` where `b` or `.a` doesn't exist; `composes` inside a nested or non-class selector | manual |
| `css-modules/class-naming` | class names that `styles.x` can't reach without brackets (option: `camelCase` \| `kebab` \| `any`, default `camelCase`) | manual |
| `css-modules/no-global` | `:global` selectors (strict: a module should stay local) | manual |

- **Dynamic access** (`styles[name]`, `styles[\`btn-${v}\`]`): the module is marked dynamic and `no-unused-class` stays silent for it — never a false positive.
- Prior art: eslint-plugin-css-modules (MIT: `no-unused-class`, `no-undef-class`), stylelint-config-css-modules.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | a React fixture: an unused class, an undefined `styles.foo`, a bad `composes` — each reported once in the right file | ⏳ |
| 2 · | `:global`, `composes`, `:export` give no `css/*` findings | ⏳ |
| 3 · | dynamic access silences `no-unused-class` for that module | ⏳ |
| 4 · | works for `.module.scss` once W-62 lands | ⏳ |

## Blockers

None for `.module.css`; `.module.scss` needs **W-62**.

## Record affected

SR-PROJECT-GRAPH (stylesheet modules as graph nodes) · L6 (the `css-modules` pack) · SR-RULE-PACKS (its row) · SR-CONFIG (`[css] modules`).

## Hazards

- Bundler options change names (`localsConvention: camelCase` maps `.btn-primary` to `styles.btnPrimary`): an option `[css] modules_convention` mirrors it.
- Next.js and Vite both use `*.module.*`; webpack setups may use any glob — hence the config key.

## Reference

[CSS Modules spec](https://github.com/css-modules/css-modules) · [eslint-plugin-css-modules](https://github.com/atfzl/eslint-plugin-css-modules) · [stylelint-config-css-modules](https://github.com/pascalduez/stylelint-config-css-modules)
