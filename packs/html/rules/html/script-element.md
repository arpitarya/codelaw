---
id: html/script-element
title: "<script> has an end tag"
languages: [html]
fix: suggest
fix_reason: "the markup the open script swallowed starts being rendered"
stability: preview
ported_from: [html-validate/script-element]
since: 0.1.0
---

A `<script>` is closed with `</script>`, also when it has `src`; `<script … />` is not a closed script.

**Why:** HTML ignores the `/>`, so the browser reads everything up to the next `</script>` as script and the content in between disappears.

```codelaw
native: html/script-element
```

## Examples

### HTML

#### Don't

```html
<script src="app.js" />
<p>Hello</p>
```

#### Do

```html
<script src="app.js"></script>
<p>Hello</p>
```
