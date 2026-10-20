---
id: html/valid-id
title: "Ids are well-formed"
languages: [html]
fix: manual
fix_reason: "renaming an id breaks the CSS, scripts and links that use it"
stability: preview
options:
  relaxed: { type: "boolean", default: false, description: "only require a non-empty id without whitespace" }
ported_from: [html-validate/valid-id]
since: 0.1.0
---

An `id` is not empty and has no whitespace; it starts with a letter and uses only letters, digits, `-` and `_`.

**Why:** Whitespace makes the id unusable; other characters need escaping in CSS selectors and are easy to get wrong.

```codelaw
native: html/valid-id
```

## How to fix

1. Rename the id, and update everything that refers to it.

## Examples

### HTML

#### Don't

```html
<div id="1st item"></div>
```

#### Do

```html
<div id="first-item"></div>
```
