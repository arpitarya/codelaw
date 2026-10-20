---
id: html/long-title
title: "Page titles are short"
languages: [html]
fix: manual
fix_reason: "shortening a title needs a human"
stability: preview
options:
  maxlength: { type: "integer", default: 70, description: "the longest title allowed, in characters" }
ported_from: [html-validate/long-title]
since: 0.1.0
---

The `<title>` text is at most 70 characters.

**Why:** Search results and browser tabs cut long titles, often before the words that matter.

```codelaw
native: html/long-title
```

## How to fix

1. Shorten the title to what identifies the page.

## Examples

### HTML

#### Don't

```html
<title>Acme Corporation — Products, Services, Pricing, Support, Downloads and Contact Details</title>
```

#### Do

```html
<title>Pricing — Acme</title>
```
