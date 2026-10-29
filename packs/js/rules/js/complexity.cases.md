# js/complexity — cases

### case: a function over the limit is reported on its name

```js options={"max":2}
function f(a, b) {
  if (a) return 1;
  if (b) return 2;
  return 3;
}
```

```expect
1:10 js/complexity
```

### case: at the limit is fine

```js options={"max":3}
function f(a, b) {
  if (a) return 1;
  if (b) return 2;
  return 3;
}
```

```expect
```

### case: logical operators and the conditional add branches

```js options={"max":3}
const f = (a, b) => (a && b ? 1 : a || b);
```

```expect
1:7 js/complexity
```

### case: switch cases count, default does not

```js options={"max":2}
function f(a) {
  switch (a) {
    case 1:
      return 1;
    case 2:
      return 2;
    default:
      return 3;
  }
}
```

```expect
1:10 js/complexity
```

### case: loops and catch

```js options={"max":3}
function f(xs) {
  for (const x of xs) {
    while (x) {
      try {
        x();
      } catch (e) {
        x = null;
      }
    }
  }
}
```

```expect
1:10 js/complexity
```

### case: an inner function is counted on its own

```js options={"max":2}
function outer(a) {
  const inner = () => {
    if (a) return 1;
    if (!a) return 2;
    return 3;
  };
  return inner;
}
```

```expect
2:9 js/complexity
```

### case: optional chains do not count by default

```js options={"max":1}
function f(a) {
  return a?.b?.c;
}
```

```expect
```

### case: optional chains count on request

```js options={"max":1,"count_optional_chain":true}
function f(a) {
  return a?.b?.c;
}
```

```expect
1:10 js/complexity
```

### case: ignore_names exempts a function

```js options={"max":1,"ignore_names":"^route"}
function routeRequest(a) {
  if (a) return 1;
  return 2;
}
```

```expect
```
