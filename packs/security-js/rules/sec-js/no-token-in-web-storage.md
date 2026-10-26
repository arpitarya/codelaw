---
id: sec-js/no-token-in-web-storage
title: "No token kept in localStorage or sessionStorage"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the token lives instead is a change to the sign-in flow"
stability: preview
security: { cwe: [922], owasp: ["A07:2025"], severity: high }
since: 0.3.0
---

A token, a JWT, a secret or a password is not stored in `localStorage` or `sessionStorage`.

**Why:** Web storage is readable by every script on the origin, so one XSS bug or one compromised dependency reads the token and uses it from anywhere (CWE-922).

```codelaw
match:
  any:
    - all:
        - pattern: $S.setItem($K, $V)
        - regex: '(?i)^(window\.)?(localStorage|sessionStorage)\.setItem\(\s*["''`][^"''`]*(token|jwt|secret|password|credential|api[_-]?key|auth|session_?id)'
    - all:
        - pattern: $S2.$P = $V2
        - regex: '(?i)^(window\.)?(localStorage|sessionStorage)\.\w*(token|jwt|secret|password|credential|auth|session_?id)\w*\s*='
    - all:
        - pattern: $S3[$K3] = $V3
        - regex: '(?i)^(window\.)?(localStorage|sessionStorage)\[\s*["''`][^"''`]*(token|jwt|secret|password|credential|auth|session_?id)'
message: "A credential is kept in web storage, where every script on the page can read it"
```

## How to fix

1. Have the server set the token in a cookie with `HttpOnly`, `Secure` and `SameSite`, and let the browser send it.
2. If a script must hold a token, keep it in memory (a closure or a module variable) and refresh it from an `HttpOnly` cookie.

## Examples

### JavaScript

#### Don't

```js
localStorage.setItem("accessToken", token);
```

#### Do

```js
localStorage.setItem("theme", theme);
```

### JSX

#### Don't

```jsx
localStorage.setItem("accessToken", token);
```

#### Do

```jsx
localStorage.setItem("theme", theme);
```

### TypeScript

#### Don't

```ts
localStorage.setItem("accessToken", token);
```

#### Do

```ts
localStorage.setItem("theme", theme);
```

### TSX

#### Don't

```tsx
localStorage.setItem("accessToken", token);
```

#### Do

```tsx
localStorage.setItem("theme", theme);
```
