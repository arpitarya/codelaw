---
id: html/element-permitted-order
title: "Children come in the order HTML requires"
languages: [html]
fix: manual
fix_reason: "moving markup may change how it is styled"
stability: preview
ported_from: [html-validate/element-permitted-order]
since: 0.1.0
---

Children with a required order are in it: in a table, `<caption>` then `<colgroup>`, `<thead>`, `<tbody>`/`<tr>`, `<tfoot>`; `<summary>` first in `<details>`; `<legend>` first in `<fieldset>`; `<source>` before `<img>` in `<picture>`.

**Why:** Out of order, browsers ignore or move the element, so it doesn't do its job.

```codelaw
native: html/element-permitted-order
```

## How to fix

1. Move the element named in the message before the one it must precede.

## Examples

### HTML

#### Don't

```html
<details>
  <p>More text</p>
  <summary>Read more</summary>
</details>
```

#### Do

```html
<details>
  <summary>Read more</summary>
  <p>More text</p>
</details>
```
