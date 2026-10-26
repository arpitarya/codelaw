# Cases: scss/no-duplicate-mixins

### case: different scopes

```scss
@mixin a {
  b: c;
}
@media print {
  @mixin a {
    b: d;
  }
}
```

```expect
(none)
```

### case: a duplicate with arguments

```scss
@mixin a($x) {
  b: $x;
}
@mixin a($x, $y) {
  b: $y;
}
```

```expect
4:8 scss/no-duplicate-mixins
```
