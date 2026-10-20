---
id: stencil/decorators-context
title: "Stencil decorators only inside @Component classes"
languages: [typescript, tsx]
fix: manual
fix_reason: "whether the class should be a component needs the author"
stability: preview
ported_from: [stencil/decorators-context]
since: 0.1.0
---

`@Prop`, `@State`, `@Element`, `@Method`, `@Watch`, `@Listen`, `@Event` and `@AttachInternals`
appear only in a class decorated with `@Component`.

**Why:** Outside a component Stencil ignores them, so the code silently does nothing.

```codelaw
native: stencil/decorators-context
```

## How to fix

1. Make the class a `@Component`, or remove the Stencil decorators.

## Examples

### TypeScript

#### Don't

```ts
export class Store {
  @State() count = 0;
}
```

#### Do

```ts
export class Store {
  count = 0;
}
```

### TSX

#### Don't

```tsx
export class Store {
  @State() count = 0;
}
```

#### Do

```tsx
export class Store {
  count = 0;
}
```
