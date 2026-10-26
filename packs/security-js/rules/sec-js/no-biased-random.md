---
id: sec-js/no-biased-random
title: "No byte taken modulo a number that does not divide 256"
languages: [javascript, jsx, typescript, tsx]
fix: suggest
fix_reason: "`crypto.randomInt` needs Node 14.10 or later, and a range that starts above zero needs the bounds written"
stability: preview
security: { cwe: [327], owasp: ["A04:2025"], severity: medium }
since: 0.3.0
---

A random byte or integer from `crypto.randomBytes` is not reduced with `%`; `crypto.randomInt(max)` does it without bias.

**Why:** `byte % 6` makes some results more likely than others (256 is not a multiple of 6), which weakens a token, a die or a shuffle (CWE-327).

```codelaw
match:
  any:
    - pattern: crypto.randomBytes($N)[$I] % $K
    - pattern: randomBytes($N)[$I] % $K
    - pattern: crypto.randomBytes($N).readUInt8($I) % $K
    - pattern: crypto.randomBytes($N).readUInt16BE($I) % $K
    - pattern: crypto.randomBytes($N).readUInt32BE($I) % $K
    - pattern: crypto.randomBytes($N).readUInt32LE($I) % $K
constraints:
  K: { not: { regex: '^(2|4|8|16|32|64|128|256|512|1024)$' } }
message: "A random value reduced with `%` is biased"
fix:
  - replace: "crypto.randomInt($K)"
```

## How to fix


## Examples

### JavaScript

#### Don't

```js
const roll = crypto.randomBytes(1)[0] % 6;
```

#### Do

```js
const roll = crypto.randomInt(6);
```

#### Fixed

```js
const roll = crypto.randomInt(6);
```

### JSX

#### Don't

```jsx
const roll = crypto.randomBytes(1)[0] % 6;
```

#### Do

```jsx
const roll = crypto.randomInt(6);
```

#### Fixed

```jsx
const roll = crypto.randomInt(6);
```

### TypeScript

#### Don't

```ts
const roll = crypto.randomBytes(1)[0] % 6;
```

#### Do

```ts
const roll = crypto.randomInt(6);
```

#### Fixed

```ts
const roll = crypto.randomInt(6);
```

### TSX

#### Don't

```tsx
const roll = crypto.randomBytes(1)[0] % 6;
```

#### Do

```tsx
const roll = crypto.randomInt(6);
```

#### Fixed

```tsx
const roll = crypto.randomInt(6);
```
