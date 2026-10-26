---
id: sec-js/no-weak-cipher
title: "No createCipher, no DES, RC4, Blowfish or ECB"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the key, the IV and the stored format have to change together"
stability: preview
ported_from: [bandit/B304, ruff/S304]
security: { cwe: [327], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`createCipher` and `createDecipher` (no IV) are not used, and `createCipheriv` is not given DES, RC4, Blowfish, or an ECB mode.

**Why:** `createCipher` derives its key with MD5 and no salt and reuses a zero IV; ECB turns equal blocks into equal ciphertext; DES and RC4 are broken (CWE-327).

```codelaw
match:
  any:
    - pattern: createCipher($$$A)
    - pattern: $O.createCipher($$$A)
    - pattern: createDecipher($$$B)
    - pattern: $O.createDecipher($$$B)
    - pattern: createCipheriv($C, $$$R)
    - pattern: $O.createCipheriv($C, $$$R)
    - pattern: createDecipheriv($C, $$$R)
    - pattern: $O.createDecipheriv($C, $$$R)
constraints:
  C: { regex: '(?i)^["'']([a-z0-9-]*-ecb|des[a-z0-9-]*|rc[24][a-z0-9-]*|bf[a-z0-9-]*|blowfish[a-z0-9-]*|idea[a-z0-9-]*|cast5[a-z0-9-]*|seed[a-z0-9-]*)["'']$' }
message: "This cipher or mode is broken or leaves the key and IV to the library"
```

## How to fix

1. Use `createCipheriv("aes-256-gcm", key, iv)` with a random 12-byte IV per message, and keep the auth tag with the ciphertext.
2. Derive the key from a password with `scrypt` or `pbkdf2` and a random salt, not from the password directly.
3. Replace DES, RC4 and Blowfish with AES, and ECB with GCM (or CBC with a MAC).

## Examples

### JavaScript

#### Don't

```js
const cipher = crypto.createCipher("aes-256-cbc", password);
```

#### Do

```js
const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
```

### JSX

#### Don't

```jsx
const cipher = crypto.createCipher("aes-256-cbc", password);
```

#### Do

```jsx
const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
```

### TypeScript

#### Don't

```ts
const cipher = crypto.createCipher("aes-256-cbc", password);
```

#### Do

```ts
const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
```

### TSX

#### Don't

```tsx
const cipher = crypto.createCipher("aes-256-cbc", password);
```

#### Do

```tsx
const cipher = crypto.createCipheriv("aes-256-gcm", key, iv);
```
