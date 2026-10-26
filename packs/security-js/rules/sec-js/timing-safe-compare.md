---
id: sec-js/timing-safe-compare
title: "A token, hash or signature is compared in constant time"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "`timingSafeEqual` throws when the two buffers differ in length, so the length has to be compared first"
stability: preview
ported_from: [eslint-plugin-security/detect-possible-timing-attacks]
security: { cwe: [208], owasp: ["A07:2025"], severity: medium }
since: 0.3.0
---

A value named like a token, an HMAC, a signature, a digest, a secret or a CSRF token is not compared with `===` or `==`: `crypto.timingSafeEqual` does it.

**Why:** `===` stops at the first byte that differs, so the time it takes tells an attacker how many leading bytes they got right, and they can guess a secret byte by byte (CWE-208).

```codelaw
match:
  any:
    - all:
        - pattern: $A === $B
        - regex: '(?i)\b\w*(token|hmac|signature|digest|secret|password|apikey|csrf)\w*\b'
        - not:
            regex: '(===?|!==?)\s*(null|undefined|""|\x27\x27|true|false|0)\b|^typeof\b|\btypeof\b|(?i)confirm|repeat|retype'
    - all:
        - pattern: $C !== $D
        - regex: '(?i)\b\w*(token|hmac|signature|digest|secret|password|apikey|csrf)\w*\b'
        - not:
            regex: '(===?|!==?)\s*(null|undefined|""|\x27\x27|true|false|0)\b|^typeof\b|\btypeof\b|(?i)confirm|repeat|retype'
message: "This comparison takes longer the more leading bytes match, which leaks the secret"
fix:
  - replace: "crypto.timingSafeEqual(Buffer.from($A), Buffer.from($B))"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
if (providedToken === expectedToken) {
  grant();
}
```

#### Do

```js
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

#### Fixed

```js
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

### JSX

#### Don't

```jsx
if (providedToken === expectedToken) {
  grant();
}
```

#### Do

```jsx
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

#### Fixed

```jsx
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

### TypeScript

#### Don't

```ts
if (providedToken === expectedToken) {
  grant();
}
```

#### Do

```ts
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

#### Fixed

```ts
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

### TSX

#### Don't

```tsx
if (providedToken === expectedToken) {
  grant();
}
```

#### Do

```tsx
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```

#### Fixed

```tsx
if (crypto.timingSafeEqual(Buffer.from(providedToken), Buffer.from(expectedToken))) {
  grant();
}
```
