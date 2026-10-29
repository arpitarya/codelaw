---
id: py/max-depth
title: "Blocks nest no deeper than a maximum"
languages: [python]
fix: manual
fix_reason: "flattening nested blocks changes the function's structure; which guard to invert or which block to extract is the author's call"
stability: preview
options:
  max: { type: "integer", default: 4, description: "the most blocks that may nest inside one another in a function" }
ported_from: [pylint/too-many-nested-blocks, ruff/PLR1702]
since: 0.5.0
---

Blocks nest at most `max` deep (4 by default; Pylint's `too-many-nested-blocks` allows 5). A block is `if`, `for`, `while`, `try`, `with` or `match`. `elif` and `else` continue their `if` and add no depth. Each `def` starts again at zero, and so does the module's own code. The rule reports on each statement that sits too deep: `Blocks are nested 5 deep (max 4)`.

**Why:** Every level of nesting is another condition a reader must keep in mind to know when a line runs. Deep nesting is also where the bug lives: the `else` that belongs to the wrong `if`.

```codelaw
native: py/max-depth
```

## How to fix

1. Turn a guard into an early `return`, `continue` or `raise`, so the rest of the function sits one level up.
2. Extract the body of the innermost block into a well-named function.
3. Join a nested pair of `if`s with `and`, or flatten a `with` pair into one `with a, b:`.

## Examples

### Python

#### Don't

```py options={"max":2}
def f(a):
    if a:
        for x in a:
            if x:
                x()
```

#### Do

```py options={"max":2}
def f(a):
    if not a:
        return
    for x in a:
        if x:
            x()
```
