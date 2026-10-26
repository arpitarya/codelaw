---
id: sec-js/no-cors-wildcard-credentials
title: "No CORS that allows every origin together with credentials"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which origins may call the API is a decision for the author"
stability: preview
security: { cwe: [942], owasp: ["A02:2025"], severity: high }
since: 0.3.0
---

`cors()` is not given `origin: "*"` or `origin: true` together with `credentials: true`.

**Why:** `origin: true` reflects whatever origin asks, so any website can send the visitor's cookies to your API and read the answer (CWE-942).

```codelaw
match:
  all:
    - pattern: cors($$$ARGS)
    - has:
        kind: pair
        stopBy: end
        regex: '^["'']?origin["'']?\s*:\s*(["'']\*["'']|true)\s*$'
    - has:
        kind: pair
        stopBy: end
        regex: '^["'']?credentials["'']?\s*:\s*true\s*$'
message: "Every origin is allowed together with credentials; any site can use the visitor's session"
```

## How to fix

1. List the origins that may call the API: `origin: ["https://app.example.com"]`, or a function that checks a list.
2. Drop `credentials: true` if the API does not use cookies; then `origin: "*"` is acceptable for public data.

## Examples

### JavaScript

#### Don't

```js
app.use(cors({ origin: true, credentials: true }));
```

#### Do

```js
app.use(cors({ origin: ["https://app.example.com"], credentials: true }));
```

### JSX

#### Don't

```jsx
app.use(cors({ origin: true, credentials: true }));
```

#### Do

```jsx
app.use(cors({ origin: ["https://app.example.com"], credentials: true }));
```

### TypeScript

#### Don't

```ts
app.use(cors({ origin: true, credentials: true }));
```

#### Do

```ts
app.use(cors({ origin: ["https://app.example.com"], credentials: true }));
```

### TSX

#### Don't

```tsx
app.use(cors({ origin: true, credentials: true }));
```

#### Do

```tsx
app.use(cors({ origin: ["https://app.example.com"], credentials: true }));
```
