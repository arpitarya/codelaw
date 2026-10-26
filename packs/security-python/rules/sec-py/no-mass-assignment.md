---
id: sec-py/no-mass-assignment
title: 'A request is not splatted into a model'
languages: [python]
fix: manual
fix_reason: "which fields a client may set is the author's decision"
stability: preview
security: { cwe: [915], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

`Model(**request.POST)`, `Model.objects.create(**request.data)` and `update(**request.json)` are not used, and a `ModelForm` or serializer does not set `fields = "__all__"`.

**Why:** The client chooses which columns are set, so it can set `is_staff`, `owner_id` or a price (CWE-915).

```codelaw
match:
  any:
    - all:
        - { kind: call }
        - has:
            field: function
            regex: '\.(create|update|get_or_create|update_or_create|bulk_create)$|^[A-Z][A-Za-z0-9_]*$'
        - has:
            field: arguments
            has:
              kind: dictionary_splat
              regex: '^\*\*\s*(self\.)?request\.(POST|GET|data|json|form|args|values|query_params)\b'
    - all:
        - { kind: assignment }
        - { has: { field: left, regex: '^fields$' } }
        - { has: { field: right, regex: '^[''"]__all__[''"]$' } }
message: 'The client chooses which fields are set; list the fields'
```

## How to fix

1. List the allowed fields: `fields = ["title", "body"]`.
2. Build the model from named values: `Post.objects.create(title=data["title"], owner=request.user)`.
3. Mark server-only fields `read_only_fields`.

## Examples

### Python

#### Don't

```py
def create(request):
    return Post.objects.create(**request.POST)
```

#### Do

```py
def create(request):
    return Post.objects.create(title=request.POST["title"], owner=request.user)
```
