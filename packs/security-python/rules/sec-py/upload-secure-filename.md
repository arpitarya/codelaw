---
id: sec-py/upload-secure-filename
title: 'Uploaded files are saved under a safe name'
languages: [python]
fix: suggest
fix_reason: "`secure_filename` removes characters a name may legitimately have, and returns an empty string for some names, which needs handling"
stability: preview
security: { cwe: [434, 22], owasp: ["A01:2025"], severity: high }
since: 0.2.0
---

A file from a request is not saved under the name the client gave it.

**Why:** The name can be `../../app.py` or an absolute path, so the upload overwrites a file outside the upload folder (CWE-434, CWE-22).

```codelaw
match:
  all:
    - { kind: call }
    - { has: { field: function, regex: '\.save$' } }
    - has:
        field: arguments
        has:
          any:
            - { kind: attribute, regex: '\.filename$', pattern: $NAME }
            - kind: call
              has: { field: arguments, has: { kind: attribute, regex: '\.filename$', pattern: $NAME } }
message: 'The upload is saved under the client''s file name; wrap it in `secure_filename`'
fix:
  - { replace: { target: $NAME, with: 'secure_filename($NAME)' } }
```

## How to fix

1. Import `from werkzeug.utils import secure_filename`.
2. Use `secure_filename(file.filename)`, and refuse an empty result.
3. Better: store the file under a name you generate (a UUID) and keep the client's name as data.

## Examples

### Python

#### Don't

```py
from werkzeug.utils import secure_filename

file.save(os.path.join(UPLOADS, file.filename))
```

#### Do

```py
from werkzeug.utils import secure_filename

file.save(os.path.join(UPLOADS, secure_filename(file.filename)))
```

#### Fixed

```py
from werkzeug.utils import secure_filename

file.save(os.path.join(UPLOADS, secure_filename(file.filename)))
```
