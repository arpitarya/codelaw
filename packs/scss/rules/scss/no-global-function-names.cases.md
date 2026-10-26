# Cases: scss/no-global-function-names

### case: a module call is fine

```scss
@use "sass:map";
.a {
  width: map.get($sizes, small);
}
```

```expect
(none)
```

### case: two calls add the `@use` once

```scss
.a {
  width: map-get($s, a);
  height: map-get($s, b);
}
```

```expect
2:10 scss/no-global-function-names
3:11 scss/no-global-function-names
```

```fixed
@use "sass:map";

.a {
  width: map.get($s, a);
  height: map.get($s, b);
}
```

### case: an existing `@use` is kept

```scss
@use "sass:map";
.a {
  width: map-get($s, a);
}
```

```expect
3:10 scss/no-global-function-names
```

```fixed
@use "sass:map";
.a {
  width: map.get($s, a);
}
```

### case: two modules

```scss
.a {
  width: percentage(0.5);
  height: nth($l, 1);
}
```

```expect
2:10 scss/no-global-function-names
3:11 scss/no-global-function-names
```

### case: a CSS function with a Sass name is left alone

```scss
.a {
  filter: grayscale(50%);
  width: min(1px, 2px);
}
```

```expect
(none)
```

### case: a function of the file's own

```scss
@function nth($a) {
  @return $a;
}
.a {
  width: nth(1);
}
```

```expect
(none)
```

### case: a colour adjuster is reported without a fix

```scss
.a {
  color: darken($c, 10%);
}
```

```expect
2:10 scss/no-global-function-names
```

### case: a name in a string or a variable

```scss
$map-get: 1;
.a {
  content: "map-get(x)";
  width: $map-get;
}
```

```expect
(none)
```
