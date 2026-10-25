---
id: react/no-context
title: "No React context: no createContext, no useContext"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what replaces the context — props, a store, a module — is the team's design choice"
stability: preview
options:
  use_instead: { type: "string", default: "props or the app's state store", description: "what the codebase uses instead of context, named in the message" }
since: 0.3.0
---

No code calls `createContext` or `useContext` from `react`: the named import under any alias, or
`React.createContext` / `React.useContext` on the default or namespace import. A function of your
own that happens to be named `useContext` is not flagged. The rule is a team's choice, so it is not
in `react/recommended`; turn it on in `codelaw.toml`, and name what the team uses instead in the
`use_instead` option. To keep today's context as warnings while new context fails, grandfather it
with a baseline: `codelaw baseline add --rule react/no-context --warn-from <today> --on-touch keep
--reason "legacy context, grandfathered"`.

**Why:** Context hides where a value comes from and re-renders every consumer when it changes; a
team that has settled on props or a store keeps one way of passing data.

```codelaw
native: react/no-context
```

## How to fix

1. Pass the value another way — props, or the app's state store (the `use_instead` option names the team's choice) — and delete the `createContext` / `useContext` call.
2. If the use predates the rule, grandfather it instead: `codelaw baseline add --rule react/no-context --warn-from <today> --on-touch keep --reason "legacy context, grandfathered"` — today's uses then warn and new ones fail.

## Examples

### JavaScript

#### Don't

```js
import { createContext } from 'react';

export const ThemeContext = createContext('light');
```

#### Do

```js
export const defaultTheme = 'light';
```

### JSX

#### Don't

```jsx
import { useContext } from 'react';
import { ThemeContext } from './theme';

export function Button() {
  const theme = useContext(ThemeContext);
  return <button className={theme}>OK</button>;
}
```

#### Do

```jsx
export function Button({ theme }) {
  return <button className={theme}>OK</button>;
}
```

### TypeScript

#### Don't

```ts
import * as React from 'react';

export const UserContext = React.createContext<string | null>(null);
```

#### Do

```ts
export type User = string | null;
```

### TSX

#### Don't

```tsx
import React from 'react';
import { UserContext } from './user';

export function Name() {
  const user = React.useContext(UserContext);
  return <span>{user}</span>;
}
```

#### Do

```tsx
export function Name({ user }: { user: string }) {
  return <span>{user}</span>;
}
```

## Adopting it in a codebase that uses context

Old context warns, new context fails — no engine feature beyond the baseline
(SR-BASELINE decisions 1–2):

```toml
# codelaw.toml
[rules]
"react/no-context" = "error"
```

```bash
codelaw baseline add --rule react/no-context \
  --warn-from 2026-10-25 --on-touch keep --reason "legacy context, grandfathered"
```

| what changes | what `codelaw check` says |
|---|---|
| nothing: each file that used context when the baseline was written | a **warning** per use, with no deadline (exit 0) |
| a **new file** calls `createContext` or `useContext` | an **error** — the file has no baseline entry (exit 1) |
| **one more** use in an old file | **every** use in that file is an error: "2 in this file; the baseline allows 1" (exit 1) |
| an old file edited for other reasons | nothing: `--on-touch keep` leaves its uses as warnings |

Add `--expires <date>` to give the old uses a deadline: from that day they are errors too, with a warning `warn_days` before it.
`fixtures/react-no-context/` is this recipe as a repo, run by `crates/codelaw-cli/tests/react_no_context.rs`.

## What is not flagged

- React 19's `use(SomeContext)`: without types, `use` reading a context can't be told from `use` reading a promise.
- `<Ctx.Provider>` and `<Ctx>` in JSX: the context they provide was made by a `createContext` the rule already reports.
- The count hole: with `--on-touch keep`, removing one `useContext` from an old file and adding another in the same file nets out, and the run passes. The baseline counts uses per file; it does not tell one use from another.
