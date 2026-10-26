---
id: sec-js/no-weak-key-size
title: "No short RSA key, no weak elliptic curve"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "keys already issued stay weak until they are replaced, and a larger key is slower to generate and use"
stability: preview
ported_from: [bandit/B505, ruff/S505]
security: { cwe: [326], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`generateKeyPair` and `generateKeyPairSync` are given an RSA `modulusLength` of at least 2048, and no weak `namedCurve`.

**Why:** A 1024-bit RSA key can be factored with the resources of a large organisation; curves below 224 bits are within reach of a determined attacker (CWE-326).

```codelaw
match:
  any:
    - pattern:
        context: '({ modulusLength: $N })'
        selector: pair
    - pattern:
        context: '({ namedCurve: $C })'
        selector: pair
constraints:
  N: { regex: '^([0-9]{1,3}|1[0-9]{3}|20[0-3][0-9]|204[0-7])$' }
  C: { regex: '(?i)^["'']\s*(secp(112|128|160|192)\w*|prime192v\d|P-192|P-160|sect\d{2,3}\w*|brainpool\w*160\w*)\s*["'']$' }
message: "This key is too short or uses a weak curve"
fix:
  - replace: { target: $N, with: "2048" }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 1024 });
```

#### Do

```js
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

#### Fixed

```js
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

### JSX

#### Don't

```jsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 1024 });
```

#### Do

```jsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

#### Fixed

```jsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

### TypeScript

#### Don't

```ts
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 1024 });
```

#### Do

```ts
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

#### Fixed

```ts
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

### TSX

#### Don't

```tsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 1024 });
```

#### Do

```tsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```

#### Fixed

```tsx
const { publicKey } = crypto.generateKeyPairSync("rsa", { modulusLength: 2048 });
```
