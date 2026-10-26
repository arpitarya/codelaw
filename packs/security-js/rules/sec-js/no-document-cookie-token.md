---
id: sec-js/no-document-cookie-token
title: "No token or session written to document.cookie"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the cookie has to be set by the server, which is a change to the response"
stability: preview
security: { cwe: [1004], owasp: ["A07:2025"], severity: medium }
since: 0.3.0
---

A token, a session id or a password is not written to `document.cookie`; the server sets it with `Set-Cookie`.

**Why:** A cookie script can write is a cookie script can read, so it has no `HttpOnly` flag, and any XSS reads it (CWE-1004).

```codelaw
match:
  any:
    - all:
        - pattern: document.cookie = $V
        - regex: '(?i)(token|jwt|session|auth|secret|password|sid)\w*\s*='
    - all:
        - pattern: document.cookie += $W
        - regex: '(?i)(token|jwt|session|auth|secret|password|sid)\w*\s*='
message: "A credential is written to a cookie script can read"
```

## How to fix

1. Return the credential in the response and let the server send `Set-Cookie: token=…; HttpOnly; Secure; SameSite=Lax`.
2. Keep the cookie out of script's reach: the page calls the API and the browser attaches the cookie.

## Examples

### JavaScript

#### Don't

```js
document.cookie = "token=" + token + "; path=/";
```

#### Do

```js
document.cookie = "theme=dark; path=/";
```

### JSX

#### Don't

```jsx
document.cookie = "token=" + token + "; path=/";
```

#### Do

```jsx
document.cookie = "theme=dark; path=/";
```

### TypeScript

#### Don't

```ts
document.cookie = "token=" + token + "; path=/";
```

#### Do

```ts
document.cookie = "theme=dark; path=/";
```

### TSX

#### Don't

```tsx
document.cookie = "token=" + token + "; path=/";
```

#### Do

```tsx
document.cookie = "theme=dark; path=/";
```
