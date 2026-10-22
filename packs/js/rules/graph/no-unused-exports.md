---
id: graph/no-unused-exports
title: Exported symbols must be used somewhere
languages: [javascript, jsx, typescript, tsx]
scope: project
severity: error
fix: suggest
fix_reason: "Static analysis can't rule out reflective or external use, so removal needs a human or agent to accept the diff."
stability: preview
ported_from: [knip/exports, knip/types]
since: 0.1.0
---

Every exported symbol is imported by another module, is reachable from an entry point, or is marked `@public` / `@keep`.

**Why:** an export nothing imports is dead code that still looks like API: readers keep it, refactors carry it, and it hides what a module is really for.

```codelaw
graph:
  symbol:
    exported: true
    kind: [function, class, variable, type, interface, enum]
  unless:
    - reachable_from: entry
    - tagged: [public, keep]
  when:
    external_references: 0
  certainty_from: module.dynamic
message: "`$SYMBOL` is exported but never used outside `$FILE`"
fix:
  - when: { local_references: ">0" }
    do: [strip-export-keyword]
  - else:
    do: [remove-declaration]
```

## How to fix

1. Run `codelaw graph why-unused <symbol>` (`<file>#<export>` or `<module>:<name>`) to see each entry, keep and import clause.
2. If something outside the repo uses it, add the file to `[graph] entry` or `[graph] keep`, add its decorator to `[graph] registrars`, or tag it `/** @public */` / `# codelaw: keep`.
3. Otherwise delete the declaration (and its `__all__` or `export { … }` entry), or drop `export` if its own file still uses it.

## Examples

### JavaScript

#### Don't

```js files
// src/date.js
export function formatDate(d) { return d.toISOString(); }
export function parseDate(s) { return new Date(s); }
// src/app.js
import { formatDate } from "./date";
console.log(formatDate(new Date()));
```

#### Do

```js files
// src/date.js
export function formatDate(d) { return d.toISOString(); }
// src/app.js
import { formatDate } from "./date";
console.log(formatDate(new Date()));
```

### JSX

#### Don't

```jsx files
// src/Button.jsx
export function Button() { return <button />; }
export function Unused() { return <span />; }
// src/App.jsx
import { Button } from "./Button";
console.log(<Button />);
```

#### Do

```jsx files
// src/Button.jsx
export function Button() { return <button />; }
// src/App.jsx
import { Button } from "./Button";
console.log(<Button />);
```

### TypeScript

#### Don't

```ts files
// packages/utils/src/date.ts
export function formatDate(d: Date) { return d.toISOString(); }
export function parseDate(s: string) { return new Date(s); }
// packages/web/src/app.ts
import { formatDate } from "../../utils/src/date";
console.log(formatDate(new Date()));
```

#### Do

```ts files
// packages/utils/src/date.ts
export function formatDate(d: Date) { return d.toISOString(); }
// packages/web/src/app.ts
import { formatDate } from "../../utils/src/date";
console.log(formatDate(new Date()));
```

### TSX

#### Don't

```tsx files
// src/theme.tsx
export const theme = { dark: false };
export const unusedTheme = { dark: true };
// src/main.tsx
import { theme } from "./theme";
console.log(theme);
```

#### Do

```tsx files
// src/theme.tsx
export const theme = { dark: false };
// src/main.tsx
import { theme } from "./theme";
console.log(theme);
```
