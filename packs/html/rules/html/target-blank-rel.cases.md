# html/target-blank-rel — cases

### case: noreferrer also prevents the opener

```html
<a href="/a" target="_blank" rel="noreferrer">a</a>
```

```expect
(none)
```

### case: an existing rel gets noopener added

```html
<a href="/a" target="_blank" rel="external">a</a>
```

```expect
1:14 html/target-blank-rel
```

```fixed
<a href="/a" target="_blank" rel="external noopener">a</a>
```

### case: a link without href opens nothing

```html
<a target="_blank">a</a>
```

```expect
(none)
```
