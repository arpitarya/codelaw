---
id: html/text-content
title: "Buttons have text"
languages: [html]
fix: manual
fix_reason: "the button's name needs a human"
stability: preview
ported_from: [html-validate/text-content]
tags: [a11y]
since: 0.1.0
---

A `<button>` has text a screen reader can announce: content, `aria-label`, `aria-labelledby`, or an image with `alt`.

**Why:** Without it the button is announced as just "button".

```codelaw
native: html/text-content
```

## How to fix

1. Add visible text, or an `aria-label` that says what the button does.

## Examples

### HTML

#### Don't

```html
<button type="button"><svg width="16" height="16"></svg></button>
```

#### Do

```html
<button type="button" aria-label="Close"><svg width="16" height="16"></svg></button>
```
