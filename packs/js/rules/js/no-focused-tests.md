---
id: js/no-focused-tests
title: "No focused tests left in the suite"
languages: [javascript, jsx, typescript, tsx]
severity: error
fix: suggest
fix_reason: "dropping the focus puts every other test in the file back in the run, and tests that were hidden behind it may fail; the author should see that"
stability: preview
since: 0.5.0
---

A test file has no `.only` (`it.only`, `test.only`, `describe.only`, `test.describe.only`, `it.only.each(…)`) and no Jasmine `fit`, `fdescribe` or `ftest`. A call on something else that happens to have an `only` method is not a test and is not reported. The suggested fix drops the focus: `.only` is removed, `fit` becomes `it` and `fdescribe` becomes `describe`.

**Why:** A focused test makes the runner skip every other test in the run, and the run still passes. It is meant as a debugging aid for one minute on a laptop and it reaches the main branch as a suite that checks almost nothing.

```codelaw
native: js/no-focused-tests
```

## How to fix

1. Remove the focus: `it.only(…)` becomes `it(…)`, `fit` becomes `it`, `fdescribe` becomes `describe` (`codelaw fix --unsafe` does).
2. Run the whole suite once; tests that were hidden may now fail, and that is the point.

## Examples

### JavaScript

#### Don't

```js
describe("cart", () => {
  it.only("totals the items", () => {
    expect(total([1, 2])).toBe(3);
  });
});
```

#### Do

```js
describe("cart", () => {
  it("totals the items", () => {
    expect(total([1, 2])).toBe(3);
  });
});
```

### JSX

#### Don't

```jsx
test.only("renders the badge", () => {
  render(<Badge />);
});
```

#### Do

```jsx
test("renders the badge", () => {
  render(<Badge />);
});
```

### TypeScript

#### Don't

```ts
fdescribe("cart", () => {
  it("totals the items", () => expect(total([1, 2])).toBe(3));
});
```

#### Do

```ts
describe("cart", () => {
  it("totals the items", () => expect(total([1, 2])).toBe(3));
});
```

### TSX

#### Don't

```tsx
fit("renders the badge", () => {
  render(<Badge />);
});
```

#### Do

```tsx
it("renders the badge", () => {
  render(<Badge />);
});
```
