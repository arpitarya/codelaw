---
id: stencil/host-data-deprecated
title: "No hostData()"
languages: [typescript, tsx]
fix: manual
fix_reason: "the attributes move into `render()` by hand"
stability: preview
ported_from: [stencil/host-data-deprecated]
since: 0.1.0
---

A component doesn't define `hostData()`.

**Why:** `hostData()` is deprecated; the host element's attributes are set by rendering `<Host>`.

```codelaw
native: stencil/host-data-deprecated
```

## How to fix

1. Render a `<Host>` element in `render()` with the attributes `hostData()` returned.
2. Remove `hostData()`.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'my-card' })
export class Card {
  hostData() {
    return { class: 'card' };
  }
}
```

#### Do

```ts
@Component({ tag: 'my-card' })
export class Card {
  render() {
    return null;
  }
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'my-card' })
export class Card {
  hostData() {
    return { class: 'card' };
  }
}
```

#### Do

```tsx
@Component({ tag: 'my-card' })
export class Card {
  render() {
    return null;
  }
}
```
