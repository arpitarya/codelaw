---
id: py/assert-raises-exception
title: "Tests don't expect a bare `Exception`"
languages: [python]
fix: manual
fix_reason: "the exception the test expects has to be named by its author"
stability: preview
ported_from: [ruff/B017]
since: 0.1.0
---

`assertRaises(Exception)` and `pytest.raises(Exception)` (without `match=`) aren't used as context managers.

**Why:** They pass for any error at all, including a typo in the code under test.

```codelaw
native: py/assert-raises-exception
```

## How to fix

1. Name the specific exception the code should raise.
2. With `pytest.raises`, add `match=` if the class can't be narrowed.

## Examples

### Python

#### Don't

```py
import unittest


class TestParse(unittest.TestCase):
    def test_bad_input(self):
        with self.assertRaises(Exception):
            parse("x")
```

#### Do

```py
import unittest


class TestParse(unittest.TestCase):
    def test_bad_input(self):
        with self.assertRaises(ValueError):
            parse("x")
```
