---
id: ts/no-explicit-any
title: "No explicit any"
languages: [typescript, tsx]
fix: suggest
fix_reason: "`unknown` may need narrowing code where the value is used"
stability: preview
ported_from: [typescript-eslint/no-explicit-any]
since: 0.1.0
---

Types are not written as `any`.

**Why:** `any` silently disables checking for everything it touches; `unknown` keeps the checks and
forces a narrowing.

```codelaw
match:
  kind: predefined_type
  regex: ^any$
message: "`any` turns off type-checking here; use `unknown` and narrow it"
fix: "unknown"
```

## Examples

### TypeScript

#### Don't

```ts
function parse(input: any) {
  return input;
}
```

#### Do

```ts
function parse(input: unknown) {
  return input;
}
```

### TSX

#### Don't

```tsx
function parse(input: any) {
  return input;
}
```

#### Do

```tsx
function parse(input: unknown) {
  return input;
}
```
