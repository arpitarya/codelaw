---
id: sec-py/no-open-redirect
title: 'A request value is not a redirect target'
languages: [python]
fix: manual
fix_reason: "the allowed destinations depend on the application"
stability: preview
security: { cwe: [601], owasp: ["A01:2025"], severity: medium }
since: 0.2.0
---

A value from the request is not passed to `redirect`, `HttpResponseRedirect` or `RedirectResponse`.

**Why:** A link on another site can send your users to a look-alike page that starts from yours (CWE-601). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^(redirect|(.*\.)?(HttpResponseRedirect|HttpResponsePermanentRedirect|RedirectResponse))$'
    - { has: { field: arguments, has: { nthChild: 1, pattern: $U } } }
where: { $U: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$U` comes from the request and is a redirect target'
```

## How to fix

1. Redirect to a named route (`redirect(url_for("home"))`) and take only an id from the client.
2. Or check the target: Django `url_has_allowed_host_and_scheme(url, allowed_hosts={request.get_host()})`; elsewhere require a path that starts with a single `/`.

## Examples

### Python

#### Don't

```py
def login(request):
    nxt = request.args.get("next")
    return redirect(nxt)
```

#### Do

```py
def login(request):
    target = SAFE_TARGETS.get(request.args.get("next"), "/")
    return redirect(target)
```
