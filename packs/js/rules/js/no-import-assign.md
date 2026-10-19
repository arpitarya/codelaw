---
id: js/no-import-assign
title: "An imported binding is not reassigned"
languages: [javascript, jsx]
fix: manual
fix_reason: "the new value needs a variable of its own"
stability: preview
ported_from: [eslint/no-import-assign]
since: 0.1.0
---

Code doesn't assign a new value to a name bound by `import`.

**Why:** Imported bindings are read-only: assigning to one throws a TypeError.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [write, { binding: import }]
message: "`$$MATCH` is an import; imports are read-only"
```

## How to fix

1. Copy the import into a local variable and change that.

## Examples

### JavaScript

#### Don't

```js
import config from "./config.js";
config = {};
```

#### Do

```js
import config from "./config.js";
const local = { ...config };
```

### JSX

#### Don't

```jsx
import config from "./config.js";
config = {};
```

#### Do

```jsx
import config from "./config.js";
const local = { ...config };
```
