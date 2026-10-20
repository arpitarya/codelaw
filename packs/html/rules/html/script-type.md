---
id: html/script-type
title: "No type on JavaScript scripts"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/script-type]
since: 0.1.0
---

A `<script>` doesn't say `type="text/javascript"` or another JavaScript MIME type.

**Why:** JavaScript is the default; the attribute adds nothing, and legacy types such as `text/ecmascript` mislead.

```codelaw
native: html/script-type
```

## Examples

### HTML

#### Don't

```html
<script type="text/javascript" src="app.js"></script>
```

#### Do

```html
<script src="app.js"></script>
```
