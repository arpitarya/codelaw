---
id: js/no-dupe-class-members
title: "No two class members share a name"
languages: [javascript, jsx]
fix: manual
fix_reason: "which of the two members to keep is a decision"
stability: preview
ported_from: [eslint/no-dupe-class-members]
since: 0.1.0
---

A class declares each method and field name once, separately for static and instance members; a
getter and a setter may share a name.

**Why:** The later declaration replaces the earlier one, so one of them is dead code.

```codelaw
native: js/no-dupe-class-members
```

## How to fix

1. Decide which declaration is right.
2. Remove or rename the other.

## Examples

### JavaScript

#### Don't

```js
class Cart {
  total() {
    return 1;
  }
  total() {
    return 2;
  }
}
```

#### Do

```js
class Cart {
  total() {
    return 1;
  }
  subtotal() {
    return 2;
  }
}
```

### JSX

#### Don't

```jsx
class Cart {
  total() {
    return 1;
  }
  total() {
    return 2;
  }
}
```

#### Do

```jsx
class Cart {
  total() {
    return 1;
  }
  subtotal() {
    return 2;
  }
}
```
