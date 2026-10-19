---
id: ts/prefer-as-const
title: "as const instead of repeating a literal as its type"
languages: [typescript, tsx]
fix: auto
stability: preview
ported_from: [typescript-eslint/prefer-as-const]
since: 0.1.0
---

A literal cast to its own literal type (`"a" as "a"`) uses `as const` instead.

**Why:** `as const` says the same thing once, and stays right when the literal changes.

```codelaw
native: ts/prefer-as-const
```

## Examples

### TypeScript

#### Don't

```ts
const mode = "dark" as "dark";
```

#### Do

```ts
const mode = "dark" as const;
```

### TSX

#### Don't

```tsx
const mode = "dark" as "dark";
```

#### Do

```tsx
const mode = "dark" as const;
```
