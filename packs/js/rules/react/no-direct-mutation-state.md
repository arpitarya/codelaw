---
id: react/no-direct-mutation-state
title: "No assigning to this.state outside the constructor"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the update has to be rewritten as a `setState` call"
stability: preview
ported_from: [eslint-plugin-react/no-direct-mutation-state]
since: 0.1.0
---

A class component changes its state with `this.setState`, never by assigning to `this.state` outside
the constructor.

**Why:** A direct assignment doesn't trigger a re-render, and a later `setState` may overwrite it.

```codelaw
match:
  kind: assignment_expression
  has:
    field: left
    regex: ^this\.state\b
  not:
    inside:
      kind: method_definition
      has: { field: name, regex: ^constructor$ }
      stopBy: end
  inside: { kind: class_body, stopBy: end }
message: "Assigning to `this.state` doesn't re-render; call `this.setState`"
```

## How to fix

1. Replace the assignment with `this.setState(…)`; use the updater form when the new state depends on the old.

## Examples

### JavaScript

#### Don't

```js
class Counter extends React.Component {
  inc() {
    this.state.count = this.state.count + 1;
  }
}
```

#### Do

```js
class Counter extends React.Component {
  inc() {
    this.setState((s) => ({ count: s.count + 1 }));
  }
}
```

### JSX

#### Don't

```jsx
class Counter extends React.Component {
  inc() {
    this.state.count = this.state.count + 1;
  }
}
```

#### Do

```jsx
class Counter extends React.Component {
  inc() {
    this.setState((s) => ({ count: s.count + 1 }));
  }
}
```

### TSX

#### Don't

```tsx
class Counter extends React.Component {
  inc() {
    this.state.count = this.state.count + 1;
  }
}
```

#### Do

```tsx
class Counter extends React.Component {
  inc() {
    this.setState((s) => ({ count: s.count + 1 }));
  }
}
```
