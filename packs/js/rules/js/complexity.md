---
id: js/complexity
title: "A function's cyclomatic complexity stays under a maximum"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "splitting a function changes its structure; where to cut and what to call each piece is the author's call"
stability: preview
options:
  max: { type: "integer", default: 20, description: "the highest cyclomatic complexity a function may have" }
  count_optional_chain: { type: "boolean", default: false, description: "count each optional chain, `a?.b`, as a branch" }
  ignore_names: { type: "string", description: "a regular expression; functions named like this are exempt" }
ported_from: [eslint/complexity]
since: 0.5.0
---

A function's cyclomatic complexity is 1 plus the number of ways control can branch inside it: each `if` (an `else if` too), `?:`, `for`, `for…in`/`for…of`, `while`, `do…while`, `case` (not `default`), `catch`, and each `&&`, `||` and `??` (their assignment forms `&&=`, `||=` and `??=` as well). A function nested in another is counted on its own and not in the outer one. `count_optional_chain` adds each `?.`. The rule reports on the function's name when more than `max` (20 by default) is reached: `route has a complexity of 24 (max 20)`. `ignore_names` is a regular expression for names to exempt.

**Why:** Each branch is another path a reader must hold in mind and a test must cover. Past about twenty a function is not understood, only trusted. The limit is a prompt to split, not a goal in itself.

```codelaw
native: js/complexity
```

## How to fix

1. Find a branch that decides one thing (a `switch` over a kind, a run of `if`s that validate input) and extract it into a well-named function that returns the result.
2. Replace a long `if`/`else if` chain on one value with a lookup table or a small map of handlers.
3. Return early on the guard cases so the main path has fewer nested branches.
4. If the function is complex on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule js/complexity`.

## Examples

### JavaScript

#### Don't

```js options={"max":3}
function label(n) {
  if (n === 0) return "zero";
  if (n === 1) return "one";
  if (n === 2) return "two";
  return "many";
}
```

#### Do

```js options={"max":3}
const LABELS = ["zero", "one", "two"];
function label(n) {
  return LABELS[n] ?? "many";
}
```

### JSX

#### Don't

```jsx options={"max":2}
function Badge({ a, b }) {
  if (a && b) return <b>both</b>;
  return <i>{a || b ? "one" : "none"}</i>;
}
```

#### Do

```jsx options={"max":2}
function Badge({ count }) {
  return <i>{count}</i>;
}
```

### TypeScript

#### Don't

```ts options={"max":3}
function label(n: number): string {
  if (n === 0) return "zero";
  if (n === 1) return "one";
  if (n === 2) return "two";
  return "many";
}
```

#### Do

```ts options={"max":3}
const LABELS = ["zero", "one", "two"];
function label(n: number): string {
  return LABELS[n] ?? "many";
}
```

### TSX

#### Don't

```tsx options={"max":2}
function Badge({ a, b }: { a: boolean; b: boolean }) {
  if (a && b) return <b>both</b>;
  return <i>{a || b ? "one" : "none"}</i>;
}
```

#### Do

```tsx options={"max":2}
function Badge({ count }: { count: number }) {
  return <i>{count}</i>;
}
```
