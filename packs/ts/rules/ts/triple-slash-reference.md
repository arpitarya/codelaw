---
id: ts/triple-slash-reference
title: "No /// <reference path> directives"
languages: [typescript, tsx]
fix: manual
fix_reason: "the matching import needs the module path and what it provides"
stability: preview
ported_from: [typescript-eslint/triple-slash-reference]
since: 0.1.0
---

Files are connected with `import`, not `/// <reference path="…" />`.

**Why:** Reference directives predate ES modules and hide dependencies from bundlers and other
tools.

```codelaw
match:
  kind: comment
  regex: ^///\s*<reference\s+path=
message: "Use an `import` instead of a `/// <reference path>` directive"
```

## How to fix

1. Replace the directive with an `import` of the module it pointed to.

## Examples

### TypeScript

#### Don't

```ts
/// <reference path="./globals.d.ts" />
export const x = 1;
```

#### Do

```ts
import "./globals";
export const x = 1;
```

### TSX

#### Don't

```tsx
/// <reference path="./globals.d.ts" />
export const x = 1;
```

#### Do

```tsx
import "./globals";
export const x = 1;
```
