# Cases: scss/no-slash-division

### case: a CSS slash

```scss
.a {
  font: 12px/1.5 Arial;
  grid-area: 1 / 2 / 3;
  aspect-ratio: 16 / 9;
}
```

```expect
(none)
```

### case: calc keeps the slash

```scss
.a {
  width: calc(100% / 3);
  height: calc($a / 2);
}
```

```expect
(none)
```

### case: a variable divided

```scss
.a {
  width: $a/2;
}
```

```expect
2:12 scss/no-slash-division
```

```fixed
@use "sass:math";

.a {
  width: math.div($a, 2);
}
```

### case: numbers in parentheses

```scss
.a {
  width: (10px / 2);
}
```

```expect
2:16 scss/no-slash-division
```

### case: a return value

```scss
@function half($n) {
  @return $n / 2;
}
```

```expect
2:14 scss/no-slash-division
```

### case: math.div is fine

```scss
@use "sass:math";
.a {
  width: math.div($a, 2);
}
```

```expect
(none)
```
