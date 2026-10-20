# html/img-alt — cases

### case: a hidden image is not announced

```html
<img src="a.png" hidden>
<div aria-hidden="true"><img src="b.png"></div>
```

```expect
(none)
```

### case: an empty alt marks a decorative image

```html
<img src="a.png" alt="">
```

```expect
(none)
```
