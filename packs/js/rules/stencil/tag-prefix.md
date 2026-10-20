---
id: stencil/tag-prefix
title: "Component tags start with the project's prefix"
languages: [typescript, tsx]
fix: manual
fix_reason: "renaming a tag changes every page that uses it"
stability: preview
options:
  prefix: { type: string, default: "", description: "the prefix every tag must start with" }
since: 0.1.0
---

Every component's `tag` starts with the project's prefix (the `prefix` option); with no prefix set,
a tag still needs one.

**Why:** One prefix keeps a project's elements apart from other libraries'.

```codelaw
native: stencil/tag-prefix
```

## How to fix

1. Rename the tag to start with the project's prefix.
2. Update every usage of the element.

## Examples

### TypeScript

#### Don't

```ts
@Component({ tag: 'card' })
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
@Component({ tag: 'card' })
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
