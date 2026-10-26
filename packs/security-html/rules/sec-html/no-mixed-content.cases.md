# sec-html/no-mixed-content — cases

### case: loopback is trusted

```html
<img src="http://localhost:3000/a.png" alt="">
```

```expect
(none)
```

### case: a stylesheet link is fixed

```html
<link rel="stylesheet" href="http://a.example/s.css">
```

```expect
1:24 sec-html/no-mixed-content
```

```fixed
<link rel="stylesheet" href="https://a.example/s.css">
```

### case: a canonical link is only a pointer

```html
<link rel="canonical" href="http://a.example/">
```

```expect
(none)
```

### case: a plain link is navigation, a download is not

```html
<a href="http://a.example/x">a</a>
<a href="http://a.example/x.pdf" download>b</a>
```

```expect
2:4 sec-html/no-mixed-content
```

```fixed
<a href="http://a.example/x">a</a>
<a href="https://a.example/x.pdf" download>b</a>
```

### case: a form action

```html
<form action="http://a.example/p"></form>
```

```expect
1:7 sec-html/no-mixed-content
```

```fixed
<form action="https://a.example/p"></form>
```

### case: the scheme is read in any case

```html
<img src="HTTP://a.example/p.png" alt="">
```

```expect
1:6 sec-html/no-mixed-content
```

```fixed
<img src="https://a.example/p.png" alt="">
```
