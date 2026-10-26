# sec-js/no-child-process-shell — cases

### case: a literal command is fine

```js
exec("ls -l");
```

```expect
(none)
```

### case: shell: true on spawn

```js
spawn("ls", [dir], { shell: true });
```

```expect
1:1 sec-js/no-child-process-shell
```

### case: spawn without a shell

```js
spawn("ls", [dir], { cwd: "/tmp" });
```

```expect
(none)
```

### case: a regex exec is not a command

```js
const m = /a(b)/.exec(text);
```

```expect
(none)
```

### case: execSync with a template

```js
execSync(`ls ${dir}`);
```

```expect
1:1 sec-js/no-child-process-shell
```

### case: a member call

```js
child_process.exec(cmd);
```

```expect
1:1 sec-js/no-child-process-shell
```

### case: a literal template

```js
exec(`ls -l`);
```

```expect
(none)
```
