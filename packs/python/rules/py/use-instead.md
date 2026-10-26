---
id: py/use-instead
title: "Use the project's function instead of the one it wraps"
languages: [python]
fix: auto
stability: preview
options:
  replacements:
    type: "object"
    default: {}
    description: "a dotted path to { use, from, drop_in?, allow_in? }: `use` is the function or variable to call instead, `from` the module it comes from, `drop_in` lets `codelaw fix` swap it in and add the import, `allow_in` lists globs of files exempt (the wrapper itself)"
since: 0.3.0
---

A project names the function it wants called instead of another, once, and codelaw holds the code to it. The rule is inert until `replacements` is set. A key is a dotted path, matched on a call's callee or an attribute read, exactly or as a prefix at a `.` boundary (`os.environ` matches `os.environ.get`), and only where its first name is a builtin, a global or an import, never a local that shadows it. A file that matches the entry's `allow_in` is exempt.

```toml
[rules."py/use-instead".options.replacements]
"requests.get" = { use = "http_get", from = "acme.http", drop_in = true, allow_in = ["src/acme/http.py"] }
"print"        = { use = "log_info", from = "acme.log" }
"os.environ"   = { use = "env",      from = "acme.env" }
```

**Why:** A wrapper is where a team puts what every call needs - a timeout, an auth header, a logger. Code that calls the wrapped function directly skips all of it, and only a rule can notice.

**Limits:** it matches by the spelling of a path and where its first name comes from (`import requests as r` is read as `r`); it does not follow a value through a variable (`get = requests.get`). The wrapper calls the thing it wraps, so name its file in `allow_in`.

```codelaw
native: py/use-instead
```

## How to fix

1. With `drop_in = true`, run `codelaw fix`: the path becomes `use` and the import is added when the file lacks it.
2. Without `drop_in`, change the call by hand: the replacement takes other arguments or returns another shape.
3. In the wrapper's own file, add the file to `allow_in` rather than the call to the baseline.

## Examples

### Python

#### Don't

```py options={"replacements":{"print":{"use":"log_info","from":"acme.log","drop_in":true}}}
print("loaded")
```

#### Do

```py options={"replacements":{"print":{"use":"log_info","from":"acme.log","drop_in":true}}}
from acme.log import log_info
log_info("loaded")
```
