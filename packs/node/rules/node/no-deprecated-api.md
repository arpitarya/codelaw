---
id: node/no-deprecated-api
title: "No deprecated Node.js APIs"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "each deprecated API has its own replacement"
stability: preview
ported_from: [eslint-plugin-n/no-deprecated-api]
since: 0.1.0
---

Code doesn't use Node.js APIs the documentation deprecates — `new Buffer()`, `fs.exists`,
`url.parse`, `util.isArray`, the `domain` module and the rest — whether reached by `import`,
`require()` or a global.

**Why:** Deprecated APIs are unsafe, slow or due for removal, and each has a documented replacement.

```codelaw
native: node/no-deprecated-api
```

## How to fix

1. Replace the call with the API the message names.

## Examples

### JavaScript

#### Don't

```js
import { isArray } from "node:util";
export const ok = isArray([]);
```

#### Do

```js
export const ok = Array.isArray([]);
```

### JSX

#### Don't

```jsx
import { isArray } from "node:util";
export const ok = isArray([]);
```

#### Do

```jsx
export const ok = Array.isArray([]);
```

### TypeScript

#### Don't

```ts
import { isArray } from "node:util";
export const ok = isArray([]);
```

#### Do

```ts
export const ok = Array.isArray([]);
```

### TSX

#### Don't

```tsx
import { isArray } from "node:util";
export const ok = isArray([]);
```

#### Do

```tsx
export const ok = Array.isArray([]);
```

## Differences from the source

The pack's `node/deprecated-api` data table lists the APIs; the Node.js version a project supports
is not read.
