---
id: sec-js/no-insecure-random
title: "No Math.random for a token, a secret or a password"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "the token's length and alphabet change, and anything that checks its shape has to follow; a numeric code needs `crypto.randomInt` instead"
stability: preview
ported_from: [eslint-plugin-security/detect-pseudoRandomBytes, bandit/B311, ruff/S311]
security: { cwe: [338], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`Math.random()` is not used to make a token, a secret, a password, a nonce, a salt or a session id, and `crypto.pseudoRandomBytes` is not used.

**Why:** `Math.random()` is not random enough to guess-proof anything: its next value can be worked out from a few earlier ones (CWE-338); `pseudoRandomBytes` may return predictable bytes.

```codelaw
match:
  any:
    - all:
        - pattern:
            context: 'const $N = $E'
            selector: variable_declarator
        - regex: '(?i)^[\w$]*(token|secret|passw(or)?d|passwd|pwd|nonce|salt|otp|csrf|session|api_?key|uuid|guid|reset|invite|verification)[\w$]*\s*=\s*Math\.random\(\)\.toString\(\d+\)(\.(slice|substring|substr)\(\d+(,\s*\d+)?\))?$'
    - all:
        - pattern: $N = $E
        - regex: '(?i)^[\w$.]*(token|secret|passw(or)?d|passwd|pwd|nonce|salt|otp|csrf|session|api_?key|uuid|guid|reset|invite|verification)[\w$]*\s*=\s*Math\.random\(\)\.toString\(\d+\)(\.(slice|substring|substr)\(\d+(,\s*\d+)?\))?$'
    - all:
        - kind: variable_declarator
        - regex: '(?is)^[\w$]*(token|secret|passw(or)?d|passwd|pwd|nonce|salt|otp|csrf|session|api_?key|uuid|guid|reset|invite|verification)[\w$]*\s*=.*Math\.random\(\)'
    - all:
        - kind: assignment_expression
        - regex: '(?is)^[\w$.]*(token|secret|passw(or)?d|passwd|pwd|nonce|salt|otp|csrf|session|api_?key|uuid|guid|reset|invite|verification)[\w$]*\s*=.*Math\.random\(\)'
    - all:
        - kind: pair
        - regex: '(?is)^["'']?[\w$]*(token|secret|passw(or)?d|passwd|pwd|nonce|salt|otp|csrf|session|api_?key|uuid|guid|reset|invite|verification)[\w$]*["'']?\s*:.*Math\.random\(\)'
    - pattern: crypto.pseudoRandomBytes($$$P)
    - pattern: pseudoRandomBytes($$$P)
message: "This value is meant to be unguessable, but `Math.random()` can be predicted"
fix:
  - replace: { target: $E, with: "crypto.randomUUID()" }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const token = Math.random().toString(36).slice(2);
```

#### Do

```js
const token = crypto.randomUUID();
```

#### Fixed

```js
const token = crypto.randomUUID();
```

### JSX

#### Don't

```jsx
const token = Math.random().toString(36).slice(2);
```

#### Do

```jsx
const token = crypto.randomUUID();
```

#### Fixed

```jsx
const token = crypto.randomUUID();
```

### TypeScript

#### Don't

```ts
const token = Math.random().toString(36).slice(2);
```

#### Do

```ts
const token = crypto.randomUUID();
```

#### Fixed

```ts
const token = crypto.randomUUID();
```

### TSX

#### Don't

```tsx
const token = Math.random().toString(36).slice(2);
```

#### Do

```tsx
const token = crypto.randomUUID();
```

#### Fixed

```tsx
const token = crypto.randomUUID();
```
