---
id: acme/no-datetime-now
title: "Read the time through acme.clock.now()"
languages: [python]
fix: suggest
fix_reason: "the call is replaced, but the `from acme.clock import now` import is the author's to add"
stability: preview
since: 0.2.0
---

Code reads the current time only through `acme.clock.now()`, never `datetime.now()`,
`datetime.utcnow()` or `datetime.today()`.

**Why:** the clock is the one seam tests freeze; a direct call makes the code untestable and
mixes naive and aware datetimes.

```codelaw
match:
  any:
    - pattern: datetime.now($$$A)
    - pattern: datetime.utcnow($$$A)
    - pattern: datetime.today($$$A)
    - pattern: datetime.datetime.now($$$A)
    - pattern: datetime.datetime.utcnow($$$A)
message: "Read the time through `acme.clock.now()` (from acme.clock import now)"
fix: "now()"
```

## Examples

### Python

#### Don't

```python
stamp = datetime.now()
```

#### Do

```python
stamp = now()
```
