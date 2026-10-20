---
id: js/exports-at-end
title: "Exports are gathered at the end of the file"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
invariant: exports-unchanged
since: 0.1.0
---

Declarations are written without `export`, and the module's exports are listed in one `export { … }`
(and `export type { … }`) block at the end of the file.

**Why:** A reader or an agent sees the module's whole public API in one place.

```codelaw
match:
  kind: export_statement
  has: { field: declaration, pattern: $DECL }
  not:
    any:
      - has: { regex: ^default$ }
      - has: { field: declaration, kind: internal_module }
      - has: { field: declaration, kind: ambient_declaration }
      - has: { field: declaration, kind: module }
message: "Export at the end of the file, in one `export { … }` block"
fix:
  - strip-export-keyword: $DECL
  - collect-into-block:
      target: $DECL
      block: "export { $$$NAMES };"
      type_block: "export type { $$$TYPE_NAMES };"
      position: end-of-file
```

## Examples

### JavaScript

#### Don't

```js
export function add(a, b) {
  return a + b;
}
export const zero = 0;
```

#### Do

```js
function add(a, b) {
  return a + b;
}
const zero = 0;
export { add, zero };
```

### JSX

#### Don't

```jsx
export function add(a, b) {
  return a + b;
}
export const zero = 0;
```

#### Do

```jsx
function add(a, b) {
  return a + b;
}
const zero = 0;
export { add, zero };
```

### TypeScript

#### Don't

```ts
export function add(a: number, b: number) {
  return a + b;
}
export interface Point {
  x: number;
}
```

#### Do

```ts
function add(a: number, b: number) {
  return a + b;
}
interface Point {
  x: number;
}
export { add };
export type { Point };
```

### TSX

#### Don't

```tsx
export function add(a: number, b: number) {
  return a + b;
}
export interface Point {
  x: number;
}
```

#### Do

```tsx
function add(a: number, b: number) {
  return a + b;
}
interface Point {
  x: number;
}
export { add };
export type { Point };
```

## Differences from the source

`export default`, namespaces and `declare` blocks are left alone.
