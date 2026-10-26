# Cases: scss/at-mixin-pattern

### case: an underscore

```scss
@mixin button_base($a) {
  color: $a;
}
```

```expect
1:8 scss/at-mixin-pattern
```

### case: an include is not checked

```scss
.a {
  @include Button_Base;
}
```

```expect
(none)
```
