---
id: acme/no-global-statement
title: "Functions don't rebind module variables with global"
languages: [python]
fix: manual
fix_reason: "the state has to move into a parameter, a return value or an object; which one fits depends on the callers"
stability: preview
ported_from: [pylint/global-statement]
since: 0.4.0
---

A function, method or nested function has no `global` statement. A `global` at module level is a
different check in the source tool and is not reported here; `nonlocal` is not reported either.

**Why:** A function that rebinds a module variable has an effect its signature does not show, and its
callers share hidden state that tests must reset. Ported from Pylint's `global-statement` (W0603) by the
`codelaw-port-rule` skill: if a first-party pack ships a rule with the same `ported_from:`, turn that one
on and delete this file.

```codelaw
match:
  kind: global_statement
  inside: { kind: function_definition, stopBy: end }
message: "`global` in a function rebinds module state: pass and return the value, or keep it on an object"
```

## How to fix

1. Find every function that names the variable in a `global` statement, and its callers.
2. Pass the current value in as a parameter and return the new one, assigning it where the call is; or
   keep the state on an object (a class instance, a small dataclass) that the callers share.
3. Delete the `global` statement and, when nothing at module level still needs it, the module variable.

## Examples

### Python

#### Don't

```py
hits = 0


def record():
    global hits
    hits += 1
```

#### Do

```py
class Counter:
    def __init__(self):
        self.hits = 0

    def record(self):
        self.hits += 1
```
