---
id: stencil/ban-side-effects
title: "No calls at a module's top level"
languages: [typescript, tsx]
fix: manual
fix_reason: "where the call should run instead — a function, a lifecycle method — needs the author"
stability: preview
options:
  allow: { type: "string[]", default: ["describe", "test", "bind", "createStore"], description: "functions that may be called at the top level; setting it replaces the default list" }
ported_from: [stencil/ban-side-effects]
since: 0.2.0
---

A module makes no calls at its top level: every call sits inside a function, a method or a class, or
in an `export` declaration. Test files (a name with `spec.`, `e2e.` or `test.` in it) are not
checked, and `describe`, `test`, `bind` and `createStore` may be called anywhere (the `allow` option
changes that list).

**Why:** Stencil's compiler loads modules to analyse them and bundles them lazily, so a call that runs
on import runs at a time no one chose, and may not run at all when a bundle is split.

```codelaw
native: stencil/ban-side-effects
```

## How to fix

1. Move the call into the function, method or lifecycle hook (`componentWillLoad`, `connectedCallback`)
   that needs its effect.
2. If the call builds a value, export a function that builds it, or keep it in an `export const`.
3. If the call must run at load time, add the function's name to the `allow` option.

## Examples

### TypeScript

#### Don't

```ts
import { setupConfig } from './config';

setupConfig({ mode: 'md' });
```

#### Do

```ts
import { setupConfig } from './config';

export function init() {
  setupConfig({ mode: 'md' });
}
```

### TSX

#### Don't

```tsx
import { setupConfig } from './config';

setupConfig({ mode: 'md' });
```

#### Do

```tsx
import { setupConfig } from './config';

export function init() {
  setupConfig({ mode: 'md' });
}
```

## Differences from the source

- Every function is a boundary: the source treats arrow functions and function declarations as
  boundaries but reports calls inside a function expression or an object method, which run only
  when called.
- The option is named `allow`, not the source's unnamed list.
- A tagged template and a dynamic `import()` are not calls here, as in the source.
