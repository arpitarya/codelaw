---
id: sec-js/no-weak-hash
title: "No MD5 or SHA-1"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the digest length changes, so what stores or compares it has to change too"
stability: preview
ported_from: [bandit/B324, ruff/S324]
security: { cwe: [327], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`createHash`, `createHmac` and `crypto.subtle.digest` are not given `md5`, `sha1`, `ripemd160` or another broken hash.

**Why:** MD5 and SHA-1 collide: two inputs with one hash can be made, so a signature, a certificate fingerprint or a password hash built on them can be forged (CWE-327).

```codelaw
match:
  any:
    - pattern: createHash($A)
    - pattern: createHash($A, $$$REST)
    - pattern: crypto.createHash($A)
    - pattern: crypto.createHash($A, $$$REST)
    - pattern: createHmac($A)
    - pattern: createHmac($A, $$$REST)
    - pattern: crypto.createHmac($A)
    - pattern: crypto.createHmac($A, $$$REST)
    - pattern: $O.subtle.digest($A)
    - pattern: $O.subtle.digest($A, $$$REST)
constraints:
  A: { regex: '(?i)^["''`]\s*(md[245]|sha-?1|ripemd-?160?|rmd160|whirlpool)\s*["''`]$' }
message: "`$A` is a broken hash; two inputs can be made to share it"
```

## How to fix

1. Use SHA-256 or SHA-512 (`createHash("sha256")`) for digests and `createHmac("sha256", key)` for keyed ones.
2. For passwords use a password hash: `scrypt`, `argon2id` or `bcrypt` (see `sec-js/password-hash-effort`).
3. An MD5 used only as a cache key or a non-security checksum may stay: suppress the finding with a reason that says so.

## Examples

### JavaScript

#### Don't

```js
const digest = crypto.createHash("md5").update(data).digest("hex");
```

#### Do

```js
const digest = crypto.createHash("sha256").update(data).digest("hex");
```

### JSX

#### Don't

```jsx
const digest = crypto.createHash("md5").update(data).digest("hex");
```

#### Do

```jsx
const digest = crypto.createHash("sha256").update(data).digest("hex");
```

### TypeScript

#### Don't

```ts
const digest = crypto.createHash("md5").update(data).digest("hex");
```

#### Do

```ts
const digest = crypto.createHash("sha256").update(data).digest("hex");
```

### TSX

#### Don't

```tsx
const digest = crypto.createHash("md5").update(data).digest("hex");
```

#### Do

```tsx
const digest = crypto.createHash("sha256").update(data).digest("hex");
```
