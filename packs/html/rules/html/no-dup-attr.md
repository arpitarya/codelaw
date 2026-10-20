---
id: html/no-dup-attr
title: "No attribute written twice"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/no-dup-attr, htmlhint/attr-no-duplication]
since: 0.1.0
---

An element does not have the same attribute twice (names compare without case).

**Why:** Browsers keep the first one and silently drop the rest, which is rarely what the author meant.

```codelaw
native: html/no-dup-attr
```

## Examples

### HTML

#### Don't

```html
<input type="text" name="q" name="search">
```

#### Do

```html
<input type="text" name="q">
```

## Differences from the source

The fix removes the later copies, keeping what browsers already use. Check that the first value is the one you meant.
