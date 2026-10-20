---
id: html/element-required-attributes
title: "Elements have the attributes HTML requires"
languages: [html]
fix: manual
fix_reason: "the value can only come from the author"
stability: preview
ported_from: [html-validate/element-required-attributes, htmlhint/html-lang-require]
since: 0.1.0
---

Elements have their required attributes: `src` on `<img>`, `lang` on `<html>`, `title` on `<iframe>`, `href` (or `imagesrcset`) on `<link>`, and so on.

**Why:** Without them the element does nothing useful: an `<img>` without `src` shows nothing, and an `<html>` without `lang` makes screen readers guess the language.

```codelaw
native: html/element-required-attributes
```

## How to fix

1. Add the attribute the message names, with the right value.

## Examples

### HTML

#### Don't

```html
<html>
  <head><title>Home</title></head>
  <body><img alt="Logo"></body>
</html>
```

#### Do

```html
<html lang="en">
  <head><title>Home</title></head>
  <body><img src="logo.png" alt="Logo"></body>
</html>
```
