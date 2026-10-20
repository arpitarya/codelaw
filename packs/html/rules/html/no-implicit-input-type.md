---
id: html/no-implicit-input-type
title: "<input> says what type it is"
languages: [html]
fix: suggest
fix_reason: "`type=\"text\"` is the default, but the field may have been meant as another type"
stability: preview
ported_from: [html-validate/no-implicit-input-type]
since: 0.1.0
---

An `<input>` has a `type` attribute.

**Why:** The default, `text`, is easy to leave by accident where `email`, `number` or `search` was meant.

```codelaw
native: html/no-implicit-input-type
```

## Examples

### HTML

#### Don't

```html
<input name="q">
```

#### Do

```html
<input type="text" name="q">
```
