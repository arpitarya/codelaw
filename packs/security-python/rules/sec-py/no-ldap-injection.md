---
id: sec-py/no-ldap-injection
title: 'LDAP filters are not built from strings'
languages: [python]
fix: manual
fix_reason: "the escaping function belongs to the LDAP library in use"
stability: preview
security: { cwe: [90], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

An LDAP search filter is not made with an f-string, `%`, `+` or `.format`.

**Why:** A value with `*`, `)` or `(` in it changes which entries the filter matches, which can turn a login check into a match-all (CWE-90).

```codelaw
match:
  kind: call
  all:
    - has: { field: function, regex: '\.(search|search_s|search_st|search_ext|search_ext_s)$' }
    - has:
        field: arguments
        has:
          regex: '\(.*='
          any:
            - { kind: string, has: { kind: interpolation } }
            - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
            - { kind: call, has: { field: function, regex: '\.format$' } }
message: 'This LDAP filter is built from a string; escape the values'
```

## How to fix

1. Escape each value: `ldap3.utils.conv.escape_filter_chars(value)` or `ldap.filter.escape_filter_chars(value)`.
2. Or build the filter with `ldap.filter.filter_format("(uid=%s)", [value])`.

## Examples

### Python

#### Don't

```py
conn.search_s(base, ldap.SCOPE_SUBTREE, f"(uid={name})")
```

#### Do

```py
conn.search_s(base, ldap.SCOPE_SUBTREE, filter_format("(uid=%s)", [name]))
```
