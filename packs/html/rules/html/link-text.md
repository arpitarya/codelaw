---
id: html/link-text
title: "Links have text"
languages: [html]
fix: manual
fix_reason: "the link text needs a human"
stability: preview
ported_from: [html-validate/wcag/h30]
tags: [a11y]
since: 0.1.0
---

A link with `href` has text that says where it goes: content, an image with `alt`, or `aria-label`.

**Why:** Screen readers announce a link by its text; without it the user hears only "link".

```codelaw
native: html/link-text
```

## How to fix

1. Give the link text, an image with `alt`, or an `aria-label`.

## Examples

### HTML

#### Don't

```html
<a href="/cart"><img src="cart.png" alt=""></a>
```

#### Do

```html
<a href="/cart"><img src="cart.png" alt="Cart"></a>
```
