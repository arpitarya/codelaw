---
id: sec-js/cookie-flags
title: "A cookie sets httpOnly, secure and sameSite"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a cookie a script must read, or one used cross-site, needs different flags"
stability: preview
security: { cwe: [614, 1004], owasp: ["A02:2025"], severity: medium }
since: 0.3.0
---

`res.cookie`, `reply.setCookie` and `ctx.cookies.set` pass an options object with `httpOnly`, `secure` and `sameSite`.

**Why:** Without `httpOnly` a script reads the cookie, without `secure` it travels over plain HTTP, and without `sameSite` the browser sends it on cross-site requests (CWE-614, CWE-1004, CWE-352).

```codelaw
match:
  any:
    - pattern: $C($N, $V)
    - pattern: $C2($N2, $V2, $OPTS)
constraints:
  C: { regex: '^(res|response)\.cookie$|^reply\.setCookie$|^ctx\.cookies\.set$' }
  C2: { regex: '^(res|response)\.cookie$|^reply\.setCookie$|^ctx\.cookies\.set$' }
  OPTS:
    any:
      - not: { regex: 'httpOnly' }
      - not: { regex: 'secure' }
      - not: { regex: 'sameSite' }
message: "This cookie is missing `httpOnly`, `secure` or `sameSite`"
fix:
  - replace: "$C($N, $V, { httpOnly: true, secure: true, sameSite: \"lax\" })"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
res.cookie("sid", sid);
```

#### Do

```js
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

#### Fixed

```js
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

### JSX

#### Don't

```jsx
res.cookie("sid", sid);
```

#### Do

```jsx
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

#### Fixed

```jsx
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

### TypeScript

#### Don't

```ts
res.cookie("sid", sid);
```

#### Do

```ts
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

#### Fixed

```ts
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

### TSX

#### Don't

```tsx
res.cookie("sid", sid);
```

#### Do

```tsx
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```

#### Fixed

```tsx
res.cookie("sid", sid, { httpOnly: true, secure: true, sameSite: "lax" });
```
