---
id: sec-js/no-ssrf
title: "No request made to an address taken from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which hosts the server may call is a decision for the author"
stability: preview
ported_from: [bandit/B310, ruff/S310]
security: { cwe: [918], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

`fetch`, `axios`, `http.get` and similar are not given a URL taken from request data.

**Why:** The server makes the request from inside your network, so a visitor who picks the URL reaches internal services, cloud metadata (`169.254.169.254`) and local files (CWE-918).

```codelaw
match:
  any:
    - pattern: fetch($U)
    - pattern: fetch($U, $$$REST)
    - pattern: axios($U)
    - pattern: axios($U, $$$REST)
    - pattern: axios.get($U)
    - pattern: axios.get($U, $$$REST)
    - pattern: axios.post($U)
    - pattern: axios.post($U, $$$REST)
    - pattern: axios.put($U)
    - pattern: axios.put($U, $$$REST)
    - pattern: axios.patch($U)
    - pattern: axios.patch($U, $$$REST)
    - pattern: axios.delete($U)
    - pattern: axios.delete($U, $$$REST)
    - pattern: axios.head($U)
    - pattern: axios.head($U, $$$REST)
    - pattern: axios.request($U)
    - pattern: axios.request($U, $$$REST)
    - pattern: got($U)
    - pattern: got($U, $$$REST)
    - pattern: got.get($U)
    - pattern: got.get($U, $$$REST)
    - pattern: got.post($U)
    - pattern: got.post($U, $$$REST)
    - pattern: http.get($U)
    - pattern: http.get($U, $$$REST)
    - pattern: https.get($U)
    - pattern: https.get($U, $$$REST)
    - pattern: http.request($U)
    - pattern: http.request($U, $$$REST)
    - pattern: https.request($U)
    - pattern: https.request($U, $$$REST)
    - pattern: request($U)
    - pattern: request($U, $$$REST)
    - pattern: request.get($U)
    - pattern: request.get($U, $$$REST)
    - pattern: request.post($U)
    - pattern: request.post($U, $$$REST)
    - pattern: needle.get($U)
    - pattern: needle.get($U, $$$REST)
    - pattern: needle.post($U)
    - pattern: needle.post($U, $$$REST)
    - pattern: superagent.get($U)
    - pattern: superagent.get($U, $$$REST)
    - pattern: superagent.post($U)
    - pattern: superagent.post($U, $$$REST)
    - pattern: ky($U)
    - pattern: ky($U, $$$REST)
    - pattern: ky.get($U)
    - pattern: ky.get($U, $$$REST)
    - pattern: ky.post($U)
    - pattern: ky.post($U, $$$REST)
    - pattern: undici.fetch($U)
    - pattern: undici.fetch($U, $$$REST)
    - pattern: undici.request($U)
    - pattern: undici.request($U, $$$REST)
    - pattern: $.ajax($U)
    - pattern: $.ajax($U, $$$REST)
    - pattern: $.get($U)
    - pattern: $.get($U, $$$REST)
    - pattern: $.getJSON($U)
    - pattern: $.getJSON($U, $$$REST)
where:
  $U: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$U` comes from the request and chooses which address the server calls"
```

## How to fix

1. Parse the URL and compare `protocol` and `hostname` with an allow-list of the hosts the feature needs.
2. Resolve the host and refuse private, loopback and link-local addresses (or call through a proxy that does), and do not follow redirects blindly (`redirect: "error"`).
3. Better still, take an id from the request and look the URL up on the server.

## Examples

### JavaScript

#### Don't

```js
const response = await fetch(req.query.url);
```

#### Do

```js
const target = new URL(req.query.url);
if (target.hostname !== "api.example.com") {
  throw new Error("host not allowed");
}
const response = await fetch(target);
```

### JSX

#### Don't

```jsx
const response = await fetch(req.query.url);
```

#### Do

```jsx
const target = new URL(req.query.url);
if (target.hostname !== "api.example.com") {
  throw new Error("host not allowed");
}
const response = await fetch(target);
```

### TypeScript

#### Don't

```ts
const response = await fetch(req.query.url);
```

#### Do

```ts
const target = new URL(req.query.url);
if (target.hostname !== "api.example.com") {
  throw new Error("host not allowed");
}
const response = await fetch(target);
```

### TSX

#### Don't

```tsx
const response = await fetch(req.query.url);
```

#### Do

```tsx
const target = new URL(req.query.url);
if (target.hostname !== "api.example.com") {
  throw new Error("host not allowed");
}
const response = await fetch(target);
```
