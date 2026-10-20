# html/no-inline-style — cases

### case: display and custom properties are allowed by default

```html
<p style="display: none">a</p>
<p style="--gap: 4px">b</p>
```

```expect
(none)
```
