---
id: redux-toolkit/prefer-hooks-api
title: "Components read the store with hooks, not connect"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the component's props and its wrapper are rewritten as hook calls"
stability: preview
since: 0.1.0
---

`connect` is not imported from `react-redux`; function components use `useSelector` and
`useDispatch`.

**Why:** The hooks are simpler to read and type than `connect`'s wrapper and its `mapState` and `mapDispatch`
props. *Style Guide, Priority B: Use the React-Redux Hooks API.*

```codelaw
match:
  kind: import_specifier
  has: { field: name, regex: ^connect$ }
  inside:
    kind: import_statement
    has: { field: source, regex: "^['\"]react-redux['\"]$" }
    stopBy: end
message: "Read the store with `useSelector` and `useDispatch` instead of wrapping the component in `connect`"
```

## How to fix

1. In the component, call `useSelector` for each value `mapState` gave it, and `useDispatch` for the
   actions `mapDispatch` gave it.
2. Remove the `connect(…)` wrapper and export the component itself.

## Examples

### JavaScript

#### Don't

```js
import { connect } from 'react-redux';
```

#### Do

```js
import { useDispatch, useSelector } from 'react-redux';
```

### JSX

#### Don't

```jsx
import { connect } from 'react-redux';
```

#### Do

```jsx
import { useDispatch, useSelector } from 'react-redux';
```

### TypeScript

#### Don't

```ts
import { connect } from 'react-redux';
```

#### Do

```ts
import { useDispatch, useSelector } from 'react-redux';
```

### TSX

#### Don't

```tsx
import { connect } from 'react-redux';
```

#### Do

```tsx
import { useDispatch, useSelector } from 'react-redux';
```
