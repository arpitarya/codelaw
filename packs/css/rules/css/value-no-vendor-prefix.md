---
id: css/value-no-vendor-prefix
title: "No vendor-prefixed values"
languages: [css, scss]
fix: suggest
fix_reason: "browsers that need the prefix lose the value"
stability: preview
options:
  ignoreValues: { type: "string[]", default: ["box", "inline-box"], description: "values to accept, without their prefix; `/regex/` for a pattern" }
ported_from: [stylelint/value-no-vendor-prefix]
since: 0.1.0
---

Values a build tool prefixes, such as `-webkit-flex` or `-moz-calc()`, are written without the prefix. `-webkit-box` and `-webkit-inline-box` are allowed, as line clamping still needs them.

**Why:** Prefixing is a build step; hand-written prefixes go stale and duplicate code.

```codelaw
native: css/value-no-vendor-prefix
```

## Examples

### CSS

#### Don't

```css
a {
  display: -webkit-flex;
}
```

#### Do

```css
a {
  display: flex;
}
```
### SCSS

#### Don't

```scss
a {
  display: -webkit-flex;
}
```

#### Do

```scss
a {
  display: flex;
}
```
