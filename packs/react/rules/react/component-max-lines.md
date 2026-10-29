---
id: react/component-max-lines
title: "A React component stays under a maximum number of lines"
languages: [jsx, tsx]
fix: manual
fix_reason: "splitting a component means choosing which part becomes its own component and which props it takes; that is the author's call"
stability: preview
options:
  max: { type: "integer", default: 200, description: "the most lines one component may span" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "lines that hold only a comment do not count" }
  ignore_names: { type: "string", description: "a regular expression; components named like this are exempt" }
ported_from: ["@eslint-react/no-large-components"]
since: 0.5.0
---

A React component spans at most `max` lines (200 by default), from its first line to its last. A component is a function whose name starts with a capital letter and that holds JSX — a declaration, an arrow or a function expression, also inside `memo(…)` or `forwardRef(…)` — or a class that extends `Component` or `PureComponent`. Blank lines and comment-only lines are not counted unless `skip_blank_lines` or `skip_comments` is `false`. `js/max-lines-per-function` measures every function; this rule measures components only, so its limit can be looser (a component is mostly markup). The rule reports on the component's name: `Component Table is 245 lines (max 200)`.

**Why:** A component that long renders several things, and a change to one of them re-reads all of them. Split out the parts that have a name of their own.

```codelaw
native: react/component-max-lines
```

## How to fix

1. Find a block of markup with its own name (a row, a header, a modal) and move it into a component of its own, passing in what it reads as props.
2. Move state and effects that belong to one part into a custom hook (`useTableSort`).
3. If the component is long on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule react/component-max-lines`.

## Examples

### JSX

#### Don't

```jsx options={"max":4}
function Card({ title, body }) {
  const heading = <h2>{title}</h2>;
  const text = <p>{body}</p>;
  const footer = <small>end</small>;
  return <div>{heading}{text}{footer}</div>;
}
```

#### Do

```jsx options={"max":4}
function Card({ title, body }) {
  return <div><h2>{title}</h2><p>{body}</p></div>;
}
```

### TSX

#### Don't

```tsx options={"max":4}
function Card({ title, body }: CardProps) {
  const heading = <h2>{title}</h2>;
  const text = <p>{body}</p>;
  const footer = <small>end</small>;
  return <div>{heading}{text}{footer}</div>;
}
```

#### Do

```tsx options={"max":4}
function Card({ title, body }: CardProps) {
  return <div><h2>{title}</h2><p>{body}</p></div>;
}
```
