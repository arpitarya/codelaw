# Cases: scss/at-rule-no-unknown

### case: Sass at-rules are known

```scss
@use "a";
@mixin m($a: 1) {
  @content;
}
@function f($a) {
  @return $a;
}
.a {
  @include m;
  @extend %p;
  @if $x {
    a: b;
  } @else {
    c: d;
  }
}
```

```expect
(none)
```

### case: a misspelt CSS at-rule

```scss
@media-query screen {
  a {
    color: red;
  }
}
```

```expect
1:1 scss/at-rule-no-unknown
```
