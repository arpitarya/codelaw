---
id: redux-toolkit/object-shorthand-mapdispatch
title: "connect's mapDispatch is an object of action creators"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "each wrapped dispatch call is replaced by the action creator it calls"
stability: preview
since: 0.1.0
---

The second argument of `connect` from `react-redux` is an object of action creators, not a function
of `dispatch`. A function that takes `ownProps` too is left alone.

**Why:** `connect` binds an object of action creators to `dispatch` itself; the function form only repeats
that by hand. *Style Guide, Priority B: Use the Object Shorthand Form of mapDispatch with connect.*

```codelaw
native: redux-toolkit/object-shorthand-mapdispatch
```

## How to fix

1. Replace the function with an object of the action creators it wraps: `{ todoAdded }`.
2. Remove the function.

## Examples

### JavaScript

#### Don't

```js
import { connect } from 'react-redux';

const mapDispatch = (dispatch) => ({
  todoAdded: (text) => dispatch(todoAdded(text)),
});

export default connect(mapState, mapDispatch)(TodoList);
```

#### Do

```js
import { connect } from 'react-redux';

export default connect(mapState, { todoAdded })(TodoList);
```

### JSX

#### Don't

```jsx
import { connect } from 'react-redux';

const mapDispatch = (dispatch) => ({
  todoAdded: (text) => dispatch(todoAdded(text)),
});

export default connect(mapState, mapDispatch)(TodoList);
```

#### Do

```jsx
import { connect } from 'react-redux';

export default connect(mapState, { todoAdded })(TodoList);
```

### TypeScript

#### Don't

```ts
import { connect } from 'react-redux';

const mapDispatch = (dispatch) => ({
  todoAdded: (text) => dispatch(todoAdded(text)),
});

export default connect(mapState, mapDispatch)(TodoList);
```

#### Do

```ts
import { connect } from 'react-redux';

export default connect(mapState, { todoAdded })(TodoList);
```

### TSX

#### Don't

```tsx
import { connect } from 'react-redux';

const mapDispatch = (dispatch) => ({
  todoAdded: (text) => dispatch(todoAdded(text)),
});

export default connect(mapState, mapDispatch)(TodoList);
```

#### Do

```tsx
import { connect } from 'react-redux';

export default connect(mapState, { todoAdded })(TodoList);
```
