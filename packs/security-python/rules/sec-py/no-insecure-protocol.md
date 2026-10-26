---
id: sec-py/no-insecure-protocol
title: 'No Telnet, FTP or SNMP v1/v2'
languages: [python]
fix: manual
fix_reason: "the replacement (SSH or SFTP, FTPS, SNMPv3) needs the server to support it"
stability: preview
ported_from: [bandit/B312, bandit/B321, bandit/B401, bandit/B402, bandit/B508, bandit/B509, ruff/S401, ruff/S402, ruff/S312, ruff/S321, ruff/S508, ruff/S509]
security: { cwe: [319], owasp: ["A04:2025"], severity: high }
since: 0.2.0
---

`telnetlib`, `ftplib` and SNMP community strings (v1/v2c) are not used.

**Why:** These send credentials and data in clear text, so anyone on the path reads and changes them (CWE-319).

```codelaw
match:
  any:
    - all: [{ kind: import_statement }, { has: { field: name, regex: '^(telnetlib|ftplib)$' } }]
    - all:
        - { kind: import_from_statement }
        - { has: { field: module_name, regex: '^(telnetlib|ftplib)$' } }
    - kind: call
      all: [{ has: { field: function, regex: '^(pysnmp\.hlapi\.)?CommunityData$' } }]
message: 'This protocol sends credentials in clear text'
```

## How to fix

1. Use SSH or SFTP (`paramiko`, `asyncssh`) instead of Telnet and FTP, or `ftplib.FTP_TLS` for FTPS.
2. Use SNMPv3 (`UsmUserData` with authentication and privacy) instead of a community string.

## Examples

### Python

#### Don't

```py
import ftplib

ftp = ftplib.FTP(host)
```

#### Do

```py
import paramiko

client = paramiko.SSHClient()
```
