# html/hidden-focusable — cases

### case: a disabled button can not take focus

```html
<button type="button" aria-hidden="true" disabled>x</button>
```

```expect
(none)
```

### case: hidden by an ancestor

```html
<div aria-hidden="true">
  <a href="/a">a</a>
</div>
```

```expect
2:3 html/hidden-focusable
```
