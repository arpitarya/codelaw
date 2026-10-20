---
id: html/element-permitted-occurrences
title: "Elements that may appear once do"
languages: [html]
fix: manual
fix_reason: "which of the duplicates to keep is the author's call"
stability: preview
ported_from: [html-validate/element-permitted-occurrences]
since: 0.1.0
---

Children that may appear only once in their parent do: one `<caption>`, `<thead>` and `<tfoot>` per table, one `<title>` and `<base>` per head, one `<legend>` per fieldset.

**Why:** The browser uses the first one and ignores the rest.

```codelaw
native: html/element-permitted-occurrences
```

## How to fix

1. Merge the duplicates into one, or remove the extra one.

## Examples

### HTML

#### Don't

```html
<table>
  <caption>Prices</caption>
  <caption>2024</caption>
  <tr><td>1</td></tr>
</table>
```

#### Do

```html
<table>
  <caption>Prices, 2024</caption>
  <tr><td>1</td></tr>
</table>
```
