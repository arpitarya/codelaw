---
id: stencil/prefer-vdom-listener
title: "DOM events are handled in render(), not with @Listen"
languages: [typescript, tsx]
fix: manual
fix_reason: "the handler moves into the rendered markup by hand"
stability: preview
ported_from: [stencil/prefer-vdom-listener]
since: 0.1.0
---

A component handles DOM events on itself (`click`, `keydown`, …) with an `on…` handler in
`render()`, not `@Listen`; `@Listen` with a `target` is fine.

**Why:** Handlers in `render()` are scoped to the element they're on and cheaper than a host-wide
listener.

```codelaw
native: stencil/prefer-vdom-listener
```

## How to fix

1. Attach the handler in `render()` (e.g. `onClick={…}` on `<Host>`).
2. Remove the `@Listen`.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  @Listen('click') onClick() {}
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  @Listen('resize', { target: 'window' }) onResize() {}
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  @Listen('click') onClick() {}
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  @Listen('resize', { target: 'window' }) onResize() {}
}
```
