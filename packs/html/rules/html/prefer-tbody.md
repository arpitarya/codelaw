---
id: html/prefer-tbody
title: "Table rows are in <tbody>"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/prefer-tbody]
since: 0.1.0
---

A table's `<tr>` rows sit in a `<tbody>` (or `<thead>`/`<tfoot>`), not directly in `<table>`.

**Why:** Browsers insert the `<tbody>` anyway, so the markup doesn't match the DOM that CSS and scripts see.

```codelaw
native: html/prefer-tbody
```

## Examples

### HTML

#### Don't

```html
<table>
  <tr><td>1</td></tr>
</table>
```

#### Do

```html
<table>
  <tbody><tr><td>1</td></tr></tbody>
</table>
```

## Differences from the source

The fix is offered when the rows are next to each other; rows separated by other elements are only reported.
