---
id: sec-html/no-javascript-url
title: "No javascript: URLs"
languages: [html]
fix: manual
fix_reason: "the script has to move out of the URL into a listener or a real link"
stability: preview
security: { cwe: [79], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

An `href`, `src`, `action`, `formaction`, `xlink:href` or `<object data>` is never a `javascript:` URL.

**Why:** A `javascript:` URL runs script from an attribute, so it is cross-site scripting waiting for one interpolated value, and a strict Content Security Policy blocks it.

```codelaw
native: sec-html/no-javascript-url
```

## How to fix

1. Replace a `javascript:` link with a `<button type="button">` and attach the listener with `addEventListener`.
2. Point `href`, `src`, `action` or `formaction` at a real URL.

## Examples

### HTML

#### Don't

```html
<a href="javascript:void(0)">Menu</a>
```

#### Do

```html
<button type="button" id="menu">Menu</button>
```
