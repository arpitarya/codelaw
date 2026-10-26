---
id: sec-js/no-format-string
title: "No request data as a format string"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "it changes what is printed when the value has `%` in it"
stability: preview
security: { cwe: [134], owasp: ["A05:2025"], severity: low }
since: 0.3.0
---

`util.format` and the `console` methods are not given request data as their only argument, where it is read as a format string.

**Why:** A value with `%s`, `%d` or `%o` in it is a format string, so it can print values the author did not mean to log, and a formatter given too few arguments drops text (CWE-134).

```codelaw
match:
  any:
    - pattern: util.format($V)
    - pattern: console.log($V)
    - pattern: console.info($V)
    - pattern: console.warn($V)
    - pattern: console.error($V)
    - pattern: console.debug($V)
where:
  $V: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$V` comes from the request and is read as a format string"
fix:
  - replace: { target: $V, with: '"%s", $V' }
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
console.log(req.query.msg);
```

#### Do

```js
console.log("%s", req.query.msg);
```

#### Fixed

```js
console.log("%s", req.query.msg);
```

### JSX

#### Don't

```jsx
console.log(req.query.msg);
```

#### Do

```jsx
console.log("%s", req.query.msg);
```

#### Fixed

```jsx
console.log("%s", req.query.msg);
```

### TypeScript

#### Don't

```ts
console.log(req.query.msg);
```

#### Do

```ts
console.log("%s", req.query.msg);
```

#### Fixed

```ts
console.log("%s", req.query.msg);
```

### TSX

#### Don't

```tsx
console.log(req.query.msg);
```

#### Do

```tsx
console.log("%s", req.query.msg);
```

#### Fixed

```tsx
console.log("%s", req.query.msg);
```
