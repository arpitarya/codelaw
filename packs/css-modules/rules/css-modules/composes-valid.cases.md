# Cases: css-modules/composes-valid

### case: a local class

```css path=src/a.module.css
.a { composes: b; }
.b { color: red; }
```

```expect
(none)
```

### case: a global class and a package path are not judged

```css path=src/a.module.css
.a { composes: x from global; composes: y from "lib/y.css"; }
```

```expect
(none)
```

### case: a missing local class

```css path=src/a.module.css
.a { composes: gone; }
```

```expect
1:16 css-modules/composes-valid
```

### case: a selector list

```css path=src/a.module.css
.a, .b { composes: c; }
.c { }
```

```expect
1:10 css-modules/composes-valid
```

### case: a nested rule

```css path=src/a.module.css
.a { .b { composes: a; } }
```

```expect
1:11 css-modules/composes-valid
```

### case: a rule in an at-rule

```css path=src/a.module.css
@media print { .a { composes: b; } .b { } }
```

```expect
1:21 css-modules/composes-valid
```
