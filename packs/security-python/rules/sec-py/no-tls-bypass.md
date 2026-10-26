---
id: sec-py/no-tls-bypass
title: 'TLS certificate checks stay on'
languages: [python]
fix: suggest
fix_reason: "the server may use a private CA, which needs the CA bundle passed as `verify=` rather than the check removed"
stability: preview
ported_from: [bandit/B501, bandit/B502, bandit/B503, bandit/B504, bandit/B323, ruff/S501, ruff/S502, ruff/S503, ruff/S504, ruff/S323]
security: { cwe: [295], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

Certificate verification is not switched off (`verify=False`, `_create_unverified_context`), and an SSL context is not pinned to an old protocol.

**Why:** Without the check anyone on the network path can pose as the server and read or change the traffic (CWE-295).

```codelaw
match:
  any:
    - kind: call
      all:
        - has:
            field: function
            regex: '^(requests|httpx|session|client|s|self\.session|self\.client)\.(get|post|put|patch|delete|head|options|request|stream|send)$|^(requests|httpx)\.(Client|AsyncClient|Session)$'
        - has:
            field: arguments
            has:
              kind: keyword_argument
              all: [{ has: { field: name, regex: '^verify$' } }, { has: { field: value, kind: 'false' } }]
              pattern: $KW
    - kind: call
      all:
        - has: { field: function, regex: '^ssl\._create_unverified_context$|^ssl\.wrap_socket$' }
    - { kind: attribute, regex: '^ssl\.PROTOCOL_(SSLv2|SSLv3|SSLv23|TLSv1|TLSv1_1)$' }
    - { pattern: '$C.check_hostname = False' }
    - { pattern: '$C.verify_mode = ssl.CERT_NONE' }
message: 'TLS verification is off or pinned to an old protocol'
fix:
  - { remove: $KW }
```

## How to fix

1. Delete `verify=False`; if the server uses a private CA, pass its bundle: `verify="/etc/ssl/corp-ca.pem"`.
2. Use `ssl.create_default_context()` instead of `_create_unverified_context()`.
3. Never set `check_hostname = False` or `verify_mode = ssl.CERT_NONE`; set `minimum_version = ssl.TLSVersion.TLSv1_2` instead of an old `PROTOCOL_*`.

## Examples

### Python

#### Don't

```py
import requests

resp = requests.get(url, verify=False, timeout=5)
```

#### Do

```py
import requests

resp = requests.get(url, timeout=5)
```
