---
id: sec-py/no-sql-string
title: 'SQL built from a string'
languages: [python]
fix: manual
fix_reason: "which placeholder style the driver takes, and where each value goes, depends on the driver"
stability: preview
ported_from: [bandit/B608, bandit/B610, bandit/B611]
security: { cwe: [89], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A SQL statement is not built with an f-string, `%`, `+` or `.format`; the values go in as parameters.

**Why:** A value that is part of the statement text can change the statement (CWE-89).

```codelaw
match:
  any:
    - kind: call
      all:
        - has:
            field: function
            regex: '^((.*\.)?(execute|executemany|executescript|raw|read_sql|read_sql_query)|RawSQL|text|sa\.text|sqlalchemy\.text)$'
        - has:
            field: arguments
            has:
              nthChild: 1
              any:
                - { kind: string, has: { kind: interpolation } }
                - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
                - { kind: call, has: { field: function, regex: '\.format$' } }
    - kind: call
      all:
        - { has: { field: function, regex: '\.extra$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              has:
                field: value
                any:
                  - { kind: string, has: { kind: interpolation } }
                  - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
                  - { kind: call, has: { field: function, regex: '\.format$' } }
                  - kind: list
                    has:
                      any:
                        - { kind: string, has: { kind: interpolation } }
                        - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
                        - { kind: call, has: { field: function, regex: '\.format$' } }
message: 'This SQL is built from a string; pass the values as parameters'
```

## How to fix

1. Write the statement with placeholders: `cursor.execute("SELECT * FROM t WHERE id = %s", (item_id,))`.
2. Pass the values as the second argument; the driver quotes them.
3. A table or column name cannot be a parameter: choose it from a fixed allow-list.
4. In Django use the ORM, `.raw(sql, params)` or `RawSQL(sql, params)`; give `.extra` parameters through `params=`.

## Examples

### Python

#### Don't

```py
def find(cursor, name):
    cursor.execute(f"SELECT * FROM users WHERE name = '{name}'")
```

#### Do

```py
def find(cursor, name):
    cursor.execute("SELECT * FROM users WHERE name = %s", (name,))
```
