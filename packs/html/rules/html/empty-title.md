---
id: html/empty-title
title: "<title> has text"
languages: [html]
fix: manual
fix_reason: "the page title needs a human"
stability: preview
ported_from: [html-validate/empty-title]
tags: [a11y]
since: 0.1.0
---

The `<title>` element has text.

**Why:** The title names the page in tabs, history, bookmarks, search results and screen readers.

```codelaw
native: html/empty-title
```

## How to fix

1. Write a short title that says what the page is.

## Examples

### HTML

#### Don't

```html
<title></title>
```

#### Do

```html
<title>Pricing — Acme</title>
```
