---
id: sec-py/no-exec-eval
title: "`exec`, `eval` and `compile` of a non-literal"
languages: [python]
fix: manual
fix_reason: "the safe replacement (`ast.literal_eval`, a lookup table, a parser) depends on what the string is for"
stability: preview
ported_from: [bandit/B102, bandit/B307]
security: { cwe: [94], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

`exec`, `eval` and `compile` are not given a value that is not a string literal.

**Why:** The string is run as code, so whoever controls it controls the program (CWE-94).

```codelaw
match:
  any:
    - { pattern: 'exec($X)' }
    - { pattern: 'exec($X, $$$R)' }
    - { pattern: 'eval($X)' }
    - { pattern: 'eval($X, $$$R)' }
    - all:
        - { pattern: 'compile($X, $$$R)' }
        - not:
            inside:
              kind: argument_list
              inside: { kind: call, has: { field: function, regex: '^(exec|eval)$' } }
constraints: { X: { not: { kind: string, not: { has: { kind: interpolation } } } } }
message: '`$X` is run as code; use `ast.literal_eval` or a lookup table'
```

## How to fix

1. For a literal (a number, a list, a dict), call `ast.literal_eval`.
2. For a choice among behaviours, map the input to a function through a dict.
3. For arithmetic or expressions, use a parser built for it, not the interpreter.

## Examples

### Python

#### Don't

```py
result = eval(user_text)
```

#### Do

```py
import ast

result = ast.literal_eval(user_text)
```
