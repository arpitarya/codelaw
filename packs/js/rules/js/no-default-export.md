---
id: js/no-default-export
title: "No default exports"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "every importer of the default export must change"
stability: preview
since: 0.1.0
---

Modules export by name; `export default` is not used.

**Why:** A named export has one name everywhere, so it can be found, renamed and auto-imported
reliably.

```codelaw
match:
  kind: export_statement
  has: { regex: ^default$ }
message: "Export this by name instead of as the default"
```

## How to fix

1. Give the exported value a name and export it with `export { name }`.
2. Change every `import x from` of this module to `import { name } from`.

## Examples

### JavaScript

#### Don't

```js
export default function load() {}
```

#### Do

```js
function load() {}
export { load };
```

### JSX

#### Don't

```jsx
export default function load() {}
```

#### Do

```jsx
function load() {}
export { load };
```

### TypeScript

#### Don't

```ts
export default function load() {}
```

#### Do

```ts
function load() {}
export { load };
```

### TSX

#### Don't

```tsx
export default function load() {}
```

#### Do

```tsx
function load() {}
export { load };
```
