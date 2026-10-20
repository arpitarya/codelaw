---
id: css/property-no-deprecated
title: "No deprecated properties"
languages: [css]
fix: suggest
fix_reason: "older browsers may support only the deprecated name"
stability: preview
options:
  ignoreProperties: { type: "string[]", default: [], description: "properties to accept; `/regex/` for a pattern" }
ported_from: [stylelint/property-no-deprecated]
since: 0.1.0
---

The sheet uses no property the specifications deprecated, such as `word-wrap`, `grid-gap`, `page-break-*` or `clip`.

**Why:** Deprecated properties are kept only as aliases or not at all, and may stop working.

```codelaw
native: css/property-no-deprecated
```

## Examples

### CSS

#### Don't

```css
a {
  word-wrap: break-word;
}
```

#### Do

```css
a {
  overflow-wrap: break-word;
}
```
