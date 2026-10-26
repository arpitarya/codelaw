---
id: sec-py/no-xpath-injection
title: 'XPath expressions are not built from strings'
languages: [python]
fix: manual
fix_reason: "the expression has to be rewritten around `$name` variables, and which parts are values is the author's call"
stability: preview
security: { cwe: [643], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

An XPath expression is not made with an f-string, `%`, `+` or `.format`; values go in as XPath variables.

**Why:** A quote in the value ends the string in the expression and the rest of the value is read as XPath (CWE-643).

```codelaw
match:
  kind: call
  all:
    - { has: { field: function, regex: '\.(xpath|findall|find|findtext|iterfind)$' } }
    - has:
        field: arguments
        has:
          nthChild: 1
          any:
            - { kind: string, has: { kind: interpolation } }
            - { kind: binary_operator, has: { field: operator, regex: '^(%|\+)$' } }
            - { kind: call, has: { field: function, regex: '\.format$' } }
          regex: '(//|/|\[@)'
message: 'This XPath is built from a string; pass the values as variables'
```

## How to fix

1. Write the expression with variables: `tree.xpath("//user[@name=$name]", name=name)` (lxml).
2. ElementTree has no variables: select by a simple path and compare the attribute in Python.

## Examples

### Python

#### Don't

```py
nodes = tree.xpath(f"//user[@name='{name}']")
```

#### Do

```py
nodes = tree.xpath("//user[@name=$name]", name=name)
```
