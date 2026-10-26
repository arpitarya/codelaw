---
id: sec-js/no-hardcoded-crypto-key
title: "No key or IV written into the source"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "where the key comes from instead is a deployment decision"
stability: preview
security: { cwe: [321, 329], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`createCipheriv`, `createHmac` and `jwt.sign` are not given a literal key, and `createCipheriv` is not given a literal or a zero IV.

**Why:** A key in the source is in every clone and build, so whoever reads the code decrypts the data or forges the signature; a fixed IV makes equal messages produce equal ciphertext and breaks GCM completely (CWE-321, CWE-329).

```codelaw
match:
  any:
    - all:
        - kind: call_expression
        - regex: '(createCipheriv|createDecipheriv)\(\s*[^,]+,\s*(["''`]|Buffer\.from\(\s*["''`])'
    - all:
        - kind: call_expression
        - regex: '(createCipheriv|createDecipheriv)\(\s*(?:[^,()]|\([^()]*\))+,\s*(?:[^,()]|\([^()]*\))+,\s*(["''`]|Buffer\.from\(\s*["''`]|Buffer\.alloc\(\s*\d+\s*\)|Buffer\.from\(\[)'
    - all:
        - kind: call_expression
        - regex: '\bcreateHmac\(\s*[^,]+,\s*(["''`]|Buffer\.from\(\s*["''`])'
    - all:
        - kind: call_expression
        - regex: '\b(jwt|jsonwebtoken)\.sign\(\s*[^,]+,\s*["''`]'
message: "A key or IV is a literal here"
```

## How to fix

1. Read the key from the environment or a key-management service, and rotate it if it was committed.
2. Make a new random IV for every message with `crypto.randomBytes(12)` and store it beside the ciphertext.
3. Give `jwt.sign` and `createHmac` a key from the environment, not a string in the code.

## Examples

### JavaScript

#### Don't

```js
const cipher = crypto.createCipheriv("aes-256-gcm", "0123456789abcdef0123456789abcdef", iv);
```

#### Do

```js
const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(process.env.DATA_KEY, "base64"), crypto.randomBytes(12));
```

### JSX

#### Don't

```jsx
const cipher = crypto.createCipheriv("aes-256-gcm", "0123456789abcdef0123456789abcdef", iv);
```

#### Do

```jsx
const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(process.env.DATA_KEY, "base64"), crypto.randomBytes(12));
```

### TypeScript

#### Don't

```ts
const cipher = crypto.createCipheriv("aes-256-gcm", "0123456789abcdef0123456789abcdef", iv);
```

#### Do

```ts
const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(process.env.DATA_KEY, "base64"), crypto.randomBytes(12));
```

### TSX

#### Don't

```tsx
const cipher = crypto.createCipheriv("aes-256-gcm", "0123456789abcdef0123456789abcdef", iv);
```

#### Do

```tsx
const cipher = crypto.createCipheriv("aes-256-gcm", Buffer.from(process.env.DATA_KEY, "base64"), crypto.randomBytes(12));
```
