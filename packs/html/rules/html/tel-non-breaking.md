---
id: html/tel-non-breaking
title: "Phone numbers don't wrap"
languages: [html]
fix: suggest
fix_reason: "the text then contains non-breaking characters, which copy and search keep"
stability: preview
options:
  ignoreClasses: { type: "string[]", default: [], description: "classes that already prevent wrapping" }
  ignoreStyle: { type: "boolean", default: true, description: "skip links styled `white-space: nowrap` or `pre`" }
ported_from: [html-validate/tel-non-breaking]
since: 0.1.0
---

The text of a `tel:` link uses non-breaking spaces and hyphens.

**Why:** A phone number broken over two lines is hard to read and to copy.

```codelaw
native: html/tel-non-breaking
```

## Examples

### HTML

#### Don't

```html
<a href="tel:5551234">555 1234</a>
```

#### Do

```html
<a href="tel:5551234">555&nbsp;1234</a>
```
