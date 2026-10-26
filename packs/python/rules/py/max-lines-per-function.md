---
id: py/max-lines-per-function
title: "A function stays under a maximum number of lines"
languages: [python]
fix: manual
fix_reason: "extracting part of a function changes its structure; where to cut and what to call the piece is the author's call"
stability: preview
options:
  max: { type: "integer", default: 50, description: "the most lines one function may span" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "lines that hold only a comment do not count" }
  skip_docstrings: { type: "boolean", default: true, description: "the lines of the function's docstring (and of a nested one's) do not count" }
  include_nested: { type: "boolean", default: true, description: "the lines of an inner function count toward the outer one too" }
  ignore_names: { type: "string", description: "a regular expression; functions named like this are exempt, e.g. \"^test_\"" }
since: 0.3.0
---

A function (`def` or `async def`, a method included) spans at most `max` lines (50 by default), from its `def` line to
its last. Blank lines, comment-only lines and docstring lines are not counted unless `skip_blank_lines`,
`skip_comments` or `skip_docstrings` is `false`. With `include_nested` (the default) an inner function's lines count
toward the outer function as well; with `include_nested = false` they do not, from the inner `def` to its last
line. `ignore_names` is a regular expression for names to exempt, such as `"^test_"`. The rule reports on the
function's name: `render_orders is 84 lines (max 50)`. A `lambda` is an expression, not a function here.

**Why:** A function that long does several jobs, and its name can say only one. The limit is a prompt to extract, not a goal in
itself: a long table of test data is long on purpose, so exempt it with `ignore_names`.

```codelaw
native: py/max-lines-per-function
```

## How to fix

1. Find a block inside the function that does one thing (a loop body, a branch, a run of statements that build one value).
2. Extract it into a well-named function, and pass in what it reads.
3. Repeat until each piece fits `max`. If the function is long on purpose, name it in `ignore_names`, or freeze it with `codelaw baseline add --rule py/max-lines-per-function`.

## Examples

### Python

#### Don't

```py options={"max":3}
def total(items):
    result = 0
    for item in items:
        result += item
    return result
```

#### Do

```py options={"max":3}
def total(items):
    return sum(items)
```
