---
id: css/declaration-block-no-duplicate-properties
title: "No property declared twice in one block"
languages: [css, scss]
fix: suggest
fix_reason: "the earlier declaration may be a deliberate fallback for browsers that reject the later value"
stability: preview
options:
  ignore: { type: "string[]", default: ["consecutive-duplicates-with-different-syntaxes"], description: "`consecutive-duplicates-with-different-syntaxes` allows fallbacks" }
  ignoreProperties: { type: "string[]", default: [], description: "properties to skip; `/regex/` for a pattern" }
ported_from: [stylelint/declaration-block-no-duplicate-properties]
since: 0.1.0
---

A declaration block declares each property once. Back-to-back declarations whose values have different syntaxes — a fallback such as `width: 100px; width: 50vw;` — are allowed, and so is `src`.

**Why:** The later declaration overrides the earlier one, so the earlier one is dead code or a mistake.

```codelaw
native: css/declaration-block-no-duplicate-properties
```

## Examples

### CSS

#### Don't

```css
a {
  color: pink;
  background: orange;
  color: orange;
}
```

#### Do

```css
a {
  background: orange;
  color: orange;
}
```
### SCSS

#### Don't

```scss
a {
  color: pink;
  background: orange;
  color: orange;
}
```

#### Do

```scss
a {
  background: orange;
  color: orange;
}
```
