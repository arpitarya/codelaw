---
id: sec-js/password-hash-effort
title: "A password is stored with a slow, salted hash"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the parameters that fit the hardware are a decision for the author, and stored hashes have to be re-made"
stability: preview
security: { cwe: [916], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

A password is not hashed with `createHash`, `pbkdf2` is given at least 600 000 iterations (SHA-256), and `bcrypt` a cost of at least 10.

**Why:** A fast hash lets an attacker who steals the table try billions of passwords a second; the work factor is what makes each guess cost something (CWE-916).

```codelaw
match:
  any:
    - all:
        - pattern: $H.update($P)
        - regex: '(?i)createHash\([^)]*\)\.update\(\s*[\w$.]*(password|passwd|pwd)[\w$.]*\s*\)'
    - pattern: pbkdf2Sync($PW, $SALT, $IT, $$$R)
    - pattern: $O.pbkdf2Sync($PW, $SALT, $IT, $$$R)
    - pattern: pbkdf2($PW, $SALT, $IT, $$$R)
    - pattern: $O.pbkdf2($PW, $SALT, $IT, $$$R)
    - pattern: bcrypt.hash($PW2, $COST)
    - pattern: bcrypt.hashSync($PW2, $COST)
    - pattern: bcrypt.hash($PW2, $COST, $$$R)
constraints:
  IT: { regex: '^([0-9]{1,5}|[1-5][0-9]{5}|[0-9]{1,3}_[0-9]{3})$' }
  COST: { regex: '^[0-9]$' }
message: "This password hash is too fast: an attacker with the table can try guesses quickly"
```

## How to fix

1. Prefer `argon2id` (the `argon2` package) or `bcrypt` with a cost of 12, `scrypt` with N=2^17.
2. With `pbkdf2` use SHA-256 with 600 000 iterations or more, and a random 16-byte salt per password.
3. Never store `createHash(...).update(password)`; re-hash existing passwords at the next sign-in.

## Examples

### JavaScript

#### Don't

```js
const hash = crypto.pbkdf2Sync(password, salt, 1000, 32, "sha256");
```

#### Do

```js
const hash = crypto.pbkdf2Sync(password, salt, 600000, 32, "sha256");
```

### JSX

#### Don't

```jsx
const hash = crypto.pbkdf2Sync(password, salt, 1000, 32, "sha256");
```

#### Do

```jsx
const hash = crypto.pbkdf2Sync(password, salt, 600000, 32, "sha256");
```

### TypeScript

#### Don't

```ts
const hash = crypto.pbkdf2Sync(password, salt, 1000, 32, "sha256");
```

#### Do

```ts
const hash = crypto.pbkdf2Sync(password, salt, 600000, 32, "sha256");
```

### TSX

#### Don't

```tsx
const hash = crypto.pbkdf2Sync(password, salt, 1000, 32, "sha256");
```

#### Do

```tsx
const hash = crypto.pbkdf2Sync(password, salt, 600000, 32, "sha256");
```
