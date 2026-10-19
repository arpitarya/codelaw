---
id: js/no-redeclare
title: "No variable is declared twice"
languages: [javascript, jsx]
fix: manual
fix_reason: "whether to rename or to assign needs the author"
stability: preview
ported_from: [eslint/no-redeclare]
since: 0.1.0
---

A name is declared once per scope: a second `var` or `function` with the same name is not allowed.

**Why:** Redeclaring silently merges two declarations, so one of them is usually a mistake.

```codelaw
match:
  kind: identifier
where:
  $$MATCH: [declaration, redeclared]
message: "`$$MATCH` is already declared in this scope"
```

## How to fix

1. If it's the same variable, drop the second `var` and assign.
2. If it's a different one, rename it.

## Examples

### JavaScript

#### Don't

```js
var count = 1;
var count = 2;
```

#### Do

```js
var count = 1;
count = 2;
```

### JSX

#### Don't

```jsx
var count = 1;
var count = 2;
```

#### Do

```jsx
var count = 1;
count = 2;
```
