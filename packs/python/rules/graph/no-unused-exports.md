---
id: graph/no-unused-exports
title: Exported symbols must be used somewhere
languages: [python]
scope: project
severity: error
fix: suggest
fix_reason: "Static analysis can't rule out reflective or external use, so removal needs a human or agent to accept the diff."
stability: preview
ported_from: [vulture/unused-function]
since: 0.1.0
---

Every name a module exports is imported by another module, is reachable from an entry point, or is marked `# codelaw: keep`.

**Why:** a module-level function or class nothing imports is dead code that still looks like API. A module with `__all__` exports what it lists; without one, a public name counts only when its own module doesn't use it either.

```codelaw
graph:
  symbol:
    exported: true
    kind: [function, class, variable, type]
  unless:
    - reachable_from: entry
    - tagged: [public, keep]
  when:
    external_references: 0
  certainty_from: module.dynamic
message: "`$SYMBOL` is exported but never used outside `$FILE`"
fix:
  - when: { local_references: ">0" }
    do: [strip-export-keyword]
  - else:
    do: [remove-declaration]
```

## How to fix

1. Run `codelaw graph why-unused <symbol>` (`<file>#<export>` or `<module>:<name>`) to see each entry, keep and import clause.
2. If something outside the repo uses it, add the file to `[graph] entry` or `[graph] keep`, add its decorator to `[graph] registrars`, or tag it `/** @public */` / `# codelaw: keep`.
3. Otherwise delete the declaration (and its `__all__` or `export { … }` entry), or drop `export` if its own file still uses it.

## Examples

### Python

#### Don't

```py files
# app/dates.py
def format_date(d):
    return d.isoformat()
def parse_date(s):
    return s
# app/main.py
from app.dates import format_date
print(format_date(None))
```

#### Do

```py files
# app/dates.py
def format_date(d):
    return d.isoformat()
# app/main.py
from app.dates import format_date
print(format_date(None))
```
