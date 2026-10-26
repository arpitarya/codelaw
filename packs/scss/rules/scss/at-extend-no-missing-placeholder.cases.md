# Cases: scss/at-extend-no-missing-placeholder

### case: a placeholder

```scss
.a {
  @extend %b;
}
```

```expect
(none)
```

### case: an element

```scss
.a {
  @extend a;
}
```

```expect
2:11 scss/at-extend-no-missing-placeholder
```

### case: an optional class

```scss
.a {
  @extend .b !optional;
}
```

```expect
2:11 scss/at-extend-no-missing-placeholder
```

### case: interpolation

```scss
.a {
  @extend #{$sel};
}
```

```expect
(none)
```
