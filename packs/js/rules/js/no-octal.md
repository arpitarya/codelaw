---
id: js/no-octal
title: "No legacy octal literals"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether octal or decimal was meant needs the author"
stability: preview
ported_from: [eslint/no-octal]
since: 0.1.0
---

Number literals don't start with `0` followed by a digit.

**Why:** `071` is octal (57) in sloppy code and an error in strict code; `0o71` says what it means.

```codelaw
match:
  kind: number
  regex: ^0[0-9]
message: "`$$MATCH` is a legacy octal literal; write 0o… for octal or drop the leading 0"
```

## How to fix

1. For an octal number, write `0o` and the digits.
2. For a decimal number, drop the leading zero.

## Examples

### JavaScript

#### Don't

```js
const mode = 0755;
```

#### Do

```js
const mode = 0o755;
```

### JSX

#### Don't

```jsx
const mode = 0755;
```

#### Do

```jsx
const mode = 0o755;
```

### TypeScript

#### Don't

```ts
const mode = 0755;
```

#### Do

```ts
const mode = 0o755;
```

### TSX

#### Don't

```tsx
const mode = 0755;
```

#### Do

```tsx
const mode = 0o755;
```
