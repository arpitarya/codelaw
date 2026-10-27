---
id: fixture/no-debugger
title: No debugger statement
languages: [javascript]
fix: auto
stability: stable
---

The GitHub Action fixture's rule (W-97): a `debugger` left in the code.

**Why:** CI's `action` job needs one finding to annotate and to put in SARIF, with no rule pack installed.

```codelaw
match:
  kind: debugger_statement
message: "Remove the debugger statement"
fix: [remove]
```

## Examples

### JavaScript

#### Don't

```js
function run() {
  debugger;
  return 1;
}
```

#### Do

```js
function run() {
  return 1;
}
```
