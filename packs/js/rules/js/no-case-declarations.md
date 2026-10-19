---
id: js/no-case-declarations
title: "No let, const, function or class directly in a case"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "another case may rely on the shared declaration"
stability: preview
ported_from: [eslint/no-case-declarations]
since: 0.1.0
---

A `let`, `const`, `function` or `class` inside a `case` sits in a block `{ … }` of its own.

**Why:** Declarations in a case are scoped to the whole `switch`, so another case can see them
uninitialised.

```codelaw
match:
  any:
    - kind: lexical_declaration
    - kind: class_declaration
    - kind: function_declaration
  inside:
    stopBy: neighbor
    any: [{ kind: switch_case }, { kind: switch_default }]
message: "A declaration in a case is visible to the whole switch; wrap the case in braces"
```

## How to fix

1. Wrap the case's statements in `{ … }`.
2. If another case uses the declaration, move it above the `switch`.

## Examples

### JavaScript

#### Don't

```js
switch (kind) {
  case "a":
    const n = 1;
    use(n);
    break;
}
```

#### Do

```js
switch (kind) {
  case "a": {
    const n = 1;
    use(n);
    break;
  }
}
```

### JSX

#### Don't

```jsx
switch (kind) {
  case "a":
    const n = 1;
    use(n);
    break;
}
```

#### Do

```jsx
switch (kind) {
  case "a": {
    const n = 1;
    use(n);
    break;
  }
}
```

### TypeScript

#### Don't

```ts
switch (kind) {
  case "a":
    const n = 1;
    use(n);
    break;
}
```

#### Do

```ts
switch (kind) {
  case "a": {
    const n = 1;
    use(n);
    break;
  }
}
```

### TSX

#### Don't

```tsx
switch (kind) {
  case "a":
    const n = 1;
    use(n);
    break;
}
```

#### Do

```tsx
switch (kind) {
  case "a": {
    const n = 1;
    use(n);
    break;
  }
}
```
