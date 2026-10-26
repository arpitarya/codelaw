---
id: sec-js/no-header-injection
title: "No response header value taken from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "encoding or rejecting is a choice that depends on the header"
stability: preview
security: { cwe: [113], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A response header is not set from request data unchanged.

**Why:** A value with a carriage return and a line feed in it ends the header and starts another one, or the body: response splitting (CWE-113).

```codelaw
match:
  any:
    - pattern: res.setHeader($N, $V)
    - pattern: res.header($N, $V)
    - pattern: res.set($N, $V)
    - pattern: res.append($N, $V)
    - pattern: response.setHeader($N, $V)
    - pattern: response.header($N, $V)
    - pattern: reply.header($N, $V)
    - pattern: ctx.set($N, $V)
    - pattern: ctx.append($N, $V)
    - pattern: h.header($N, $V)
where:
  $V: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$V` comes from the request and is written into a response header"
```

## How to fix

1. Encode the value for a header (`encodeURIComponent`) or check it against an allow-list before it is written.
2. Reject a value that contains `\r` or `\n`.

## Examples

### JavaScript

#### Don't

```js
res.setHeader("X-Name", req.query.name);
```

#### Do

```js
res.setHeader("X-Name", encodeURIComponent(req.query.name));
```

### JSX

#### Don't

```jsx
res.setHeader("X-Name", req.query.name);
```

#### Do

```jsx
res.setHeader("X-Name", encodeURIComponent(req.query.name));
```

### TypeScript

#### Don't

```ts
res.setHeader("X-Name", req.query.name);
```

#### Do

```ts
res.setHeader("X-Name", encodeURIComponent(req.query.name));
```

### TSX

#### Don't

```tsx
res.setHeader("X-Name", req.query.name);
```

#### Do

```tsx
res.setHeader("X-Name", encodeURIComponent(req.query.name));
```
