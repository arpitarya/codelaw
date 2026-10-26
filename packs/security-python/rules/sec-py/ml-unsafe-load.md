---
id: sec-py/ml-unsafe-load
title: 'Model and array files are not unpickled'
languages: [python]
fix: manual
fix_reason: "the replacement format (`safetensors`, `.npy` without pickle, Parquet) means converting the stored files"
stability: preview
security: { cwe: [502], owasp: ["A08:2025"], severity: high }
since: 0.2.0
---

`numpy.load(allow_pickle=True)`, `joblib.load`, `pandas.read_pickle` and Keras `load_model(safe_mode=False)` are not used.

**Why:** Each one unpickles the file, so a model or dataset from anywhere else runs code on load (CWE-502).

```codelaw
match:
  any:
    - kind: call
      all:
        - { has: { field: function, regex: '^(np|numpy)\.load$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^allow_pickle$' } }
                - { has: { field: value, kind: 'true' } }
    - kind: call
      all: [{ has: { field: function, regex: '^joblib\.load$' } }]
    - kind: call
      all: [{ has: { field: function, regex: '^(pd|pandas)\.read_pickle$' } }]
    - kind: call
      all:
        - { has: { field: function, regex: '(^|\.)load_model$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all: [{ has: { field: name, regex: '^safe_mode$' } }, { has: { field: value, kind: 'false' } }]
message: 'This loads a pickle, which can run code; use `safetensors` or a data-only format'
```

## How to fix

1. Save and load arrays with `np.save` / `np.load` without `allow_pickle`, and tensors with `safetensors`.
2. Store tables as Parquet or CSV, not pickle.
3. If the file must be a pickle, load only files your own pipeline wrote and check a signature first.

## Examples

### Python

#### Don't

```py
import numpy as np

arr = np.load(path, allow_pickle=True)
```

#### Do

```py
import numpy as np

arr = np.load(path)
```
