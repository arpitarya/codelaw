---
id: react/no-is-mounted
title: "No this.isMounted()"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the callback has to be cancelled where it starts"
stability: preview
ported_from: [eslint-plugin-react/no-is-mounted]
since: 0.1.0
---

Code doesn't call `this.isMounted()`.

**Why:** `isMounted` hides leaks: a callback that outlives the component should be cancelled
instead.

```codelaw
match:
  pattern: this.isMounted()
message: "`isMounted` is an anti-pattern and gone from ES6 classes"
```

## How to fix

1. Cancel pending work in `componentWillUnmount`.
2. Remove the `isMounted` check.

## Examples

### JavaScript

#### Don't

```js
class A extends React.Component {
  done() {
    if (this.isMounted()) {
      this.setState({ ok: true });
    }
  }
}
```

#### Do

```js
class A extends React.Component {
  componentWillUnmount() {
    this.request.cancel();
  }
  done() {
    this.setState({ ok: true });
  }
}
```

### JSX

#### Don't

```jsx
class A extends React.Component {
  done() {
    if (this.isMounted()) {
      this.setState({ ok: true });
    }
  }
}
```

#### Do

```jsx
class A extends React.Component {
  componentWillUnmount() {
    this.request.cancel();
  }
  done() {
    this.setState({ ok: true });
  }
}
```

### TSX

#### Don't

```tsx
class A extends React.Component {
  done() {
    if (this.isMounted()) {
      this.setState({ ok: true });
    }
  }
}
```

#### Do

```tsx
class A extends React.Component {
  componentWillUnmount() {
    this.request.cancel();
  }
  done() {
    this.setState({ ok: true });
  }
}
```
