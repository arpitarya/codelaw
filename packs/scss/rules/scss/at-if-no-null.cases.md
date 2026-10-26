# Cases: scss/at-if-no-null

### case: not equal

```scss
@if $x != null {
  a: b;
}
```

```expect
1:5 scss/at-if-no-null
```

```fixed
@if $x {
  a: b;
}
```

### case: an else if

```scss
@if $a {
  a: b;
} @else if $x != null {
  c: d;
}
```

```expect
3:12 scss/at-if-no-null
```

```fixed
@if $a {
  a: b;
} @else if $x {
  c: d;
}
```

### case: equal is reported without a fix

```scss
@if $x == null {
  a: b;
}
```

```expect
1:5 scss/at-if-no-null
```

### case: part of a condition is reported without a fix

```scss
@if $x != null and $y {
  a: b;
}
```

```expect
1:5 scss/at-if-no-null
```

### case: no null

```scss
@if $x == 1 {
  a: b;
}
```

```expect
(none)
```
