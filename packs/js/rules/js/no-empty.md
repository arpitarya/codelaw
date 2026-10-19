---
id: js/no-empty
title: "No empty blocks"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether to fill the block, remove it or document it needs the author"
stability: preview
ported_from: [eslint/no-empty]
since: 0.1.0
---

A block or `switch` has at least one statement, or a comment saying why it is empty. Function bodies
may be empty.

**Why:** An empty block is usually unfinished code, and an empty `catch` hides errors.

```codelaw
match:
  any:
    - kind: statement_block
      regex: ^\{\s*\}$
      not:
        inside:
          stopBy: neighbor
          any: [{kind: function_declaration}, {kind: function_expression}, {kind: arrow_function}, {kind: method_definition}, {kind: generator_function_declaration}, {kind: generator_function}, { kind: class_static_block }]
    - kind: switch_statement
      has: { field: body, regex: '^\{\s*\}$' }
message: "Empty block; remove it or say why it's empty"
```

## How to fix

1. If the block is unfinished, write its code.
2. If it's meant to be empty, put a comment in it saying why.

## Examples

### JavaScript

#### Don't

```js
try {
  load();
} catch (e) {}
```

#### Do

```js
try {
  load();
} catch (e) {
  // The cache is optional: a failed load is not an error.
}
```

### JSX

#### Don't

```jsx
try {
  load();
} catch (e) {}
```

#### Do

```jsx
try {
  load();
} catch (e) {
  // The cache is optional: a failed load is not an error.
}
```

### TypeScript

#### Don't

```ts
try {
  load();
} catch (e) {}
```

#### Do

```ts
try {
  load();
} catch (e) {
  // The cache is optional: a failed load is not an error.
}
```

### TSX

#### Don't

```tsx
try {
  load();
} catch (e) {}
```

#### Do

```tsx
try {
  load();
} catch (e) {
  // The cache is optional: a failed load is not an error.
}
```
