---
id: sec-js/no-cleartext-http
title: "No request made over http://"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the server must answer on HTTPS before the scheme is switched, and the text of a URL cannot be rewritten by an edit primitive"
stability: preview
ported_from: [bandit/B310, ruff/S310]
security: { cwe: [319], owasp: ["A04:2025"], severity: medium }
since: 0.3.0
---

`fetch`, `axios`, `http.get`, `got` and similar are not given a literal `http://` URL, other than for localhost.

**Why:** A request over plain HTTP can be read and rewritten by anyone on the network path, including the credentials and tokens in it (CWE-319).

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
    - pattern: http.request($U)
    - pattern: http.request($U, $$$REST)
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
    - pattern: $.ajax($U)
    - pattern: $.ajax($U, $$$REST)
    - pattern: $.get($U)
    - pattern: $.get($U, $$$REST)
    - pattern: $.getJSON($U)
    - pattern: $.getJSON($U, $$$REST)
constraints:
  U:
    any:
      - kind: string
      - kind: template_string
    regex: '^["''`]http://'
    not:
      regex: '^["''`]http://(localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1\]|[\w.-]*\.(local|localhost|test|invalid))([:/"''`]|$)'
message: "`$U` is fetched over plain HTTP; anyone on the path can read or change it"
```

## How to fix

1. Change `http://` to `https://` in the URL, and check the server answers on it.
2. Where the host has no certificate, get one (Let's Encrypt) rather than falling back to HTTP.
3. A URL that is only ever local (`localhost`, `127.0.0.1`) may stay; those are not flagged.

## Examples

### JavaScript

#### Don't

```js
const response = await fetch("http://api.example-bank.com/accounts");
```

#### Do

```js
const response = await fetch("https://api.example-bank.com/accounts");
```

### JSX

#### Don't

```jsx
const response = await fetch("http://api.example-bank.com/accounts");
```

#### Do

```jsx
const response = await fetch("https://api.example-bank.com/accounts");
```

### TypeScript

#### Don't

```ts
const response = await fetch("http://api.example-bank.com/accounts");
```

#### Do

```ts
const response = await fetch("https://api.example-bank.com/accounts");
```

### TSX

#### Don't

```tsx
const response = await fetch("http://api.example-bank.com/accounts");
```

#### Do

```tsx
const response = await fetch("https://api.example-bank.com/accounts");
```
