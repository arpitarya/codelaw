---
id: scss/at-rule-no-unknown
title: "No unknown at-rules (SCSS)"
languages: [scss]
fix: manual
fix_reason: "the intended at-rule is the author's call"
stability: preview
options:
  ignoreAtRules: { type: "string[]", default: [], description: "at-rules to accept, such as a framework's; `/regex/` for a pattern" }
ported_from: [stylelint-scss/at-rule-no-unknown]
since: 0.3.0
---

Every at-rule is one CSS or Sass defines: `@mixin`, `@include`, `@use`, `@if`, `@each` and the rest of Sass's own are known. This rule replaces `css/at-rule-no-unknown` on `.scss` files, which does not know Sass.

**Why:** A browser skips a CSS at-rule it does not know, block and all, and Sass passes an unknown one through, so a typo silently disables its contents.

```codelaw
native: scss/at-rule-no-unknown
```

## How to fix

1. Correct the at-rule name.
2. If a build tool handles it, add it to `ignoreAtRules`.

## Examples

### SCSS

#### Don't

```scss
@mixn button {
  color: red;
}
```

#### Do

```scss
@mixin button {
  color: red;
}
```

## Differences from the source

Sass's at-rules are always known; the rule checks the CSS at-rules against the same table as `css/at-rule-no-unknown`.
