# @codelaw/rules-stencil

codelaw's Stencil rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on
npm only, as `@codelaw/rules-stencil`; PyPI has no copy (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:stencil/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

| preset | rules | ported from |
|---|---|---|
| `stencil/recommended` | 15 | @stencil/eslint-plugin 1.4.0 `recommended` — 15 of its 21 rules |
| `stencil/conventions` | 1 | codelaw's starter catalogue: `stencil/tag-prefix` — opt in |

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule.

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `stencil/tag-prefix` | with no `prefix` option set, a tag only needs some prefix |
| `stencil/prop-readonly` | the starter id for upstream `props-must-be-readonly` |

## Not ported, and why

| rule | why |
|---|---|
| `stencil/async-methods`, `stencil/render-returns-host`, `stencil/single-export`, `stencil/strict-boolean-conditions` | need type information |
| `stencil/element-type` | needs the component's tag mapped to its element type name |
| `stencil/decorators-style` | layout: the formatter owns it |
| `stencil/dependency-suggestions` | advisory, with no fixed check to port |
