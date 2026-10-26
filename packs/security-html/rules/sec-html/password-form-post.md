---
id: sec-html/password-form-post
title: "A form with a password sends it with POST over HTTPS"
languages: [html]
fix: suggest
fix_reason: "a server that reads only the query string has to change with the form"
stability: preview
security: { cwe: [598], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

A form holding a password field sets `method="post"` and does not post to an `http:` URL.

**Why:** A form with no `method` or with `method="get"` puts the password in the URL, where history, logs and referrers keep it; an `http:` action sends it in the clear.

```codelaw
native: sec-html/password-form-post
```

## Examples

### HTML

#### Don't

```html
<form action="/login" method="get"><input type="password" name="pw"></form>
```

#### Do

```html
<form action="/login" method="post"><input type="password" name="pw"></form>
```
