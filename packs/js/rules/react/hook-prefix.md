---
id: react/hook-prefix
title: "Functions that call hooks are named use…"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "every caller of the function must be renamed too"
stability: preview
since: 0.1.0
---

A function that isn't a component (PascalCase) and calls a hook is a custom hook, and is named
`use…`.

**Why:** React and its tools recognise hooks by the `use` prefix; without it the Rules of Hooks
can't be checked for the function's callers.

```codelaw
match:
  kind: function_declaration
  all:
    - has: { field: name, regex: '^[a-z]', not: { regex: '^use[A-Z]' }, pattern: $NAME }
    - has:
        field: body
        has:
          stopBy: { any: [{ kind: function_declaration }, { kind: arrow_function }, { kind: function_expression }] }
          kind: call_expression
          has: { field: function, regex: '^(React\.)?use[A-Z]' }
message: "`$NAME` calls hooks, so it is a hook: name it `use…`"
```

## How to fix

1. Rename the function to start with `use` (e.g. `loadUser` → `useUser`).
2. Rename its callers.

## Examples

### JavaScript

#### Don't

```js
function currentUser() {
  return useContext(UserContext);
}
```

#### Do

```js
function useCurrentUser() {
  return useContext(UserContext);
}
```

### JSX

#### Don't

```jsx
function currentUser() {
  return useContext(UserContext);
}
```

#### Do

```jsx
function useCurrentUser() {
  return useContext(UserContext);
}
```

### TypeScript

#### Don't

```ts
function currentUser() {
  return useContext(UserContext);
}
```

#### Do

```ts
function useCurrentUser() {
  return useContext(UserContext);
}
```

### TSX

#### Don't

```tsx
function currentUser() {
  return useContext(UserContext);
}
```

#### Do

```tsx
function useCurrentUser() {
  return useContext(UserContext);
}
```
