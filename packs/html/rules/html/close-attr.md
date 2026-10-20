---
id: html/close-attr
title: "End tags have no attributes"
languages: [html]
fix: auto
stability: preview
ported_from: [html-validate/close-attr]
since: 0.1.0
---

An end tag is just `</name>`, with no attributes.

**Why:** HTML ignores attributes on an end tag, so they are dead text that looks meaningful.

```codelaw
native: html/close-attr
```

## Examples

### HTML

#### Don't

```html
<p>Text</p class="intro">
```

#### Do

```html
<p>Text</p>
```
