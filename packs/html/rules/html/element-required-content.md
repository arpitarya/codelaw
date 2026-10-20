---
id: html/element-required-content
title: "Elements contain the children HTML requires"
languages: [html]
fix: manual
fix_reason: "the content of the missing child needs a human"
stability: preview
ported_from: [html-validate/element-required-content, htmlhint/title-require]
since: 0.1.0
---

Elements that need a particular child have it: `<html>` has `<head>` and `<body>`, `<head>` has `<title>`, `<details>` has `<summary>`.

**Why:** A page without a `<title>` has no name in tabs, bookmarks and screen readers; a `<details>` without `<summary>` gets a generic label.

```codelaw
native: html/element-required-content
```

## How to fix

1. Add the child the message names, with real content.

## Examples

### HTML

#### Don't

```html
<head>
  <meta charset="utf-8">
</head>
```

#### Do

```html
<head>
  <meta charset="utf-8">
  <title>Pricing</title>
</head>
```
