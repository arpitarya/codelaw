---
id: sec-js/express-session-secret
title: "express-session has a secret from the environment and a secure cookie"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the secret comes from is a deployment decision"
stability: preview
security: { cwe: [798, 614], owasp: ["A07:2025"], severity: high }
since: 0.3.0
---

`express-session`, `cookie-session` and `session()` are not given a literal `secret`, and their cookie does not set `secure: false`.

**Why:** A secret in the source is in every clone and every build; anyone who has it forges a session cookie, and `secure: false` lets the cookie travel over plain HTTP (CWE-798, CWE-614).

```codelaw
match:
  any:
    - all:
        - any:
            - pattern: session($$$A)
            - pattern: expressSession($$$A)
            - pattern: cookieSession($$$A)
        - has:
            kind: pair
            stopBy: end
            regex: '^["'']?(secret|keys?)["'']?\s*:\s*(["'']|\[\s*["''])'
    - all:
        - any:
            - pattern: session($$$B)
            - pattern: expressSession($$$B)
            - pattern: cookieSession($$$B)
        - has:
            kind: pair
            stopBy: end
            regex: '^["'']?secure["'']?\s*:\s*false\s*$'
message: "The session secret is a literal in the source, or the session cookie is not marked secure"
```

## How to fix

1. Read the secret from the environment or a secret store, and rotate it if it was ever committed.
2. Set `cookie: { secure: true, httpOnly: true, sameSite: "lax" }` in production (behind a proxy, set `trust proxy`).

## Examples

### JavaScript

#### Don't

```js
app.use(session({ secret: "keyboard cat", resave: false }));
```

#### Do

```js
app.use(session({ secret: process.env.SESSION_SECRET, resave: false }));
```

### JSX

#### Don't

```jsx
app.use(session({ secret: "keyboard cat", resave: false }));
```

#### Do

```jsx
app.use(session({ secret: process.env.SESSION_SECRET, resave: false }));
```

### TypeScript

#### Don't

```ts
app.use(session({ secret: "keyboard cat", resave: false }));
```

#### Do

```ts
app.use(session({ secret: process.env.SESSION_SECRET, resave: false }));
```

### TSX

#### Don't

```tsx
app.use(session({ secret: "keyboard cat", resave: false }));
```

#### Do

```tsx
app.use(session({ secret: process.env.SESSION_SECRET, resave: false }));
```
