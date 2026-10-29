---
id: py/complexity
title: "A function's cyclomatic complexity stays under a maximum"
languages: [python]
fix: manual
fix_reason: "splitting a function changes its structure; where to cut and what to call each piece is the author's call"
stability: preview
options:
  max: { type: "integer", default: 20, description: "the highest cyclomatic complexity a function may have" }
  ignore_names: { type: "string", description: "a regular expression; functions named like this are exempt" }
ported_from: [mccabe/C901, ruff/C901, pylint/too-many-branches]
since: 0.5.0
---

A function's cyclomatic complexity is 1 plus the number of ways control can branch inside it: each `if` and `elif`, `for`, `while`, `except`, conditional expression (`a if b else c`), each `and` and `or`, each `for` and `if` of a comprehension, and each `case` of a `match` (the wildcard `case _:` is the fall-through and does not count). A `def` nested in another is counted on its own and not in the outer one. The rule reports on the function's name when more than `max` (20 by default, McCabe's and Ruff's `C901` default being 10 — the stricter value is one `max` away) is reached: `route has a complexity of 24 (max 20)`. `ignore_names` is a regular expression for names to exempt.

**Why:** Each branch is another path a reader must hold in mind and a test must cover. Past about twenty a function is not understood, only trusted. The limit is a prompt to split, not a goal in itself.

```codelaw
native: py/complexity
```

## How to fix

1. Find a branch that decides one thing (a chain of `elif` over a kind, a run of `if`s that validate input) and extract it into a well-named function that returns the result.
2. Replace a long `if`/`elif` chain on one value with a dictionary of handlers, or a `match` that calls one function per case.
3. Return early on the guard cases so the main path has fewer nested branches.
4. If the function is complex on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule py/complexity`.

## Examples

### Python

#### Don't

```py options={"max":3}
def label(n):
    if n == 0:
        return "zero"
    elif n == 1:
        return "one"
    elif n == 2:
        return "two"
    return "many"
```

#### Do

```py options={"max":3}
LABELS = ["zero", "one", "two"]


def label(n):
    return LABELS[n] if n < len(LABELS) else "many"
```
