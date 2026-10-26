---
id: sec-html/no-secret-in-hidden-input
title: "No secret in a hidden input"
languages: [html]
fix: manual
fix_reason: "where the secret should live is a design decision"
stability: preview
security: { cwe: [798, 200], owasp: ["A07:2025"], severity: high }
since: 0.2.0
---

A hidden input named like a password, token, secret or API key does not hold a literal value (CSRF tokens are excluded).

**Why:** A hidden field is part of the page everyone receives, so a secret written into it is disclosed to every visitor.

```codelaw
native: sec-html/no-secret-in-hidden-input
```

## How to fix

1. Keep the secret on the server and send a session or a reference instead.
2. If the field is an anti-forgery token, name it `csrf_token`, `_csrf` or the framework's own name.

## Examples

### HTML

#### Don't

```html
<input type="hidden" name="api_key" value="sk_live_51Habc">
```

#### Do

```html
<input type="hidden" name="csrf_token" value="{{ csrf_token }}">
```
