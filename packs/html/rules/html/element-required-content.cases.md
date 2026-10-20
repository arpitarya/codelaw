# html/element-required-content — cases

### case: an implied head does not count

```html
<html lang="en">
  <body></body>
</html>
```

```expect
1:1 html/element-required-content
```

### case: a fragment head is checked too

```html
<head></head>
```

```expect
1:1 html/element-required-content
```
