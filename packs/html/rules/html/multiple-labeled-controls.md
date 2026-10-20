---
id: html/multiple-labeled-controls
title: "A label labels one control"
languages: [html]
fix: manual
fix_reason: "each control needs its own label text"
stability: preview
ported_from: [html-validate/multiple-labeled-controls]
tags: [a11y]
since: 0.1.0
---

A `<label>` is associated with at most one control, by nesting or by `for`.

**Why:** A label names one control; assistive technology announces the others without the right name.

```codelaw
native: html/multiple-labeled-controls
```

## How to fix

1. Give each control its own `<label>`.

## Examples

### HTML

#### Don't

```html
<label>Name <input type="text" name="first"> <input type="text" name="last"></label>
```

#### Do

```html
<label>First name <input type="text" name="first"></label>
<label>Last name <input type="text" name="last"></label>
```
