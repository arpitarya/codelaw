---
id: sec-py/no-unsafe-deserialize
title: 'Deserialising with `pickle` and its relatives'
languages: [python]
fix: manual
fix_reason: "the replacement format (JSON, a schema, a signed payload) is a design choice"
stability: preview
ported_from: [bandit/B301, bandit/B302, bandit/B403]
security: { cwe: [502], owasp: ["A08:2025"], severity: high }
since: 0.2.0
---

Data is not loaded with `pickle`, `cPickle`, `dill`, `marshal`, `shelve` or `jsonpickle`.

**Why:** Loading runs code chosen by whoever wrote the bytes, so untrusted input is remote code execution (CWE-502).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^((c|_)?[Pp]ickle|dill|cloudpickle)\.(loads?|Unpickler)$|^marshal\.loads?$|^shelve\.open$|^jsonpickle\.(decode|loads)$'
message: 'This loads data with a format that runs code on load'
```

## How to fix

1. Use JSON (`json.loads`) or another data-only format, and validate the result against a schema.
2. If the bytes must be a pickle, sign them (`hmac`) and verify the signature before loading, or load only files the program itself wrote.
3. For model weights use `safetensors`.

## Examples

### Python

#### Don't

```py
import pickle

obj = pickle.loads(payload)
```

#### Do

```py
import json

obj = json.loads(payload)
```
