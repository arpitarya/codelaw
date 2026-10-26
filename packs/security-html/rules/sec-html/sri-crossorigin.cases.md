# sec-html/sri-crossorigin — cases

### case: a stylesheet

```html
<link rel="stylesheet" href="https://cdn.example.com/a.css" integrity="sha384-abc">
```

```expect
1:61 sec-html/sri-crossorigin
```

```fixed
<link rel="stylesheet" href="https://cdn.example.com/a.css" integrity="sha384-abc" crossorigin="anonymous">
```

### case: a file on this origin

```html
<script src="/a.js" integrity="sha384-abc"></script>
```

```expect
(none)
```

### case: crossorigin already there

```html
<script src="https://cdn.example.com/a.js" integrity="sha384-abc" crossorigin></script>
```

```expect
(none)
```
