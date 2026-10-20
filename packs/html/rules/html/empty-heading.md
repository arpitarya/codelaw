---
id: html/empty-heading
title: "Headings have text"
languages: [html]
fix: manual
fix_reason: "the heading text needs a human"
stability: preview
ported_from: [html-validate/empty-heading]
tags: [a11y]
since: 0.1.0
---

Every `<h1>`–`<h6>` has text, or an image with `alt` text.

**Why:** Screen-reader users navigate by headings; an empty heading is a stop with nothing to announce.

```codelaw
native: html/empty-heading
```

## How to fix

1. Give the heading text, or remove it if it only made space (use CSS for that).

## Examples

### HTML

#### Don't

```html
<h2></h2>
<h2><span hidden>Pricing</span></h2>
```

#### Do

```html
<h2>Pricing</h2>
```
