# sec-js/no-javascript-url — cases

### case: an anchor

```js
link.href = "javascript:void(0)";
```

```expect
1:1 sec-js/no-javascript-url
```

### case: location.assign of the hash

```js
location.assign(location.hash.slice(1));
```

```expect
1:1 sec-js/no-javascript-url
```

### case: a relative path

```js
location.href = "/home";
```

```expect
(none)
```

### case: location.replace

```js
location.replace("javascript:alert(1)");
```

```expect
1:1 sec-js/no-javascript-url
```

### case: setAttribute

```js
a.setAttribute("href", "javascript:go()");
```

```expect
1:1 sec-js/no-javascript-url
```

### case: referrer

```js
location.href = document.referrer;
```

```expect
1:1 sec-js/no-javascript-url
```
