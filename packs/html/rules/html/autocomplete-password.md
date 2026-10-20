---
id: html/autocomplete-password
title: "Password fields say which password they want"
languages: [html]
fix: manual
fix_reason: "whether the field asks for a new or the current password depends on the form"
stability: preview
options:
  preferred: { type: "string", default: "", description: "one value every password field must use, such as `current-password`" }
ported_from: [html-validate/autocomplete-password]
since: 0.1.0
---

An `<input type="password">` has `autocomplete`, set to `new-password` or `current-password`, not `off`.

**Why:** Password managers autofill password fields anyway; saying which kind stops them filling a saved password into a "new password" field.

```codelaw
native: html/autocomplete-password
```

## How to fix

1. Use `autocomplete="current-password"` on a sign-in form.
2. Use `autocomplete="new-password"` on a sign-up or change-password form.

## Examples

### HTML

#### Don't

```html
<input type="password" name="pw">
```

#### Do

```html
<input type="password" name="pw" autocomplete="current-password">
```
