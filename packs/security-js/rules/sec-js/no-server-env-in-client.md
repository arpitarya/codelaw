---
id: sec-js/no-server-env-in-client
title: "No server environment variable in a client component, no secret in a public one"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the secret is used from decides how the code is split"
stability: preview
security: { cwe: [200], owasp: ["A02:2025"], severity: high }
since: 0.3.0
---

A `"use client"` file reads only `NEXT_PUBLIC_*` variables from `process.env`, and no public variable (`NEXT_PUBLIC_*`, `VITE_*`) is named like a secret.

**Why:** Whatever a client file reads is bundled and sent to every browser: a server variable is either `undefined` there or, if the name is public, readable by anyone (CWE-200).

```codelaw
match:
  any:
    - all:
        - pattern: process.env.$N
        - not:
            regex: '^process\.env\.(NEXT_PUBLIC_|NODE_ENV$)'
        - inside:
            kind: program
            stopBy: end
            has:
              kind: expression_statement
              regex: '^["'']use client["'']'
    - all:
        - pattern: process.env.$K
        - regex: '^process\.env\.NEXT_PUBLIC_\w*(SECRET|PRIVATE|PASSWORD|TOKEN|SERVICE_ROLE)'
    - all:
        - pattern: import.meta.env.$K2
        - regex: '^import\.meta\.env\.VITE_\w*(SECRET|PRIVATE|PASSWORD|TOKEN|SERVICE_ROLE)'
message: "This variable is bundled into the browser, or should not be public"
```

## How to fix

1. Read secrets only in server code (a route handler, a server component, a server action) and pass the client the result.
2. Name a variable `NEXT_PUBLIC_` (or `VITE_`) only when every visitor may read it; rename a secret and rotate it if it was ever public.

## Examples

### JavaScript

#### Don't

```js
"use client";
const key = process.env.STRIPE_SECRET_KEY;
```

#### Do

```js
"use client";
const url = process.env.NEXT_PUBLIC_API_URL;
```

### JSX

#### Don't

```jsx
"use client";
const key = process.env.STRIPE_SECRET_KEY;
```

#### Do

```jsx
"use client";
const url = process.env.NEXT_PUBLIC_API_URL;
```

### TypeScript

#### Don't

```ts
"use client";
const key = process.env.STRIPE_SECRET_KEY;
```

#### Do

```ts
"use client";
const url = process.env.NEXT_PUBLIC_API_URL;
```

### TSX

#### Don't

```tsx
"use client";
const key = process.env.STRIPE_SECRET_KEY;
```

#### Do

```tsx
"use client";
const url = process.env.NEXT_PUBLIC_API_URL;
```
