---
id: py/unsorted-imports
title: "Imports are sorted"
languages: [python]
fix: auto
stability: preview
options:
  known_first_party: { type: "string[]", default: [], description: "top-level modules that belong to this project" }
  line_length: { type: integer, default: 88, description: "the width a from-import may reach before it is wrapped" }
ported_from: [ruff/I001]
since: 0.1.0
---

Each block of consecutive imports is in isort's order: sections (`__future__`, standard library, third party, first party, local), plain `import x` before `from x import y` within a section, then by module name, and the names of a `from` import sorted (constants, classes, then the rest).

**Why:** A fixed order makes imports easy to scan and stops merge conflicts from reordering.

```codelaw
native: py/unsorted-imports
```

## Examples

### Python

#### Don't

```py
import sys
import os
```

#### Don't

```py
import requests
import os
```

#### Do

```py
import os
import sys
```

#### Do

```py
import os

import requests
```

## Differences from the source

- The rule checks order. Line wrapping and the blank lines after the import block belong to the formatter; the fix separates sections with one blank line and wraps a `from` import past `line_length`, as isort does.
- First-party modules are those listed in `known_first_party`; Ruff also detects them from the project's source folders.
- `import a, b` is not reported here (that is `py/multiple-imports-on-one-line`), and `from m import a, b as c` is not reported for mixing aliases; the fix still writes each on its own line.
- A block with comments the fix can't place (inside a parenthesised import, or loose between imports) is reported without an edit.
