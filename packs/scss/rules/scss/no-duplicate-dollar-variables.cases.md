# Cases: scss/no-duplicate-dollar-variables

### case: different scopes

```scss
$a: 1;
.b {
  $a: 2;
}
```

```expect
(none)
```

### case: a default

```scss
$a: 1;
$a: 2 !default;
```

```expect
(none)
```

### case: a duplicate in a block

```scss
.b {
  $a: 1;
  $a: 2;
}
```

```expect
3:3 scss/no-duplicate-dollar-variables
```

### case: a module variable is not a declaration here

```scss
@use "x";
x.$a: 1;
```

```expect
(none)
```
