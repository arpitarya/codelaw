---
id: sec-js/missing-rate-limit
title: "A login or reset route has a rate limit"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the limits that fit the route are a decision for the author"
stability: preview
security: { cwe: [770, 307], owasp: ["A07:2025"], severity: medium }
since: 0.3.0
---

A route that signs in, resets a password or checks a one-time code is registered with middleware, and the file mentions a rate limiter.

**Why:** Without a limit an attacker tries passwords, codes and reset tokens as fast as the server answers (CWE-307, CWE-770).

```codelaw
match:
  all:
    - pattern: $A.post($P, $H)
    - not:
        inside:
          kind: program
          stopBy: end
          has:
            stopBy: end
            regex: '(?i)rate-?limit|limiter|throttle|slow-?down'
constraints:
  P:
    kind: string
    regex: '(?i)login|signin|sign-in|reset|forgot|otp|verify|2fa|mfa|password|token'
  H:
    any:
      - kind: arrow_function
      - kind: function_expression
message: "This sign-in route has no rate limit"
```

## How to fix

1. Add `express-rate-limit` (or the framework's equivalent) to the route: a few attempts per minute per address and per account.
2. Add a lock-out or a growing delay after repeated failures, and log them.

## Examples

### JavaScript

#### Don't

```js
app.post("/login", (req, res) => {
  signIn(req, res);
});
```

#### Do

```js
const loginLimiter = rateLimit({ windowMs: 60000, max: 10 });
app.post("/login", loginLimiter, (req, res) => {
  signIn(req, res);
});
```

### JSX

#### Don't

```jsx
app.post("/login", (req, res) => {
  signIn(req, res);
});
```

#### Do

```jsx
const loginLimiter = rateLimit({ windowMs: 60000, max: 10 });
app.post("/login", loginLimiter, (req, res) => {
  signIn(req, res);
});
```

### TypeScript

#### Don't

```ts
app.post("/login", (req, res) => {
  signIn(req, res);
});
```

#### Do

```ts
const loginLimiter = rateLimit({ windowMs: 60000, max: 10 });
app.post("/login", loginLimiter, (req, res) => {
  signIn(req, res);
});
```

### TSX

#### Don't

```tsx
app.post("/login", (req, res) => {
  signIn(req, res);
});
```

#### Do

```tsx
const loginLimiter = rateLimit({ windowMs: 60000, max: 10 });
app.post("/login", loginLimiter, (req, res) => {
  signIn(req, res);
});
```
