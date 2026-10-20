---
id: html/no-dup-class
title: "No class listed twice"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/no-dup-class]
since: 0.1.0
---

A `class` attribute lists each class once.

**Why:** The repeat does nothing, and suggests a merge went wrong.

```codelaw
native: html/no-dup-class
```

## Examples

### HTML

#### Don't

```html
<p class="lead lead">Text</p>
```

#### Do

```html
<p class="lead">Text</p>
```
