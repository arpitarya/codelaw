# html/element-name — cases

### case: svg children have their own names

```html
<svg><foreignObject></foreignObject><linearGradient></linearGradient></svg>
```

```expect
(none)
```

### case: namespaced names are left alone

```html
<o:p></o:p>
```

```expect
(none)
```
