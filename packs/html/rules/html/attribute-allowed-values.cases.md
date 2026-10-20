# html/attribute-allowed-values — cases

### case: unknown elements are not checked

```html
<x-widget type="anything"></x-widget>
```

```expect
(none)
```

### case: template code is not checked

```html
<input type="{{ kind }}">
```

```expect
(none)
```

### case: boolean attribute written with its own name

```html
<input type="checkbox" checked="checked">
```

```expect
(none)
```
