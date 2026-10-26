# sec-js/no-unsanitized-html — cases

### case: insertAdjacentHTML has no mechanical fix

```js
box.insertAdjacentHTML("beforeend", html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: a sanitizer call is not flagged

```js
element.innerHTML = DOMPurify.sanitize(comment.body);
```

```expect
(none)
```

### case: a literal is fine

```js
element.innerHTML = "<b>hi</b>";
```

```expect
(none)
```

### case: outerHTML

```js
element.outerHTML = html;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: document.write

```js
document.write(markup);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: innerHTML +=

```js
el.innerHTML += chunk;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: createContextualFragment

```js
range.createContextualFragment(html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: setHTMLUnsafe

```js
el.setHTMLUnsafe(html);
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: srcdoc

```js
frame.srcdoc = html;
```

```expect
1:1 sec-js/no-unsanitized-html
```

### case: parseFromString

```js
new DOMParser().parseFromString(html, "text/html");
```

```expect
1:1 sec-js/no-unsanitized-html
```
