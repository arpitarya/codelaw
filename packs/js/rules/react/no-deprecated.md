---
id: react/no-deprecated
title: "No deprecated React APIs"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "each deprecated API has its own replacement"
stability: preview
ported_from: [eslint-plugin-react/no-deprecated]
since: 0.1.0
---

Code doesn't use React APIs that are deprecated: the legacy `componentWill…` lifecycles,
`React.createClass`, `React.PropTypes`, `ReactDOM.render`, `hydrate` and `unmountComponentAtNode`.

**Why:** Deprecated APIs are removed in later versions, and the legacy lifecycles misbehave with
concurrent rendering.

```codelaw
match:
  any:
    - kind: method_definition
      has: { field: name, regex: ^(componentWillMount|componentWillReceiveProps|componentWillUpdate)$ }
    - pattern: React.createClass($$$A)
    - pattern: React.PropTypes
    - pattern: ReactDOM.render($$$A)
    - pattern: ReactDOM.hydrate($$$A)
    - pattern: ReactDOM.unmountComponentAtNode($$$A)
    - pattern: ReactDOMServer.renderToNodeStream($$$A)
message: "This React API is deprecated and is removed in a later version"
```

## How to fix

1. Replace `ReactDOM.render` with `createRoot(el).render(…)` and `hydrate` with `hydrateRoot`.
2. Rename legacy lifecycles to their `UNSAFE_` names, or move their work to `componentDidMount`/`componentDidUpdate`.
3. Use the `prop-types` package instead of `React.PropTypes`.

## Examples

### JavaScript

#### Don't

```js
ReactDOM.render(<App />, root);
```

#### Do

```js
createRoot(root).render(<App />);
```

### JSX

#### Don't

```jsx
ReactDOM.render(<App />, root);
```

#### Do

```jsx
createRoot(root).render(<App />);
```

### TSX

#### Don't

```tsx
ReactDOM.render(<App />, root);
```

#### Do

```tsx
createRoot(root).render(<App />);
```

## Differences from the source

The React version setting is not read: every listed API is reported.
