---
id: html/no-abstract-role
title: "No abstract ARIA roles"
languages: [html]
fix: manual
fix_reason: "the concrete role depends on what the element is"
stability: preview
ported_from: [html-validate/no-abstract-role]
tags: [a11y]
since: 0.1.0
---

`role` does not name one of ARIA's abstract roles (`widget`, `input`, `landmark`, `section`, `command`, …).

**Why:** Abstract roles only organise the ARIA specification; browsers don't map them, so the element gets no role at all.

```codelaw
native: html/no-abstract-role
```

## How to fix

1. Use the concrete role that describes the element, or a native element.

## Examples

### HTML

#### Don't

```html
<div role="landmark">News</div>
```

#### Do

```html
<div role="region" aria-label="News">News</div>
```
