---
id: html/th-scope
title: "Header cells in complex tables have scope"
languages: [html]
fix: manual
fix_reason: "whether each header labels a row or a column is the author's call"
stability: preview
options:
  strict: { type: "boolean", default: false, description: "check simple tables too" }
ported_from: [html-validate/wcag/h63]
tags: [a11y]
since: 0.1.0
---

In a table whose headers are not just the first row or just the first column, every `<th>` has `scope` (`row`, `col`, `rowgroup` or `colgroup`).

**Why:** Screen readers announce each cell with its headers; `scope` tells them which headers belong to which cells.

```codelaw
native: html/th-scope
```

## How to fix

1. Add `scope="col"` to column headers and `scope="row"` to row headers.

## Examples

### HTML

#### Don't

```html
<table>
  <tr><th>Name</th><th>Age</th></tr>
  <tr><th>Ada</th><td>36</td></tr>
</table>
```

#### Do

```html
<table>
  <tr><th scope="col">Name</th><th scope="col">Age</th></tr>
  <tr><th scope="row">Ada</th><td>36</td></tr>
</table>
```
