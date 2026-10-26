---
id: css/at-rule-no-unknown
title: "No unknown at-rules"
languages: [css]
fix: manual
fix_reason: "the intended at-rule is the author's call"
stability: preview
options:
  ignoreAtRules: { type: "string[]", default: [], description: "at-rules to accept, such as a preprocessor's; `/regex/` for a pattern" }
ported_from: [stylelint/at-rule-no-unknown]
since: 0.1.0
---

Every at-rule is one the CSS specifications define (a vendor prefix is allowed).

**Why:** A browser skips an at-rule it does not know, block and all, so a typo silently disables its contents.

```codelaw
native: css/at-rule-no-unknown
```

## How to fix

1. Correct the at-rule name.
2. If a build tool handles it (`@tailwind`, `@include`), add it to `ignoreAtRules`.

## Examples

### CSS

#### Don't

```css
@keyframe fade {
  from {
    opacity: 0;
  }
}
```

#### Do

```css
@keyframes fade {
  from {
    opacity: 0;
  }
}
```

## Differences from the source

On `.scss` files `scss/at-rule-no-unknown` takes this rule's place: it knows Sass's own at-rules.
