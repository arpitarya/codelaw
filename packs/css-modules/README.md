# @codelaw/rules-css-modules

codelaw's rules for CSS Modules — one of the first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)), **on npm only**: CSS Modules are
a JavaScript bundler feature. The rules read the join between a module stylesheet
(`*.module.css`, `*.module.scss`, or what `[css] modules` names) and the JavaScript or
TypeScript that imports it.

```toml
# codelaw.toml
extends = ["codelaw:css/recommended", "codelaw:css-modules/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it

[css]
modules = ["src/**/*.styles.css"]    # besides *.module.css and *.module.scss
modules_convention = "camelCase"     # css-loader's localsConvention: how `.btn-primary` is read
```

The `css` pack's rules also run on module stylesheets and know `:global`, `:local`,
`composes:`, `:export {}`, `:import() {}` and `@value`.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `css-modules/recommended` | 3 | eslint-plugin-css-modules's `no-unused-class` and `no-undef-class`, and `composes` resolution |
| `css-modules/strict` | 5 | the additions to `css-modules/recommended` (it extends it): names readable as `styles.x`, no `:global` |

<!-- /docs-sync:presets -->

## Rules

<!-- docs-sync:rules -->

| rule | fix | flags |
|---|---|---|
| `css-modules/class-naming` | manual | a class `styles.x` cannot reach without brackets (`camelCase`, `kebab` or `any`) |
| `css-modules/composes-valid` | manual | `composes` naming a missing file or class, or in a nested or non-class rule |
| `css-modules/no-global` | manual | a `:global` selector in a module |
| `css-modules/no-undefined-class` | manual | `styles.x` where the module defines no `.x`, reported in the JS or TS file |
| `css-modules/no-unused-class` | suggest | a class no file reads |

<!-- /docs-sync:rules -->

## Dynamic access

`styles[name]`, ``styles[`btn-${size}`]``, `{ ...rest } = styles` and passing `styles` to a
function mark the module as read by names the rules cannot see: `no-unused-class` reports nothing
for it. A module no file imports is left alone too. Never a false positive.
