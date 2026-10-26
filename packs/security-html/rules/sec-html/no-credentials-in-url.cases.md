# sec-html/no-credentials-in-url — cases

### case: a username alone is not a secret

```html
<a href="https://user@host.example/">a</a>
```

```expect
(none)
```

### case: a protocol-relative image URL

```html
<img src="//u:p@h.example/a.png" alt="">
```

```expect
1:6 sec-html/no-credentials-in-url
```
