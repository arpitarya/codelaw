---
id: html/no-redundant-for
title: "A wrapping label has no for"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/no-redundant-for]
tags: [a11y]
since: 0.1.0
---

A `<label>` that wraps its control doesn't also point at it with `for`.

**Why:** Wrapping already associates them; the `for` is one more id reference to keep in sync.

```codelaw
native: html/no-redundant-for
```

## Examples

### HTML

#### Don't

```html
<label for="q">Search <input type="text" id="q"></label>
```

#### Do

```html
<label>Search <input type="text" id="q"></label>
```
