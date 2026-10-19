---
id: ts/no-unused-vars
title: "Every declared name is used"
languages: [typescript, tsx]
fix: manual
fix_reason: "removing a declaration can drop the side effects of its initialiser"
stability: preview
options:
  vars: { type: string, default: all }
  args: { type: string, default: after-used }
  caughtErrors: { type: string, default: all }
  ignoreRestSiblings: { type: boolean, default: false }
  varsIgnorePattern: { type: string }
  argsIgnorePattern: { type: string }
  caughtErrorsIgnorePattern: { type: string }
  destructuredArrayIgnorePattern: { type: string }
ported_from: [typescript-eslint/no-unused-vars, eslint/no-unused-vars]
since: 0.1.0
---

Every variable, function, class, type, interface, enum, import, parameter and caught error is used
somewhere. Parameters before the last used one are allowed (`args: after-used`).

**Why:** An unused declaration is dead code or a sign the code doesn't do what it was meant to.

```codelaw
native: ts/no-unused-vars
```

## How to fix

1. Check the initialiser has no side effect that matters.
2. Remove the declaration, or use it where it was meant to be used.
3. For a parameter that must stay for its position, prefix it with `_` and set `argsIgnorePattern: "^_"`.

## Examples

### TypeScript

#### Don't

```ts
import { readFile, writeFile } from "node:fs";
export const load = (path: string) => readFile(path);
```

#### Do

```ts
import { readFile } from "node:fs";
export const load = (path: string) => readFile(path);
```

### TSX

#### Don't

```tsx
import { readFile, writeFile } from "node:fs";
export const load = (path: string) => readFile(path);
```

#### Do

```tsx
import { readFile } from "node:fs";
export const load = (path: string) => readFile(path);
```
