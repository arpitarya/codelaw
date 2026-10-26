---
id: sec-js/jwt-verify
title: "A JWT is verified with a named algorithm, never `none`"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the algorithm to name is the one the issuer signs with"
stability: preview
security: { cwe: [347], owasp: ["A07:2025"], severity: high }
since: 0.3.0
---

`jwt.verify` is given an `algorithms` list, no algorithm is `none`, and `jwt.decode` is not the whole check.

**Why:** Without `algorithms` a token can pick its own: `none` carries no signature, and an RSA public key used as an HMAC secret lets anyone mint tokens (CWE-347). `decode` reads a token without checking it.

```codelaw
match:
  any:
    - pattern: $J.verify($T, $K)
    - all:
        - pattern: $J2.verify($T2, $K2, $O)
        - not:
            has:
              stopBy: end
              regex: 'algorithms'
    - all:
        - kind: pair
        - regex: '(?i)^["'']?algorithms?["'']?\s*:\s*\[?\s*["'']none["'']'
    - all:
        - pattern: $J3.decode($T3)
        - not:
            inside:
              kind: program
              stopBy: end
              has:
                stopBy: end
                regex: '\.verify\(|jwtVerify|verifyJwt'
constraints:
  J: { regex: '^(jwt|jsonwebtoken|jws)$' }
  J2: { regex: '^(jwt|jsonwebtoken|jws)$' }
  J3: { regex: '^(jwt|jsonwebtoken|jws|jwtDecode)$' }
message: "This JWT check lets the token choose its algorithm, accepts `none`, or does not verify"
```

## How to fix

1. Pass `{ algorithms: ["RS256"] }` (or the one algorithm you issue with) to `jwt.verify`.
2. Never put `none` in the list, and never use a public key as an HMAC secret.
3. Use `jwt.decode` only to read a token that `jwt.verify` has already accepted.

## Examples

### JavaScript

#### Don't

```js
const claims = jwt.verify(token, publicKey);
```

#### Do

```js
const claims = jwt.verify(token, publicKey, { algorithms: ["RS256"] });
```

### JSX

#### Don't

```jsx
const claims = jwt.verify(token, publicKey);
```

#### Do

```jsx
const claims = jwt.verify(token, publicKey, { algorithms: ["RS256"] });
```

### TypeScript

#### Don't

```ts
const claims = jwt.verify(token, publicKey);
```

#### Do

```ts
const claims = jwt.verify(token, publicKey, { algorithms: ["RS256"] });
```

### TSX

#### Don't

```tsx
const claims = jwt.verify(token, publicKey);
```

#### Do

```tsx
const claims = jwt.verify(token, publicKey, { algorithms: ["RS256"] });
```
