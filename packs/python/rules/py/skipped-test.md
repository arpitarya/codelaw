---
id: py/skipped-test
title: "A skipped test has a reason and an expiry"
languages: [python]
severity: error
fix: manual
fix_reason: "why a test is off, and until when, is a fact only the author knows; the engine can't write it"
stability: preview
since: 0.5.0
---

A test turned off with `@pytest.mark.skip`, `@pytest.mark.skipif`, `@pytest.mark.xfail`, `@unittest.skip`, `@unittest.skipIf` or `@unittest.skipUnless` (also as `pytestmark = pytest.mark.skip(…)`) says why and until when.

The reason comes from the framework's own argument where it has one, else from the text of a **`SKIP` comment** on the line directly above the test (or at the end of its first line): `# SKIP(ABC-123, 2027-01-31): reason`. The ticket and the date are in the parentheses, in either order. The expiry is the date the skip stops being allowed: it is **expired** from the as-of date on (`--as-of`, `CODELAW_AS_OF`, else `HEAD`'s commit date, never the clock: [SR-BASELINE](../../../../records/0104_baseline-and-deprecations.md) decision 9) and is a **warning** for the `warn_days` before it. pytest's `reason=` keyword and unittest's reason argument count as the reason; the expiry still goes in a `SKIP` comment above the decorators. A comment in `#` form is the same as the `//` form shown here.

**Why:** A skipped test is a check that is switched off, and nothing shows it. A reason says why, an expiry makes it come back for review on a day somebody agreed to, and a failing skip on that day is how it gets turned on again instead of staying off for years.

**Configuration** is the `[tests.skip]` table of `codelaw.toml` ([SR-CONFIG](../../../../records/0110_config.md) decision 11a), shared by `js/skipped-test` and `py/skipped-test`:

```toml
[tests.skip]
require_reason = true    # a reason is required (default)
require_expiry = true    # a SKIP(…, date) expiry is required (default)
require_ticket = false   # a ticket is required (default: no)
ticket_pattern = "[A-Z][A-Z0-9]+-\\d+"
warn_days      = 14      # warn this many days before the expiry
```

```codelaw
native: py/skipped-test
```

## How to fix

1. Write the reason, the ticket that tracks the fix and the date you will look again, on the line above: `# SKIP(ABC-123, 2027-01-31): reason`.
2. For an expired skip: fix the test and remove the skip; or, if it really must stay off, move the date and say why in the reason.
3. If the test is not needed, delete it.

## Examples

### Python

#### Don't

```py
@pytest.mark.skip(reason="flaky on CI")
def test_total():
    assert total([1, 2]) == 3
```

#### Do

```py
# SKIP(SHOP-77, 2027-01-31): flaky on the CI clock
@pytest.mark.skip(reason="flaky on CI")
def test_total():
    assert total([1, 2]) == 3
```
