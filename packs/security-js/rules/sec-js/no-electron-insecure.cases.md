# sec-js/no-electron-insecure — cases

### case: nodeIntegration is flagged with no mechanical fix

```js
const w = new BrowserWindow({ webPreferences: { nodeIntegration: true } });
```

```expect
1:49 sec-js/no-electron-insecure
```

### case: the safe values

```js
const w = new BrowserWindow({ webPreferences: { nodeIntegration: false, contextIsolation: true, sandbox: true } });
```

```expect
(none)
```

### case: webSecurity

```js
const o = { webSecurity: false };
```

```expect
1:13 sec-js/no-electron-insecure
```

### case: sandbox

```js
const o = { sandbox: false };
```

```expect
1:13 sec-js/no-electron-insecure
```
