# html/heading-level — cases

### case: a dialog starts its own outline

```html
<h1>Page</h1>
<h2>Part</h2>
<dialog><h2>Confirm</h2><h3>Details</h3></dialog>
```

```expect
(none)
```

### case: going back up any number of levels is fine

```html
<h1>a</h1>
<h2>b</h2>
<h3>c</h3>
<h2>d</h2>
```

```expect
(none)
```
