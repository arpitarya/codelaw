---
id: css/no-empty-source
title: "No empty stylesheets"
languages: [css]
fix: manual
fix_reason: "whether to delete the file or fill it is the author's call"
stability: preview
ported_from: [stylelint/no-empty-source]
since: 0.1.0
---

A stylesheet (or `<style>` element) holds something besides whitespace.

**Why:** An empty stylesheet is a wasted request or a file left behind.

```codelaw
native: css/no-empty-source
```

## How to fix

1. Delete the file and what links it, or add the styles it was meant to hold.

## Examples

### CSS

#### Don't

```css

```

#### Do

```css
a {
  color: red;
}
```
