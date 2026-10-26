# sec-js/no-window-open-opener — cases

### case: one argument has no mechanical fix

```js
window.open(url);
```

```expect
1:1 sec-js/no-window-open-opener
```

### case: features with noopener

```js
window.open(url, "_blank", "noopener");
```

```expect
(none)
```

### case: three arguments without noopener

```js
window.open(url, "_blank", "width=400");
```

```expect
1:1 sec-js/no-window-open-opener
```
