---
id: html/form-submit
title: "Forms have a submit button"
languages: [html]
fix: manual
fix_reason: "the button's label and placement need a human"
stability: preview
ported_from: [html-validate/wcag/h32]
tags: [a11y]
since: 0.1.0
---

A `<form>` has a submit button, inside it or associated with it by `form=`.

**Why:** Without one there is no visible way to send the form; Enter-to-submit isn't obvious and doesn't work everywhere.

```codelaw
native: html/form-submit
```

## How to fix

1. Add a `<button type="submit">` with a label that says what it does.

## Examples

### HTML

#### Don't

```html
<form action="/search">
  <input type="search" name="q" aria-label="Search">
</form>
```

#### Do

```html
<form action="/search">
  <input type="search" name="q" aria-label="Search">
  <button type="submit">Search</button>
</form>
```
