# acme/no-nested-ternary — cases

Written fresh from the rule's documented behaviour; none is copied from ESLint's tests.

### case: nested in the ? branch

```js
const v = ready ? (fast ? 1 : 2) : 3;
```

```expect
1:11 acme/no-nested-ternary
```

### case: three levels report the two outer ones

```js
const v = a ? 1 : b ? 2 : c ? 3 : 4;
```

```expect
1:11 acme/no-nested-ternary
1:19 acme/no-nested-ternary
```

### case: a single conditional expression is fine

```js
const v = ready ? 1 : 2;
```

```expect
(none)
```

### case: a conditional in the condition is not reported

```js
const v = (a ? b : c) ? 1 : 2;
```

```expect
(none)
```

### case: a conditional inside a function in a branch is its own scope

```js
const pick = ready ? () => (fast ? 1 : 2) : null;
```

```expect
(none)
```

### case: two separate conditionals side by side

```js
const v = [a ? 1 : 2, b ? 3 : 4];
```

```expect
(none)
```
