---
id: scss/no-global-function-names
title: "No global function names"
languages: [scss]
fix: auto
stability: preview
ported_from: [stylelint-scss/no-global-function-names]
since: 0.3.0
---

The built-in functions Dart Sass moved into modules are called through their module: `map.get()`, not `map-get()`; `math.percentage()`, not `percentage()`. Functions that are also plain CSS (`min`, `max`, `round`, `abs`, `grayscale`, `invert`, `saturate`, `opacity`) are not reported: a call to one may be CSS.

**Why:** The global names are deprecated in Dart Sass and go away in 3.0. Module functions also keep a file's names from colliding.

```codelaw
native: scss/no-global-function-names
```

## Examples

### SCSS

#### Don't

```scss
.a {
  width: map-get($sizes, small);
}
```

#### Do

```scss
@use "sass:map";

.a {
  width: map.get($sizes, small);
}
```

## Differences from the source

stylelint-scss reports the global names it lists; this rule reports the module-moved built-ins and fixes the renames, adding each `@use "sass:…"` once. The colour adjusters are reported without a fix.
