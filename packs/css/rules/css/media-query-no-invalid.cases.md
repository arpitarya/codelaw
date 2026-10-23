# Cases: css/media-query-no-invalid

### case: level 4 ranges and nesting

```css
@media (400px <= width <= 700px) and ((hover) or (pointer: fine)) {
  a {
    color: red;
  }
}
```

```expect
(none)
```

### case: mixed and/or

```css
@media (color) and (hover) or (width) {
  a {
    color: red;
  }
}
```

```expect
1:8 css/media-query-no-invalid
```

### case: a media type with not

```css
@media not print and (color) {
  a {
    color: red;
  }
}
```

```expect
(none)
```

### case: not without a space

`not(` is a function token, not the keyword. `codelaw format` writes `not (`, which turns this
query valid; the finding comes first because lint runs before format (SR-FORMATTER decision 4).

```css
@media not(min-width: 600px) {
  a {
    color: red;
  }
}
```

```expect
1:8 css/media-query-no-invalid
```
