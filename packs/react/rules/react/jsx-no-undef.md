---
id: react/jsx-no-undef
title: "Components used in JSX are declared"
languages: [javascript, jsx, tsx]
fix: manual
fix_reason: "the right import is the author's to choose"
stability: preview
ported_from: [eslint-plugin-react/jsx-no-undef]
since: 0.1.0
---

Every component used as a JSX tag is imported or declared.

**Why:** An undeclared component throws a ReferenceError when it renders.

```codelaw
match:
  kind: identifier
  regex: ^[A-Z]
  inside:
    stopBy: neighbor
    any: [{ kind: jsx_opening_element }, { kind: jsx_self_closing_element }]
    field: name
where:
  $$MATCH: undeclared
message: "`$$MATCH` is not defined; import or declare the component"
```

## How to fix

1. Import the component, or declare it, or correct the name.

## Examples

### JavaScript

#### Don't

```js
export const App = () => <Header />;
```

#### Do

```js
import { Header } from "./header";
export const App = () => <Header />;
```

### JSX

#### Don't

```jsx
export const App = () => <Header />;
```

#### Do

```jsx
import { Header } from "./header";
export const App = () => <Header />;
```

### TSX

#### Don't

```tsx
export const App = () => <Header />;
```

#### Do

```tsx
import { Header } from "./header";
export const App = () => <Header />;
```
