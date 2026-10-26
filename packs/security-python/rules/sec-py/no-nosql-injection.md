---
id: sec-py/no-nosql-injection
title: 'A request is not a MongoDB query'
languages: [python]
fix: manual
fix_reason: "which fields the client may filter on, and their types, are the author's decision"
stability: preview
security: { cwe: [943], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A value from the request is not used whole as a `pymongo` filter, and a `$where` expression is not built from a request value.

**Why:** A JSON body such as `{"$ne": null}` or `{"$gt": ""}` in a query field matches every document, and `$where` runs JavaScript (CWE-943). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any:
    - all:
        - { kind: call }
        - has:
            field: function
            regex: '\.(find|find_one|find_one_and_update|find_one_and_delete|find_one_and_replace|update_one|update_many|delete_one|delete_many|count_documents|replace_one)$'
        - { has: { field: arguments, has: { nthChild: 1, pattern: $Q } } }
    - all:
        - { kind: pair }
        - { has: { field: key, regex: '^[''"]\$where[''"]$' } }
        - { has: { field: value, pattern: $Q } }
where: { $Q: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$Q` comes from the request and is used as a MongoDB query'
```

## How to fix

1. Build the filter yourself from named fields: `{"name": str(request.json["name"])}`; `str()` stops an operator object from getting through.
2. Validate the body against a schema (pydantic, marshmallow) before it reaches the query.
3. Do not use `$where`; express the condition with query operators.

## Examples

### Python

#### Don't

```py
def find(request):
    return db.users.find_one(request.json)
```

#### Do

```py
def find(request):
    name = str(request.json["name"])
    return db.users.find_one({"name": name})
```
