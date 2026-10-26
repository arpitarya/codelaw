# sec-html/no-template-html-sink — cases

### case: Vue and Angular

```html
<p v-html="a"></p>
<div [innerHTML]="b"></div>
```

```expect
1:4 sec-html/no-template-html-sink
2:6 sec-html/no-template-html-sink
```

### case: a text binding

```html
<i [textContent]="c"></i>
```

```expect
(none)
```
