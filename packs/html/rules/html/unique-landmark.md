---
id: html/unique-landmark
title: "Repeated landmarks have distinct names"
languages: [html]
fix: manual
fix_reason: "the names need a human"
stability: preview
ported_from: [html-validate/unique-landmark]
tags: [a11y]
since: 0.1.0
---

When a landmark appears more than once (two `<nav>`, two `<aside>`, …), each has a unique, non-empty `aria-label` or `aria-labelledby`.

**Why:** Screen-reader users jump between landmarks by name; two unnamed `<nav>` are indistinguishable.

```codelaw
native: html/unique-landmark
```

## How to fix

1. Give each landmark of the same kind its own `aria-label`, or point `aria-labelledby` at its heading.

## Examples

### HTML

#### Don't

```html
<nav><a href="/">Home</a></nav>
<nav><a href="/help">Help</a></nav>
```

#### Do

```html
<nav aria-label="Main"><a href="/">Home</a></nav>
<nav aria-label="Footer"><a href="/help">Help</a></nav>
```
