---
id: sec-js/no-open-redirect
title: "No redirect to an address taken from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which targets are allowed is a decision for the author"
stability: preview
security: { cwe: [601], owasp: ["A01:2025"], severity: medium }
since: 0.3.0
---

`res.redirect` and its equivalents do not take their target from request data.

**Why:** A link that carries `?next=https://evil.example` sends the visitor, who trusts your site, to a page of the attacker's (CWE-601).

```codelaw
match:
  any:
    - pattern: res.redirect($U)
    - pattern: res.redirect($S, $U)
    - pattern: response.redirect($U)
    - pattern: response.redirect($S, $U)
    - pattern: reply.redirect($U)
    - pattern: reply.redirect($S, $U)
    - pattern: ctx.redirect($U)
    - pattern: ctx.redirect($S, $U)
    - pattern: res.location($U)
    - pattern: res.location($S, $U)
    - pattern: NextResponse.redirect($U)
    - pattern: NextResponse.redirect($S, $U)
    - pattern: Response.redirect($U)
    - pattern: Response.redirect($S, $U)
    - pattern: redirect($U)
    - pattern: redirect($S, $U)
where:
  $U: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$U` comes from the request and chooses where the visitor is sent"
```

## How to fix

1. Parse the target with `new URL(value, ownOrigin)` and redirect only when `origin` is your own, sending `pathname` (and `search`) alone.
2. Or accept a key (`?next=home`) and look the path up in a table of allowed paths.
3. Reject a value that starts with `//` or `\\` or contains a scheme you do not expect.

## Examples

### JavaScript

#### Don't

```js
res.redirect(req.query.next);
```

#### Do

```js
const next = new URL(req.query.next, "https://app.example.com");
res.redirect(next.origin === "https://app.example.com" ? next.pathname : "/");
```

### JSX

#### Don't

```jsx
res.redirect(req.query.next);
```

#### Do

```jsx
const next = new URL(req.query.next, "https://app.example.com");
res.redirect(next.origin === "https://app.example.com" ? next.pathname : "/");
```

### TypeScript

#### Don't

```ts
res.redirect(req.query.next);
```

#### Do

```ts
const next = new URL(req.query.next, "https://app.example.com");
res.redirect(next.origin === "https://app.example.com" ? next.pathname : "/");
```

### TSX

#### Don't

```tsx
res.redirect(req.query.next);
```

#### Do

```tsx
const next = new URL(req.query.next, "https://app.example.com");
res.redirect(next.origin === "https://app.example.com" ? next.pathname : "/");
```
