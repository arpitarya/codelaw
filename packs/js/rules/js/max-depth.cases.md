# js/max-depth — cases

### case: a statement nested past the limit is reported on its first line

```js options={"max":2}
function f(a) {
  if (a) {
    while (a) {
      if (a) {
        a();
      }
    }
  }
}
```

```expect
4:7 js/max-depth
```

### case: at the limit is fine

```js options={"max":3}
function f(a) {
  if (a) {
    while (a) {
      if (a) {
        a();
      }
    }
  }
}
```

```expect
```

### case: else if adds no depth

```js options={"max":1}
function f(a) {
  if (a === 1) {
    a();
  } else if (a === 2) {
    a();
  } else if (a === 3) {
    a();
  }
}
```

```expect
```

### case: try, switch, and for-of all count

```js options={"max":2}
function f(xs) {
  try {
    for (const x of xs) {
      switch (x) {
        case 1:
          break;
      }
    }
  } catch (e) {}
}
```

```expect
4:7 js/max-depth
```

### case: a function starts again at zero

```js options={"max":1}
if (a) {
  items.forEach((x) => {
    if (x) {
      x();
    }
  });
}
```

```expect
```

### case: top-level code is measured too

```js options={"max":1}
if (a) {
  if (b) {
    c();
  }
}
```

```expect
2:3 js/max-depth
```
