---
id: redux-toolkit/slice-key-names
title: "State slices are named after their data"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the key is the state's shape, so every selector that reads it changes too"
stability: preview
since: 0.1.0
---

The keys of `combineReducers`, `combineSlices` and `configureStore`'s `reducer` object name the data
each slice holds (`users`), not the reducer that manages it (`usersReducer`).

**Why:** The key is the field every selector reads: `state.usersReducer.list` reads as code, not data.
*Style Guide, Priority B: Name State Slices Based On the Stored Data.*

```codelaw
native: redux-toolkit/slice-key-names
```

## How to fix

1. Rename the key after the data: `users: usersReducer`.
2. Update every selector and test that reads the old key.

## Examples

### JavaScript

#### Don't

```js
const rootReducer = combineReducers({ usersReducer, postsReducer });
```

#### Do

```js
const rootReducer = combineReducers({ users: usersReducer, posts: postsReducer });
```

### JSX

#### Don't

```jsx
const rootReducer = combineReducers({ usersReducer, postsReducer });
```

#### Do

```jsx
const rootReducer = combineReducers({ users: usersReducer, posts: postsReducer });
```

### TypeScript

#### Don't

```ts
const rootReducer = combineReducers({ usersReducer, postsReducer });
```

#### Do

```ts
const rootReducer = combineReducers({ users: usersReducer, posts: postsReducer });
```

### TSX

#### Don't

```tsx
const rootReducer = combineReducers({ usersReducer, postsReducer });
```

#### Do

```tsx
const rootReducer = combineReducers({ users: usersReducer, posts: postsReducer });
```
