---
id: stencil/ban-prefix
title: "A component tag doesn't use a reserved prefix"
languages: [typescript, tsx]
fix: manual
fix_reason: "renaming a tag changes every page that uses it"
stability: preview
options:
  prefixes: { type: "string[]", default: ["stencil", "stnl", "st"] }
ported_from: [stencil/ban-prefix]
since: 0.1.0
---

A component's `tag` doesn't start with a prefix Stencil reserves (`stencil-`, `stnl-`, `st-` by
default; the `prefixes` option changes the list).

**Why:** Reserved prefixes may clash with Stencil's own elements.

```codelaw
native: stencil/ban-prefix
```

## How to fix

1. Choose a prefix of your own and rename the tag.
2. Update every usage of the element.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'stencil-card' })
export class Card {
  render() {
    return null;
  }
}
```

#### Do

```ts
@Component({ tag: 'acme-card' })
export class Card {
  render() {
    return null;
  }
}
```

### TSX

#### Don't

```tsx
@Component({ tag: 'stencil-card' })
export class Card {
  render() {
    return null;
  }
}
```

#### Do

```tsx
@Component({ tag: 'acme-card' })
export class Card {
  render() {
    return null;
  }
}
```
