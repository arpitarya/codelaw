# Cases: scss/function-no-unknown

### case: css and sass functions

```scss
@use "sass:map";
.a {
  width: calc(1px + 2px);
  height: map.get($m, k);
  color: rgb(0 0 0 / 50%);
  top: darken($c, 5%);
}
```

```expect
(none)
```

### case: the file's own function

```scss
@function double($n) {
  @return $n * 2;
}
.a {
  width: double(2px);
}
```

```expect
(none)
```

### case: a misspelt function

```scss
.a {
  width: calcc(1px);
}
```

```expect
2:10 scss/function-no-unknown
```

### case: a vendor prefix

```scss
.a {
  background: -webkit-linear-gradient(red, blue);
}
```

```expect
(none)
```

### case: an import hides names

```scss
@import "lib";
.a {
  width: mystery(1);
}
```

```expect
(none)
```
