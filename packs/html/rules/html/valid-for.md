---
id: html/valid-for
title: "A label points at a control"
languages: [html]
fix: manual
fix_reason: "either the id or the element is wrong, and only the author knows which"
stability: preview
ported_from: [html-validate/valid-for]
since: 0.1.0
---

A `<label for>` refers to the id of a form control a label can name (`input`, `select`, `textarea`, `button`, `meter`, `output`, `progress`).

**Why:** Pointed at a `<div>` or `<span>`, the label names nothing, and clicking it does nothing.

```codelaw
native: html/valid-for
```

## How to fix

1. Point `for` at the control's id, or make the target a form control.

## Examples

### HTML

#### Don't

```html
<label for="name">Name</label>
<div id="name"></div>
```

#### Do

```html
<label for="name">Name</label>
<input type="text" id="name">
```
