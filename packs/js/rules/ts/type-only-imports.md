---
id: ts/type-only-imports
title: "Names used only as types are imported with type"
languages: [typescript, tsx]
fix: suggest
fix_reason: "a type-only import is erased, so code that relied on the module's side effects changes"
stability: preview
since: 0.1.0
---

An imported name that the file uses only in type positions is marked `type`.

**Why:** Type-only imports are erased at compile time, which keeps runtime imports honest and helps
bundlers drop unused modules.

```codelaw
native: ts/type-only-imports
```

## Examples

### TypeScript

#### Don't

```ts
import { Config, load } from "./config";
export const c: Config = load();
```

#### Do

```ts
import { type Config, load } from "./config";
export const c: Config = load();
```

### TSX

#### Don't

```tsx
import { Config, load } from "./config";
export const c: Config = load();
```

#### Do

```tsx
import { type Config, load } from "./config";
export const c: Config = load();
```

## Differences from the source

Only within the file, without type information: a name used as a value anywhere is left alone.
Default and namespace imports are reported without a fix.
