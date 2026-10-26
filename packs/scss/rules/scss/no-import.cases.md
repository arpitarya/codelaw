# Cases: scss/no-import

### case: a plain CSS import stays

```scss
@import url("a.css");
@import "b.css";
@import "c" screen;
```

```expect
(none)
```

### case: a url import stays

```scss
@import "https://example.com/a";
```

```expect
(none)
```

### case: a single Sass import

```scss
@import "variables";
```

```expect
1:1 scss/no-import
```

### case: several imports in one statement

```scss
@import "a", "b";
```

```expect
1:1 scss/no-import
```

### case: a nested import has no `@use` form

```scss
.a {
  @import "inner";
}
```

```expect
(none)
```
