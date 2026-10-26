---
id: scss/load-no-partial-leading-underscore
title: "No leading underscore in a loaded partial"
languages: [scss]
fix: auto
stability: preview
ported_from: [stylelint-scss/at-import-no-partial-leading-underscore]
since: 0.3.0
---

`@use`, `@forward` and `@import` name a partial without its leading underscore: `@use "colors"`, not `@use "_colors"`.

**Why:** Sass finds `_colors.scss` from `"colors"`; the underscore in the path is noise, and a file renamed to drop it breaks every load.

```codelaw
native: scss/load-no-partial-leading-underscore
```

## Examples

### SCSS

#### Don't

```scss
@use "base/_colors";
```

#### Do

```scss
@use "base/colors";
```

## Differences from the source

Plain CSS imports (`.css`, URLs) and `sass:` modules are skipped.
