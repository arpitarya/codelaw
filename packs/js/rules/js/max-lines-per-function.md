---
id: js/max-lines-per-function
title: "A function stays under a maximum number of lines"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "extracting part of a function changes its structure; where to cut and what to call the piece is the author's call"
stability: preview
options:
  max: { type: "integer", default: 50, description: "the most lines one function may span" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "lines that hold only a comment do not count" }
  include_nested: { type: "boolean", default: true, description: "the lines of an inner function count toward the outer one too" }
  include_iife: { type: "boolean", default: false, description: "measure an immediately invoked function, `(() => { … })()`, as a function of its own" }
  ignore_names: { type: "string", description: "a regular expression; functions named like this are exempt, e.g. \"^(test|it|describe)\"" }
ported_from: [eslint/max-lines-per-function]
since: 0.3.0
---

A function spans at most `max` lines (50 by default), from its first line to its last. Every function form counts:
declarations, expressions, arrows, methods, getters and setters, and class `static` blocks. Blank lines and comment-only
lines are not counted unless `skip_blank_lines` or `skip_comments` is `false`. With `include_nested` (the default) an
inner function's lines count toward the outer function as well; with `include_nested = false` they do not, from the
inner function's first line to its last. An immediately invoked function is not measured on its own unless
`include_iife` is `true`; its lines belong to the function around it. `ignore_names` is a regular expression for names
to exempt, such as `"^(test|it|describe)"`. The rule reports on the function's name: `renderOrders is 84 lines (max 50)`.

**Why:** A function that long does several jobs, and its name can say only one. The limit is a prompt to extract, not a goal in
itself: a test suite's `describe` callback is long on purpose, so exempt it with `ignore_names`.

```codelaw
native: js/max-lines-per-function
```

## How to fix

1. Find a block inside the function that does one thing (a loop body, a branch, a run of statements that build one value).
2. Extract it into a well-named function, and pass in what it reads.
3. Repeat until each piece fits `max`. If the function is long on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule js/max-lines-per-function`.

## Examples

### JavaScript

#### Don't

```js options={"max":3}
function total(items) {
  let sum = 0;
  for (const item of items) sum += item;
  return sum;
}
```

#### Do

```js options={"max":3}
function total(items) {
  return items.reduce((sum, item) => sum + item, 0);
}
```

### JSX

#### Don't

```jsx options={"max":3}
function List({ items }) {
  const rows = items.map((item) => <li>{item}</li>);
  const empty = <p>none</p>;
  return rows.length ? <ul>{rows}</ul> : empty;
}
```

#### Do

```jsx options={"max":3}
function List({ items }) {
  return <ul>{items.map((item) => <li>{item}</li>)}</ul>;
}
```

### TypeScript

#### Don't

```ts options={"max":3}
function total(items: number[]): number {
  let sum = 0;
  for (const item of items) sum += item;
  return sum;
}
```

#### Do

```ts options={"max":3}
function total(items: number[]): number {
  return items.reduce((sum, item) => sum + item, 0);
}
```

### TSX

#### Don't

```tsx options={"max":3}
function List({ items }: { items: string[] }) {
  const rows = items.map((item) => <li>{item}</li>);
  const empty = <p>none</p>;
  return rows.length ? <ul>{rows}</ul> : empty;
}
```

#### Do

```tsx options={"max":3}
function List({ items }: { items: string[] }) {
  return <ul>{items.map((item) => <li>{item}</li>)}</ul>;
}
```
