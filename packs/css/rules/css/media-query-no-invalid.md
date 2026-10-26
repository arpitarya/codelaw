---
id: css/media-query-no-invalid
title: "Media queries follow the grammar"
languages: [css, scss]
fix: manual
fix_reason: "the intended query is the author's call"
stability: preview
options:
  ignoreFunctions: { type: "string[]", default: [], description: "functions to accept in queries, such as `theme`" }
ported_from: [stylelint/media-query-no-invalid]
since: 0.1.0
---

Every query in `@media` and `@custom-media` is grammatical under Media Queries level 5: `not` is followed by a space, comparisons are `<`, `<=`, `>`, `>=` or `=`, and `and` and `or` are not mixed without parentheses.

**Why:** A browser treats an invalid query as `not all`, so the block never applies.

```codelaw
native: css/media-query-no-invalid
```

## How to fix

1. Rewrite the query; add parentheses to group `and` and `or`, and a space after `not`.

## Examples

### CSS

#### Don't

```css
@media (color) and (hover) or (width >= 600px) {
  a {
    color: red;
  }
}
```

#### Do

```css
@media (color) and ((hover) or (width >= 600px)) {
  a {
    color: red;
  }
}
```
### SCSS

#### Don't

```scss
@media (color) and (hover) or (width >= 600px) {
  a {
    color: red;
  }
}
```

#### Do

```scss
@media (color) and ((hover) or (width >= 600px)) {
  a {
    color: red;
  }
}
```
