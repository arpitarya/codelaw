---
id: ts/await-thenable
title: "`await` only what can be awaited"
languages: [typescript, tsx]
fix: suggest
fix_reason: "removing `await` also removes the pause it makes, which code after it may rely on"
stability: preview
ported_from: [typescript-eslint/await-thenable]
tags: [types, async]
since: 0.3.0
---

`await` is applied only to a Promise, another thenable, or a value typed `any`.

**Why:** Awaiting anything else does nothing but delay the code after it; it usually means a call
that was thought to be asynchronous is not, or a Promise was unwrapped twice.

```codelaw
match:
  pattern: await $X
type:
  $X: { not: { any: [thenable, any] } }
message: "`$X` is not a Promise or a thenable, so awaiting it does nothing"
fix: "$X"
```

## Examples

### TypeScript

#### Don't

```ts
async function main() {
  const n = await 42;
}
```

#### Do

```ts
async function main() {
  const n = await Promise.resolve(42);
}
```

#### Fixed

```ts
async function main() {
  const n = 42;
}
```

### TSX

#### Don't

```tsx
async function main() {
  const n = await 42;
}
```

#### Do

```tsx
async function main() {
  const n = await Promise.resolve(42);
}
```

#### Fixed

```tsx
async function main() {
  const n = 42;
}
```

## Differences from the source

A value typed `unknown` is reported: typescript-eslint allows it, but SR-TYPES's type questions do
not yet tell `unknown` apart from any other type that is not a thenable.
