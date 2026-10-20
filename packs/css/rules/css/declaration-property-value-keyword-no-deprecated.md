---
id: css/declaration-property-value-keyword-no-deprecated
title: "No deprecated keywords in property values"
languages: [css]
fix: suggest
fix_reason: "the replacement keyword is the closest modern one, not always identical"
stability: preview
options:
  ignoreKeywords: { type: "string[]", default: [], description: "keywords to accept; `/regex/` for a pattern" }
ported_from: [stylelint/declaration-property-value-keyword-no-deprecated]
since: 0.1.0
---

Values use no keyword the specifications deprecated, such as `overflow: overlay`, `word-break: break-word` or the old system colours (`ThreeDFace`).

**Why:** Deprecated keywords are aliased or dropped by browsers and may stop working.

```codelaw
native: css/declaration-property-value-keyword-no-deprecated
```

## Examples

### CSS

#### Don't

```css
a {
  overflow: overlay;
}
```

#### Do

```css
a {
  overflow: auto;
}
```
