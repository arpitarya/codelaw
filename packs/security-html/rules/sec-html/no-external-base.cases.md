# sec-html/no-external-base — cases

### case: a protocol-relative base

```html
<base href="//cdn.example.com/">
```

```expect
1:7 sec-html/no-external-base
```

### case: a path

```html
<base href="/app/" target="_self">
```

```expect
(none)
```
