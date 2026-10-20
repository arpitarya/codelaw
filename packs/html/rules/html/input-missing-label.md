---
id: html/input-missing-label
title: "Form controls have a label"
languages: [html]
fix: manual
fix_reason: "the label text needs a human"
stability: preview
ported_from: [html-validate/input-missing-label, htmlhint/input-requires-label]
since: 0.1.0
---

Every visible `<input>`, `<select>` and `<textarea>` has a label: a `<label for>`, a wrapping `<label>`, `aria-label` or `aria-labelledby`, with text.

**Why:** Without a label, screen readers announce the control as "edit text" with no hint of what to enter.

```codelaw
native: html/input-missing-label
```

## How to fix

1. Add a `<label for>` with text, or wrap the control in a `<label>`.
2. If a visible label is impossible, add `aria-label`.

## Examples

### HTML

#### Don't

```html
<input type="text" name="q">
```

#### Do

```html
<label for="q">Search</label>
<input type="text" id="q" name="q">
```
