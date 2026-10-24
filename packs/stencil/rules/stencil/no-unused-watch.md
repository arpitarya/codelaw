---
id: stencil/no-unused-watch
title: "@Watch watches a @Prop or @State"
languages: [typescript, tsx]
fix: manual
fix_reason: "which member was meant needs the author"
stability: preview
ported_from: [stencil/no-unused-watch]
since: 0.1.0
---

`@Watch('name')` names a `@Prop` or `@State` of the same component.

**Why:** Watching anything else never fires, so the handler is dead code.

```codelaw
native: stencil/no-unused-watch
```

## How to fix

1. Correct the name to the @Prop or @State it should watch, or remove the watcher.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  @State() open = false;
  @Watch('opened') onOpen() {}
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  @State() open = false;
  @Watch('open') onOpen() {}
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  @State() open = false;
  @Watch('opened') onOpen() {}
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  @State() open = false;
  @Watch('open') onOpen() {}
}
```
