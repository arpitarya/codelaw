---
id: js/skipped-test
title: "A skipped test has a reason and an expiry"
languages: [javascript, jsx, typescript, tsx]
severity: error
fix: manual
fix_reason: "why a test is off, and until when, is a fact only the author knows; the engine can't write it"
stability: preview
since: 0.5.0
---

A test turned off with `.skip` (`it.skip`, `test.skip`, `describe.skip`, `test.describe.skip`, `it.skip.each(…)`) or with Jasmine's and Mocha's `xit`, `xdescribe`, `xtest`, `xcontext`, `xspecify` says why and until when. Playwright's `test.skip(condition, "reason")` already carries its reason in the call; the expiry still goes in a `SKIP` comment.

The reason comes from the framework's own argument where it has one, else from the text of a **`SKIP` comment** on the line directly above the test (or at the end of its first line): `// SKIP(ABC-123, 2027-01-31): reason`. The ticket and the date are in the parentheses, in either order. The expiry is the date the skip stops being allowed: it is **expired** from the as-of date on (`--as-of`, `CODELAW_AS_OF`, else `HEAD`'s commit date, never the clock: [SR-BASELINE](../../../../records/0104_baseline-and-deprecations.md) decision 9) and is a **warning** for the `warn_days` before it. A skip whose `SKIP` comment names a date that has come is an error at the skipped test.

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
native: js/skipped-test
```

## How to fix

1. Write the reason, the ticket that tracks the fix and the date you will look again, on the line above: `// SKIP(ABC-123, 2027-01-31): reason`.
2. For an expired skip: fix the test and remove the skip; or, if it really must stay off, move the date and say why in the reason.
3. If the test is not needed, delete it.

## Examples

### JavaScript

#### Don't

```js
it.skip("totals the items", () => {
  expect(total([1, 2])).toBe(3);
});
```

#### Do

```js
// SKIP(SHOP-77, 2027-01-31): flaky on the CI clock
it.skip("totals the items", () => {
  expect(total([1, 2])).toBe(3);
});
```

### JSX

#### Don't

```jsx
xtest("renders the badge", () => {
  render(<Badge />);
});
```

#### Do

```jsx
// SKIP(UI-8, 2027-02-15): waits for the new icon font
xtest("renders the badge", () => {
  render(<Badge />);
});
```

### TypeScript

#### Don't

```ts
describe.skip("cart", () => {
  it("totals the items", () => expect(total([1, 2])).toBe(3));
});
```

#### Do

```ts
// SKIP(SHOP-90, 2027-03-01): rewritten in the checkout rework
describe.skip("cart", () => {
  it("totals the items", () => expect(total([1, 2])).toBe(3));
});
```

### TSX

#### Don't

```tsx
test.skip("renders the badge", () => {
  render(<Badge />);
});
```

#### Do

```tsx
// SKIP(UI-8, 2027-02-15): waits for the new icon font
test.skip("renders the badge", () => {
  render(<Badge />);
});
```
