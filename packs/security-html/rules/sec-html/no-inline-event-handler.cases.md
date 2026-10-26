# sec-html/no-inline-event-handler — cases

### case: two handlers on one element

```html
<button onclick="a()" onfocus="b()">x</button>
```

```expect
1:9 sec-html/no-inline-event-handler
1:23 sec-html/no-inline-event-handler
```

### case: framework bindings and data attributes are not handlers

```html
<button data-onclick="a" (click)="b" @click="c" onboarding="d">x</button>
```

```expect
(none)
```

### case: an image error handler

```html
<img src="a.png" alt="" onerror="fallback(this)">
```

```expect
1:25 sec-html/no-inline-event-handler
```
