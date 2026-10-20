---
id: html/valid-autocomplete
title: "autocomplete values are valid"
languages: [html]
fix: manual
fix_reason: "the right field name depends on what the field collects"
stability: preview
ported_from: [html-validate/valid-autocomplete]
tags: [a11y]
since: 0.1.0
---

`autocomplete` holds a valid token list: optional `section-*`, then `shipping`/`billing`, then `home`/`work`/… for contact fields, then one field name that fits the input type, then optional `webauthn`. `<form>` takes only `on` or `off`.

**Why:** Browsers ignore an invalid value, so autofill and password managers can't help the user.

```codelaw
native: html/valid-autocomplete
```

## How to fix

1. Pick the field name that describes the data (the WHATWG autofill list).
2. Order the tokens as described above, or use `off`.

## Examples

### HTML

#### Don't

```html
<input type="email" name="email" autocomplete="mail">
<input type="number" name="n" autocomplete="email">
```

#### Do

```html
<input type="email" name="email" autocomplete="email">
<input type="number" name="n" autocomplete="off">
```
