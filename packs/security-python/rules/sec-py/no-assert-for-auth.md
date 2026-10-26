---
id: sec-py/no-assert-for-auth
title: "`assert` is not an access check"
languages: [python]
exclude: ["**/tests/**", "**/test/**", "**/test_*.py", "**/*_test.py", "**/*_tests.py", "**/tests.py", "**/conftest.py"]
fix: manual
fix_reason: "the replacement raises the right exception for the framework, which only the author knows"
stability: preview
ported_from: [bandit/B101, ruff/S101]
security: { cwe: [703], owasp: ["A10:2025"], severity: medium }
since: 0.2.0
---

A permission, role or authentication test is not written as an `assert`.

**Why:** `python -O` removes every `assert`, so the check disappears in an optimised run (CWE-703).

```codelaw
match:
  all:
    - { kind: assert_statement }
    - regex: '(?i)(is_authenticated|is_admin|is_staff|is_superuser|has_perm|has_permission|permission|authorized|authenticated|is_owner|\.role\b|\.roles\b|current_user|user\.id\s*==)'
message: 'An `assert` is removed by `python -O`; raise an exception for an access check'
```

## How to fix

1. Replace it with `if not user.is_authenticated: raise PermissionDenied`.
2. Use your framework's decorator (`@login_required`, `Depends(...)`) where one fits.

## Examples

### Python

#### Don't

```py
def delete(request, item):
    assert request.user.is_staff
```

#### Do

```py
from django.core.exceptions import PermissionDenied


def delete(request, item):
    if not request.user.is_staff:
        raise PermissionDenied
```
