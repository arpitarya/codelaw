---
id: acme/no-sql-fstring
title: "SQL is parameterised, never built with an f-string"
languages: [python]
fix: manual
fix_reason: "the values must move out of the string into a parameter tuple, which changes the call's shape"
stability: preview
since: 0.2.0
---

A query passed to `execute`, `executemany`, `fetch_one` or `fetch_all` is a constant string with
`?` placeholders and a parameter tuple; it is never an f-string, a `%` format or a `.format()` call.

**Why:** a value spliced into SQL text is an injection; the driver's placeholders are not.

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: "(^|\\.)(execute|executemany|fetch_one|fetch_all)$"
    - has:
        field: arguments
        has:
          any:
            - kind: string
              has: { kind: interpolation }
            - kind: binary_operator
              has: { field: operator, regex: "^%$" }
            - kind: call
              has: { field: function, regex: "\\.format$" }
message: "Pass the values as parameters (`?` placeholders and a tuple), not spliced into the SQL"
```

## How to fix

1. Replace each value spliced into the query with a `?` placeholder.
2. Pass the values, in order, as a tuple: `execute(sql, (a, b))` or `fetch_one(conn, sql, (a, b))`.

## Examples

### Python

#### Don't

```python
cur.execute(f"SELECT * FROM users WHERE email = '{email}'")
```

#### Do

```python
cur.execute("SELECT * FROM users WHERE email = ?", (email,))
```
