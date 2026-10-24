---
id: node/no-process-exit
title: "No process.exit()"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "how to end the program cleanly depends on what is still running"
stability: preview
ported_from: [eslint-plugin-n/no-process-exit]
since: 0.1.0
---

Code doesn't call `process.exit()`.

**Why:** `process.exit()` ends the process at once, dropping pending writes and skipping cleanup;
libraries that call it take that choice away from the program.

```codelaw
match:
  pattern: process.exit($$$ARGS)
message: "Don't end the process from here; throw, or set `process.exitCode`"
```

## How to fix

1. Set `process.exitCode` and let the program finish, or throw an error the caller handles.

## Examples

### JavaScript

#### Don't

```js
if (failed) {
  process.exit(1);
}
```

#### Do

```js
if (failed) {
  process.exitCode = 1;
}
```

### JSX

#### Don't

```jsx
if (failed) {
  process.exit(1);
}
```

#### Do

```jsx
if (failed) {
  process.exitCode = 1;
}
```

### TypeScript

#### Don't

```ts
if (failed) {
  process.exit(1);
}
```

#### Do

```ts
if (failed) {
  process.exitCode = 1;
}
```

### TSX

#### Don't

```tsx
if (failed) {
  process.exit(1);
}
```

#### Do

```tsx
if (failed) {
  process.exitCode = 1;
}
```
