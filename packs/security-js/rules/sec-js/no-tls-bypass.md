---
id: sec-js/no-tls-bypass
title: "No switching off of certificate checks"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "a connection to a host with a self-signed certificate stops working until its CA is trusted"
stability: preview
ported_from: [bandit/B501, ruff/S501]
security: { cwe: [295], owasp: ["A04:2025"], severity: high }
since: 0.3.0
---

`rejectUnauthorized: false`, `strictSSL: false` and `NODE_TLS_REJECT_UNAUTHORIZED = "0"` are not set.

**Why:** With the checks off, anyone on the network path presents any certificate and reads or changes the traffic (CWE-295).

```codelaw
match:
  any:
    - pattern:
        context: '({ rejectUnauthorized: false })'
        selector: pair
    - pattern:
        context: '({ strictSSL: false })'
        selector: pair
    - all:
        - pattern: process.env.NODE_TLS_REJECT_UNAUTHORIZED = $V
        - regex: '=\s*["'']?0["'']?\s*$'
message: "Certificate checks are switched off here; the connection can be read or changed on the way"
fix:
  - remove
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const agent = new https.Agent({ rejectUnauthorized: false, keepAlive: true });
```

#### Do

```js
const agent = new https.Agent({ keepAlive: true });
```

#### Fixed

```js
const agent = new https.Agent({ keepAlive: true });
```

### JSX

#### Don't

```jsx
const agent = new https.Agent({ rejectUnauthorized: false, keepAlive: true });
```

#### Do

```jsx
const agent = new https.Agent({ keepAlive: true });
```

#### Fixed

```jsx
const agent = new https.Agent({ keepAlive: true });
```

### TypeScript

#### Don't

```ts
const agent = new https.Agent({ rejectUnauthorized: false, keepAlive: true });
```

#### Do

```ts
const agent = new https.Agent({ keepAlive: true });
```

#### Fixed

```ts
const agent = new https.Agent({ keepAlive: true });
```

### TSX

#### Don't

```tsx
const agent = new https.Agent({ rejectUnauthorized: false, keepAlive: true });
```

#### Do

```tsx
const agent = new https.Agent({ keepAlive: true });
```

#### Fixed

```tsx
const agent = new https.Agent({ keepAlive: true });
```
