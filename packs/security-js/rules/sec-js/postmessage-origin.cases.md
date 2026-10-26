# sec-js/postmessage-origin — cases

### case: a listener with a named handler is not read

```js
window.addEventListener("message", onMessage);
```

```expect
(none)
```

### case: a handler that reads source

```js
window.addEventListener("message", (e) => { if (e.source !== frame.contentWindow) return; });
```

```expect
(none)
```

### case: onmessage

```js
window.onmessage = (e) => { use(e.data); };
```

```expect
1:1 sec-js/postmessage-origin
```

### case: single quotes

```js
frame.postMessage(msg, '*');
```

```expect
1:1 sec-js/postmessage-origin
```
