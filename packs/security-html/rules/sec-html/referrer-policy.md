---
id: sec-html/referrer-policy
title: "No referrer policy that leaks full URLs"
languages: [html]
fix: suggest
fix_reason: "a partner may rely on the full referrer; the safer value is a suggestion"
stability: preview
security: { cwe: [200], owasp: ["A02:2025"], severity: low }
since: 0.2.0
---

A `referrerpolicy` attribute or `<meta name="referrer">` is not `unsafe-url` or `no-referrer-when-downgrade`.

**Why:** Both send the full URL of the page, query string included, to other sites, and a URL often holds a token or an identifier.

```codelaw
native: sec-html/referrer-policy
```

## Examples

### HTML

#### Don't

```html
<a href="https://other.example/" referrerpolicy="unsafe-url">Other</a>
```

#### Do

```html
<a href="https://other.example/" referrerpolicy="strict-origin-when-cross-origin">Other</a>
```
