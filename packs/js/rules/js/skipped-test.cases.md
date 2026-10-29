# js/skipped-test — cases

### case: a reason and an expiry in the comment above

```js
// SKIP(ABC-1, 2027-06-01): flaky on CI
it.skip("a", () => {});
```

```expect
(none)
```

### case: a comment at the end of the line

```js
xit("a", () => {}); // SKIP(2027-06-01): waits for the API
```

```expect
(none)
```

### case: the comment has an expiry and no reason

```js
// SKIP(ABC-1, 2027-06-01)
it.skip("a", () => {});
```

```expect
2:1 js/skipped-test
```

### case: the comment is not directly above

```js
// SKIP(ABC-1, 2027-06-01): flaky on CI

it.skip("a", () => {});
```

```expect
3:1 js/skipped-test
```

### case: a skip inside a skipped suite is its own

```js
// SKIP(ABC-1, 2027-06-01): rewrite
describe.skip("a", () => {
  it.skip("b", () => {});
});
```

```expect
3:3 js/skipped-test
```

### case: Playwright's reason argument

```ts options={"require_expiry":false}
test.skip(process.env.CI === "true", "no GPU on CI");
```

```expect
(none)
```

### case: nothing required

```js options={"require_reason":false,"require_expiry":false}
it.skip("a", () => {});
```

```expect
(none)
```

### case: a ticket is required

```js options={"require_ticket":true}
// SKIP(2027-06-01): flaky
it.skip("a", () => {});
```

```expect
2:1 js/skipped-test
```
