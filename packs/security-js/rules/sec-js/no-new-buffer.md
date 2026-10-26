---
id: sec-js/no-new-buffer
title: "No new Buffer, no noAssert"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "`Buffer.from` rejects a number, so a call that passed a size needs `Buffer.alloc`"
stability: preview
ported_from: [eslint-plugin-security/detect-new-buffer, eslint-plugin-security/detect-buffer-noassert, eslint-plugin-n/no-deprecated-api]
security: { cwe: [119], owasp: ["A06:2025"], severity: medium }
since: 0.3.0
---

Buffers are made with `Buffer.from`, `Buffer.alloc` or `Buffer.allocUnsafe`, never `new Buffer(…)`, and the read and write methods are not given `noAssert`.

**Why:** `new Buffer(n)` returns uninitialised memory that can hold secrets from earlier requests, and `new Buffer(x)` means two things by the type of `x` (CWE-119); `noAssert` skips the bounds check.

```codelaw
match:
  any:
    - all:
        - pattern: new Buffer($N)
        - regex: '^new\s+Buffer\(\s*\d+\s*\)$'
    - all:
        - pattern: new $B($$$ARGS)
        - not:
            regex: '^new\s+Buffer\(\s*\d+\s*\)$'
    - pattern: $RCV.$M($OFF, true)
    - pattern: $RCV.$M($OFF, $VAL, true)
constraints:
  B: { regex: '^Buffer$' }
  M: { regex: '^(read|write)(U?Int(8|16|32)(LE|BE)?|Float(LE|BE)?|Double(LE|BE)?)$' }
message: "`new Buffer` is deprecated and can return uninitialised memory"
fix:
  - replace: "$B.from($$$ARGS)"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const buf = new Buffer(text, "utf8");
```

#### Do

```js
const buf = Buffer.from(text, "utf8");
```

#### Fixed

```js
const buf = Buffer.from(text, "utf8");
```

### JSX

#### Don't

```jsx
const buf = new Buffer(text, "utf8");
```

#### Do

```jsx
const buf = Buffer.from(text, "utf8");
```

#### Fixed

```jsx
const buf = Buffer.from(text, "utf8");
```

### TypeScript

#### Don't

```ts
const buf = new Buffer(text, "utf8");
```

#### Do

```ts
const buf = Buffer.from(text, "utf8");
```

#### Fixed

```ts
const buf = Buffer.from(text, "utf8");
```

### TSX

#### Don't

```tsx
const buf = new Buffer(text, "utf8");
```

#### Do

```tsx
const buf = Buffer.from(text, "utf8");
```

#### Fixed

```tsx
const buf = Buffer.from(text, "utf8");
```
