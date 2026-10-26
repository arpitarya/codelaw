# sec-js/no-incomplete-url-check — cases

### case: a file name is not a host

```js
names.includes("index.js");
```

```expect
(none)
```

### case: startsWith with a slash is fine

```js
url.startsWith("https://example.com/");
```

```expect
(none)
```

### case: an unanchored regular expression

```js
/example\.com/.test(url);
```

```expect
1:1 sec-js/no-incomplete-url-check
```

### case: indexOf

```js
if (url.indexOf("example.com") !== -1) {}
```

```expect
1:5 sec-js/no-incomplete-url-check
```

### case: endsWith

```js
host.endsWith("example.com");
```

```expect
1:1 sec-js/no-incomplete-url-check
```

### case: startsWith

```js
url.startsWith("https://example.com");
```

```expect
1:1 sec-js/no-incomplete-url-check
```
