---
id: sec-py/torch-load-weights-only
title: "`torch.load` with `weights_only=True`"
languages: [python]
fix: suggest
fix_reason: "`weights_only=True` refuses checkpoints that hold arbitrary objects; those need a conversion to `safetensors`"
stability: preview
ported_from: [bandit/B614]
security: { cwe: [502], owasp: ["A08:2025"], severity: high }
since: 0.2.0
---

`torch.load` is called with `weights_only=True`.

**Why:** Without it the file is unpickled, so a model file from anywhere else runs code on load (CWE-502).

```codelaw
match:
  any:
    - all:
        - { pattern: 'torch.load($$$A)' }
        - not:
            has:
              field: arguments
              has:
                kind: keyword_argument
                all: [{ has: { field: name, regex: '^weights_only$' } }]
    - kind: call
      all:
        - { has: { field: function, regex: '^torch\.load$' } }
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all:
                - { has: { field: name, regex: '^weights_only$' } }
                - { has: { field: value, kind: 'false' } }
message: '`torch.load` unpickles the file; pass `weights_only=True`'
fix: 'torch.load($$$A, weights_only=True)'
```

## How to fix

1. Add `weights_only=True`.
2. If the checkpoint holds more than tensors, convert it once with a trusted environment and keep it as `safetensors`.

## Examples

### Python

#### Don't

```py
import torch

model = torch.load(path)
```

#### Do

```py
import torch

model = torch.load(path, weights_only=True)
```

#### Fixed

```py
import torch

model = torch.load(path, weights_only=True)
```
