---
id: py/max-lines
title: "A module stays under a maximum number of lines"
languages: [python]
fix: manual
fix_reason: "splitting a module changes its structure; which names belong together is the author's call"
stability: preview
options:
  max: { type: "integer", default: 1000, description: "the most lines a module may have" }
  skip_blank_lines: { type: "boolean", default: true, description: "blank lines do not count" }
  skip_comments: { type: "boolean", default: true, description: "lines that hold only a comment do not count" }
  skip_docstrings: { type: "boolean", default: false, description: "the lines of docstrings do not count" }
  skip_imports: { type: "boolean", default: false, description: "the import block does not count" }
ported_from: [pylint/too-many-lines]
since: 0.3.0
---

A module has at most `max` lines (1000 by default, as Pylint's `too-many-lines`). Blank lines and lines that hold
only a comment are not counted unless `skip_blank_lines` or `skip_comments` is `false`; a line that holds code and a
comment counts as code. `skip_docstrings` leaves out the lines of module, class and function docstrings, and
`skip_imports` the top-level import block. The rule reports once, on the first line over the limit, with the measured
count: `1204 lines (max 1000; blank lines and comments not counted)`.

**Why:** A module that long is usually several modules: it is slow to review, and every import of it pays for all of it. The
limit is a prompt to split, not a goal in itself: raise `max` for a module that is long on purpose, or leave generated
code to `[files] lint_generated = false`. Only the host file is measured, never a code block in Markdown.

```codelaw
native: py/max-lines
```

## How to fix

1. Find a cohesive group of names (one class and its helpers, one feature's functions) and move it to a new module in the same package.
2. Import it back in the old module, or re-export it from the package's `__init__.py`, so callers do not change.
3. If the module is long on purpose, raise `max` for it in an `[[overrides]]` entry, or freeze it with `codelaw baseline add --rule py/max-lines`.

## Examples

### Python

#### Don't

```py options={"max":3}
a = 1
b = 2
c = 3
d = 4
```

#### Do

```py options={"max":3}
from more import d

a = 1
b = 2
```
