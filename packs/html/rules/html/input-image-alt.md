---
id: html/input-image-alt
title: "Image buttons have alt text"
languages: [html]
fix: manual
fix_reason: "the text needs a human"
stability: preview
ported_from: [html-validate/wcag/h36]
tags: [a11y]
since: 0.1.0
---

An `<input type="image">` has non-empty `alt` text.

**Why:** The `alt` is the button's only name for screen readers and when the image doesn't load.

```codelaw
native: html/input-image-alt
```

## How to fix

1. Add `alt` saying what the button does.

## Examples

### HTML

#### Don't

```html
<input type="image" src="go.png">
```

#### Do

```html
<input type="image" src="go.png" alt="Search">
```
