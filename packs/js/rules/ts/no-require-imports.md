---
id: ts/no-require-imports
title: "No require() calls"
languages: [typescript, tsx]
fix: manual
fix_reason: "an import may need the module's default or named exports chosen by hand"
stability: preview
ported_from: [typescript-eslint/no-require-imports]
since: 0.1.0
---

Modules are loaded with `import`, not `require()`.

**Why:** `require()` is untyped and invisible to TypeScript's module checks.

```codelaw
match:
  pattern: require($$$ARGS)
message: "Use `import` instead of `require()`"
```

## How to fix

1. Replace the call with an `import` of the names the code uses.

## Examples

### TypeScript

#### Don't

```ts
const fs = require("node:fs");
```

#### Do

```ts
import fs from "node:fs";
```

### TSX

#### Don't

```tsx
const fs = require("node:fs");
```

#### Do

```tsx
import fs from "node:fs";
```
