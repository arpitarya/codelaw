---
id: js/no-empty-pattern
title: "No empty destructuring patterns"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the pattern was meant to bind needs the author"
stability: preview
ported_from: [eslint/no-empty-pattern]
since: 0.1.0
---

A destructuring pattern names at least one binding.

**Why:** `const {} = obj` binds nothing; it's usually a mistyped default such as `{ a = {} }`.

```codelaw
match:
  any:
    - kind: object_pattern
      regex: ^\{\s*\}$
    - kind: array_pattern
      regex: ^\[\s*\]$
message: "This destructuring pattern binds nothing"
```

## How to fix

1. Name the properties to bind inside the pattern, or remove it.

## Examples

### JavaScript

#### Don't

```js
const { a: {} } = options;
```

#### Do

```js
const { a = {} } = options;
```

### JSX

#### Don't

```jsx
const { a: {} } = options;
```

#### Do

```jsx
const { a = {} } = options;
```

### TypeScript

#### Don't

```ts
const { a: {} } = options;
```

#### Do

```ts
const { a = {} } = options;
```

### TSX

#### Don't

```tsx
const { a: {} } = options;
```

#### Do

```tsx
const { a = {} } = options;
```
