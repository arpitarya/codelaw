---
id: spell/known-typo-identifier
title: "No known typos in the names a file declares"
languages: [javascript, jsx, typescript, tsx, python]
fix: auto
stability: preview
since: 0.3.0
---

A name declared in the file — a variable, a function, a class, a parameter, a type, an interface, a class member — holds a word on the pack's list of known typos (`recieveAll`, `lenght`, `AdressBook`). The name is split at case changes, `_`, `-`, `.` and digits, and each part of four letters or more is looked up. It is reported once, where it is declared, never at each use.

**Why:** a misspelled name spreads: every caller copies it, and `grep receive` never finds `recieveAll`. Fixing it is cheapest where it is declared, before it has callers in other files.

The fix renames the declaration and every use in the file, and only when that is safe from this file alone: the name is not exported, not a module-level name of a script or a Python module, not a Python parameter (a caller may pass it by keyword), not a class attribute or member (read through `.`), not a shorthand property (`{ recieve }` is a key too), the new name is not already taken, and every place the name is written in the file is a use the semantic model resolved to it. Otherwise the finding has no edit, and the steps below apply. Imported names are another module's and are not reported.

```codelaw
native: spell/known-typo-identifier
```

## How to fix

1. Run `codelaw fix`: a file-local name is renamed at its declaration and every use.
2. For an exported name, a module-level name or a member, rename it with your editor's rename across the project, and keep the old name as a deprecated alias if other packages import it.
3. For a name that is right as it stands (a product name, a field another system defines), excuse the line: `// codelaw-ignore spell/known-typo-identifier: <reason>`.

## Examples

### JavaScript

#### Don't

```js
export function total(items) {
  let lenght = 0;
  for (const item of items) lenght += item.size;
  return lenght;
}
```

#### Do

```js
export function total(items) {
  let length = 0;
  for (const item of items) length += item.size;
  return length;
}
```

### JSX

#### Don't

```jsx
import { useState } from "react";

const AdressLine = ({ text }) => <span>{text}</span>;

export const Card = () => <AdressLine text={useState("")[0]}></AdressLine>;
```

#### Do

```jsx
import { useState } from "react";

const AddressLine = ({ text }) => <span>{text}</span>;

export const Card = () => <AddressLine text={useState("")[0]}></AddressLine>;
```

### TypeScript

#### Don't

```ts
type AdressBook = Record<string, string>;

export function lookUp(book: unknown, key: string): string | undefined {
  return (book as AdressBook)[key];
}
```

#### Do

```ts
type AddressBook = Record<string, string>;

export function lookUp(book: unknown, key: string): string | undefined {
  return (book as AddressBook)[key];
}
```

### TSX

#### Don't

```tsx
export const List = ({ rows }: { rows: string[] }) => {
  const recievedRows = rows.filter(Boolean);
  return <ul>{recievedRows.map((r) => <li key={r}>{r}</li>)}</ul>;
};
```

#### Do

```tsx
export const List = ({ rows }: { rows: string[] }) => {
  const receivedRows = rows.filter(Boolean);
  return <ul>{receivedRows.map((r) => <li key={r}>{r}</li>)}</ul>;
};
```

### Python

#### Don't

```python
def total(items):
    lenght = 0
    for item in items:
        lenght += item.size
    return lenght
```

#### Do

```python
def total(items):
    length = 0
    for item in items:
        length += item.size
    return length
```
