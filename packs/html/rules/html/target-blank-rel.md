---
id: html/target-blank-rel
title: "Links that open a new tab set rel=\"noopener\""
languages: [html]
fix: auto
stability: preview
since: 0.1.0
---

An `<a>` or `<area>` with `target="_blank"` and an `href` (or a `<form>` with an `action`) has `rel="noopener"` or `rel="noreferrer"`.

**Why:** Without it, older browsers give the opened page a handle on this one (`window.opener`), which it can use to redirect it.

```codelaw
native: html/target-blank-rel
```

## Examples

### HTML

#### Don't

```html
<a href="https://example.com" target="_blank">Example</a>
```

#### Do

```html
<a href="https://example.com" target="_blank" rel="noopener">Example</a>
```
