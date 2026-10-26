---
id: sec-py/hf-pin-revision
title: 'Hugging Face downloads pin a revision'
languages: [python]
fix: manual
fix_reason: "the commit hash to pin is a choice the author makes after reviewing the model"
stability: preview
ported_from: [bandit/B615]
security: { cwe: [494], owasp: ["A03:2025"], severity: medium }
since: 0.2.0
---

`from_pretrained`, `hf_hub_download`, `snapshot_download` and `load_dataset` are called with `revision=`.

**Why:** Without a revision the code runs whatever the repository's `main` branch holds today, so a change there reaches production unreviewed (CWE-494).

```codelaw
match:
  kind: call
  all:
    - has:
        field: function
        regex: '^((.*\.)?(from_pretrained|hf_hub_download|snapshot_download)|(datasets\.)?load_dataset)$'
    - not:
        has:
          field: arguments
          has:
            kind: keyword_argument
            all: [{ has: { field: name, regex: '^revision$' } }]
message: 'This download follows `main`; pin `revision=` to a commit hash'
```

## How to fix

1. Find the commit hash of the revision you reviewed on the model's page.
2. Pass it: `from_pretrained(name, revision="<hash>")`.
3. Keep the hash in one constant or setting so an upgrade is one reviewed change.

## Examples

### Python

#### Don't

```py
from transformers import AutoModel

model = AutoModel.from_pretrained("org/model")
```

#### Do

```py
from transformers import AutoModel

MODEL_REVISION = "5c9e4d1"
model = AutoModel.from_pretrained("org/model", revision=MODEL_REVISION)
```
