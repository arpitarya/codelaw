# Cases: scss/load-no-partial-leading-underscore

### case: a clean path

```scss
@use "base/colors";
@forward "sass:math";
```

```expect
(none)
```

### case: a forward

```scss
@forward "_index";
```

```expect
1:11 scss/load-no-partial-leading-underscore
```

```fixed
@forward "index";
```

### case: a css import keeps its name

```scss
@import "_x.css";
```

```expect
(none)
```

### case: an underscore elsewhere

```scss
@use "a_b/c_d";
```

```expect
(none)
```
