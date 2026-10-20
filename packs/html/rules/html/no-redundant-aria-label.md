---
id: html/no-redundant-aria-label
title: "aria-label doesn't repeat the label"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/no-redundant-aria-label]
tags: [a11y]
since: 0.1.0
---

A form control doesn't have an `aria-label` equal to the text of its `<label for>`.

**Why:** The label already names the control; the copy drifts out of date when only the visible label is edited.

```codelaw
native: html/no-redundant-aria-label
```

## Examples

### HTML

#### Don't

```html
<label for="q">Search</label>
<input type="text" id="q" aria-label="Search">
```

#### Do

```html
<label for="q">Search</label>
<input type="text" id="q">
```
