# sec-html/no-javascript-url — cases

### case: case and leading space do not hide it

```html
<a href=" JavaScript:alert(1)">a</a>
```

```expect
1:4 sec-html/no-javascript-url
```

### case: a tab inside the scheme is still javascript:

```html
<a href="java	script:alert(1)">a</a>
```

```expect
1:4 sec-html/no-javascript-url
```

### case: formaction

```html
<button formaction="javascript:go()">a</button>
```

```expect
1:9 sec-html/no-javascript-url
```

### case: object data

```html
<object data="javascript:go()"></object>
```

```expect
1:9 sec-html/no-javascript-url
```

### case: data on another element is not a URL

```html
<div data="javascript:go()">a</div>
```

```expect
(none)
```

### case: an ordinary link

```html
<a href="/javascript:">a</a>
```

```expect
(none)
```
