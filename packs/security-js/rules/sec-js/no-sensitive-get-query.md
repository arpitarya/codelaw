---
id: sec-js/no-sensitive-get-query
title: "No password, token or secret read from the query string"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the client has to change how it sends the value"
stability: preview
security: { cwe: [598], owasp: ["A04:2025"], severity: medium }
since: 0.3.0
---

`req.query` is not read for a password, token, secret, key or one-time code.

**Why:** A query string is written to access logs, browser history, proxies and the `Referer` header, so a secret in it is copied to all of them (CWE-598).

```codelaw
match:
  any:
    - pattern: req.query.$K
    - pattern: request.query.$K
    - pattern: ctx.query.$K
    - pattern: ctx.request.query.$K
constraints:
  K: { regex: '(?i)^(password|passwd|pwd|token|access_?token|refresh_?token|secret|api_?key|apikey|otp|pin|ssn|credit_?card|cc_?number|cvv)$' }
message: "`$K` is read from the query string, which is logged and kept in history"
```

## How to fix

1. Send the value in the body of a `POST` request, or in an `Authorization` header.
2. If a link must carry a one-time token (an e-mail verification), make it single-use and short-lived, and send `Referrer-Policy: no-referrer`.

## Examples

### JavaScript

#### Don't

```js
const password = req.query.password;
```

#### Do

```js
const password = req.body.password;
```

### JSX

#### Don't

```jsx
const password = req.query.password;
```

#### Do

```jsx
const password = req.body.password;
```

### TypeScript

#### Don't

```ts
const password = req.query.password;
```

#### Do

```ts
const password = req.body.password;
```

### TSX

#### Don't

```tsx
const password = req.query.password;
```

#### Do

```tsx
const password = req.body.password;
```
