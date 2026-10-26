# sec-html/csp-meta-unsafe — cases

### case: unsafe-inline and unsafe-eval

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-inline' 'unsafe-eval'">
```

```expect
1:44 sec-html/csp-meta-unsafe
```

### case: default-src applies when there is no script-src

```html
<meta http-equiv="content-security-policy" content="default-src *">
```

```expect
1:44 sec-html/csp-meta-unsafe
```

### case: data: as a script source

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'self' data:">
```

```expect
1:44 sec-html/csp-meta-unsafe
```

### case: unsafe-inline in style-src is another rule

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'self'; style-src 'unsafe-inline'">
```

```expect
(none)
```

### case: a nonce makes the browser ignore unsafe-inline

```html
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-abc' 'unsafe-inline'">
```

```expect
(none)
```
