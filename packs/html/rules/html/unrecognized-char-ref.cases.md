# html/unrecognized-char-ref — cases

### case: numeric references are not looked up

```html
<p>&#169; &#xA9;</p>
```

```expect
(none)
```

### case: query strings in URLs may hold unterminated names

```html
<a href="/s?q=1&copy=2">x</a>
```

```expect
(none)
```

### case: a legacy name without its semicolon

```html
<p>&copy 2024</p>
```

```expect
1:4 html/unrecognized-char-ref
```

### case: an unknown name

```html
<p>&nbps;</p>
```

```expect
1:4 html/unrecognized-char-ref
```
