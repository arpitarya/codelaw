# sec-html/no-sensitive-comments — cases

### case: a comment on its own line goes with the line

```html
<p>a</p>
  <!-- password: hunter2 -->
<p>b</p>

```

```expect
2:3 sec-html/no-sensitive-comments
```

```fixed
<p>a</p>
<p>b</p>

```

### case: a comment after other markup

```html
<p>a</p><!-- TODO remove this -->
```

```expect
1:9 sec-html/no-sensitive-comments
```

```fixed
<p>a</p>
```

### case: a private address

```html
<!-- the API is at 10.0.3.7 -->
```

```expect
1:1 sec-html/no-sensitive-comments
```

### case: an ordinary comment

```html
<!-- navigation -->
```

```expect
(none)
```

### case: a conditional comment is not a note

```html
<!--[if IE]><p>password: x</p><![endif]-->
```

```expect
(none)
```
