---
id: redux-toolkit/no-immutable-js
title: "State is plain JavaScript, not Immutable.js"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "every read and update of the state is rewritten for plain objects"
stability: preview
since: 0.1.0
---

Nothing is imported from `immutable` (Immutable.js).

**Why:** Immutable.js values spread through the code that reads them, need conversion at every boundary and
don't serialize as plain data; Redux Toolkit's Immer gives immutable updates on plain objects. *Style
Guide, Priority B: Use Plain JavaScript Objects for State.*

```codelaw
match:
  kind: import_statement
  has: { field: source, regex: "^['\"]immutable['\"]$" }
message: "Keep state in plain objects and arrays, and update it with Redux Toolkit's Immer, not Immutable.js"
```

## How to fix

1. Store the state as plain objects and arrays (`fromJS(x).toJS()` once, at the boundary).
2. Rewrite the reducers with `createSlice`, and the reads (`state.get('a')`) as property access.

## Examples

### JavaScript

#### Don't

```js
import { Map } from 'immutable';
```

#### Do

```js
import { createSlice } from '@reduxjs/toolkit';
```

### JSX

#### Don't

```jsx
import { Map } from 'immutable';
```

#### Do

```jsx
import { createSlice } from '@reduxjs/toolkit';
```

### TypeScript

#### Don't

```ts
import { Map } from 'immutable';
```

#### Do

```ts
import { createSlice } from '@reduxjs/toolkit';
```

### TSX

#### Don't

```tsx
import { Map } from 'immutable';
```

#### Do

```tsx
import { createSlice } from '@reduxjs/toolkit';
```
