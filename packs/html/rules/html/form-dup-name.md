---
id: html/form-dup-name
title: "Form controls have unique names"
languages: [html]
fix: manual
fix_reason: "renaming a control changes what the server receives"
stability: preview
options:
  allowArrayBrackets: { type: "boolean", default: true, description: "let `name[]` repeat" }
  allowCheckboxDefault: { type: "boolean", default: true, description: "allow a hidden input before a checkbox of the same name" }
  shared: { type: "string[]", default: ["radio", "button", "reset", "submit"], description: "control types that may share a name" }
ported_from: [html-validate/form-dup-name]
since: 0.1.0
---

Within a form, no two controls share a `name`, except radio buttons of one group, buttons, and a hidden default before a checkbox. A `name[]` array name is not mixed with the plain `name`.

**Why:** The server receives both values under one name and usually keeps only one of them.

```codelaw
native: html/form-dup-name
```

## How to fix

1. Give each control its own name, and update the code that reads the form.

## Examples

### HTML

#### Don't

```html
<form>
  <input type="text" name="email">
  <input type="email" name="email">
</form>
```

#### Do

```html
<form>
  <input type="text" name="name">
  <input type="email" name="email">
</form>
```
