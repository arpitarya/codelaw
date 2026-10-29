# js/no-focused-tests — cases

### case: every focused form

```js
describe.only("a", () => {});
it.only("b", () => {});
test.describe.only("c", () => {});
it.only.each([1])("d", () => {});
fit("e", () => {});
fdescribe("f", () => {});
```

```expect
1:1 js/no-focused-tests
2:1 js/no-focused-tests
3:1 js/no-focused-tests
4:1 js/no-focused-tests
5:1 js/no-focused-tests
6:1 js/no-focused-tests
```

### case: something else with an only method

```js
queue.only(1);
const fitness = fit;
it("ok", () => {});
```

```expect
(none)
```
