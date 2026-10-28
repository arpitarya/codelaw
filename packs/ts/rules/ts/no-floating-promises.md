---
id: ts/no-floating-promises
title: "A Promise is awaited, returned or handled"
languages: [typescript, tsx]
fix: suggest
fix_reason: "`void` marks the Promise as deliberately not awaited; awaiting it instead changes when the code after it runs, which only the author can decide"
stability: preview
ported_from: [typescript-eslint/no-floating-promises]
tags: [types, async]
since: 0.3.0
---

A statement that makes a Promise awaits it, returns it, handles its rejection (`.catch(…)`,
`.then(…, …)`) or marks it with `void`.

**Why:** A Promise nobody awaits can reject with nobody listening: the error is lost, or stops the
process as an unhandled rejection far from the line that caused it.

```codelaw
match:
  any: [{ kind: call_expression }, { kind: new_expression }]
  inside: { kind: expression_statement, stopBy: neighbor }
  not:
    any:
      - pattern: $O.catch($H)
      - pattern: $O.then($A, $B)
type:
  $$MATCH: promise-like
message: "This Promise is not awaited, returned or handled: a rejection would go unnoticed"
fix: "void $$MATCH"
```

## Examples

### TypeScript

#### Don't

```ts
async function save(): Promise<void> {}
save();
```

#### Do

```ts
async function save(): Promise<void> {}
async function main() {
  await save();
}
```

#### Fixed

```ts
async function save(): Promise<void> {}
void save();
```

### TSX

#### Don't

```tsx
async function save(): Promise<void> {}
save();
```

#### Do

```tsx
async function save(): Promise<void> {}
save().catch((e) => console.error(e));
```

#### Fixed

```tsx
async function save(): Promise<void> {}
void save();
```

## Differences from the source

Only a call or `new` alone in a statement is checked: typescript-eslint also checks a Promise left
in an array, a comma or a conditional expression.
