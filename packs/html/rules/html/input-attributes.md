---
id: html/input-attributes
title: "Input attributes fit the input type"
languages: [html]
fix: suggest
fix_reason: "scripts or attribute selectors may still read the attribute"
stability: preview
ported_from: [html-validate/input-attributes]
since: 0.1.0
---

An `<input>` has only attributes its `type` uses: no `accept` on a text field, no `placeholder` on a checkbox, no `min` on an email field.

**Why:** The browser ignores them, so they mislead readers about how the field behaves.

```codelaw
native: html/input-attributes
```

## Examples

### HTML

#### Don't

```html
<input type="text" name="photo" accept="image/png">
```

#### Do

```html
<input type="text" name="photo">
```
