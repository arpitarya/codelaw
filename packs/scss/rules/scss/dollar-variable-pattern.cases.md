# Cases: scss/dollar-variable-pattern

### case: snake case

```scss
$font_size: 16px;
```

```expect
1:1 scss/dollar-variable-pattern
```

### case: a number inside

```scss
$h2-size: 1px;
```

```expect
(none)
```

### case: a module variable

```scss
@use "x";
x.$Big: 1;
```

```expect
(none)
```
