---
id: css/selector-no-vendor-prefix
title: "No vendor-prefixed selectors"
languages: [css]
fix: suggest
fix_reason: "browsers that need the prefix lose the rule"
stability: preview
options:
  ignoreSelectors: { type: "string[]", default: [], description: "selectors to accept; `/regex/` for a pattern" }
ported_from: [stylelint/selector-no-vendor-prefix]
since: 0.1.0
---

Pseudo-classes and pseudo-elements a build tool prefixes, such as `::-moz-placeholder`, are written without the prefix.

**Why:** Prefixing is a build step; hand-written prefixes go stale and duplicate code.

```codelaw
native: css/selector-no-vendor-prefix
```

## Examples

### CSS

#### Don't

```css
input::-moz-placeholder {
  color: gray;
}
```

#### Do

```css
input::placeholder {
  color: gray;
}
```
