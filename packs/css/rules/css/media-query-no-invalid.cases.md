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
