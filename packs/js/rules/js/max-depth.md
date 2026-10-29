---
id: js/max-depth
title: "Blocks nest no deeper than a maximum"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "flattening nested blocks changes the function's structure; which guard to invert or which block to extract is the author's call"
stability: preview
options:
  max: { type: "integer", default: 4, description: "the most blocks that may nest inside one another in a function" }
ported_from: [eslint/max-depth]
since: 0.5.0
---

Blocks nest at most `max` deep (4 by default). A block is a statement that opens its own scope of control: `if`, `switch`, `try`, `for`, `for…in`, `for…of`, `while`, `do…while` and `with`. An `else if` continues its chain and adds no depth. Each function starts again at zero, so a callback is not charged for the blocks around it. The rule reports on each statement that sits too deep: `Blocks are nested 5 deep (max 4)`.

**Why:** Every level of nesting is another condition a reader must keep in mind to know when a line runs. Deep nesting is also where the bug lives: the `else` that belongs to the wrong `if`.

```codelaw
native: js/max-depth
```

## How to fix

1. Turn a guard into an early `return`, `continue` or `throw`, so the rest of the function sits one level up.
2. Extract the body of the innermost block into a well-named function.
3. Replace a nested pair of `if`s with one condition joined by `&&`.

## Examples

### JavaScript

#### Don't

```js options={"max":2}
function f(a) {
  if (a) {
    for (const x of a) {
      if (x) {
        x();
      }
    }
  }
}
```

#### Do

```js options={"max":2}
function f(a) {
  if (!a) return;
  for (const x of a) {
    if (x) x();
  }
}
```

### JSX

#### Don't

```jsx options={"max":1}
function List({ items }) {
  if (items) {
    if (items.length) {
      return <ul />;
    }
  }
  return null;
}
```

#### Do

```jsx options={"max":1}
function List({ items }) {
  if (!items?.length) return null;
  return <ul />;
}
```

### TypeScript

#### Don't

```ts options={"max":2}
function f(a?: Array<() => void>) {
  if (a) {
    for (const x of a) {
      if (x) {
        x();
      }
    }
  }
}
```

#### Do

```ts options={"max":2}
function f(a?: Array<() => void>) {
  if (!a) return;
  for (const x of a) {
    if (x) x();
  }
}
```

### TSX

#### Don't

```tsx options={"max":1}
function List({ items }: { items?: string[] }) {
  if (items) {
    if (items.length) {
      return <ul />;
    }
  }
  return null;
}
```

#### Do

```tsx options={"max":1}
function List({ items }: { items?: string[] }) {
  if (!items?.length) return null;
  return <ul />;
}
```
