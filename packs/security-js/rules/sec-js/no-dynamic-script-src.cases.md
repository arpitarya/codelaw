# sec-js/no-dynamic-script-src — cases

### case: an iframe

```js
iframe.setAttribute("src", url);
```

```expect
1:1 sec-js/no-dynamic-script-src
```

### case: an image is not code

```js
img.src = url;
```

```expect
(none)
```

### case: a script variable

```js
tag.src = src;
```

```expect
(none)
```
