# sec-js/no-new-buffer — cases

### case: a size

```js
const buf = new Buffer(16);
```

```expect
1:13 sec-js/no-new-buffer
```

### case: noAssert

```js
const n = buf.readUInt32LE(0, true);
```

```expect
1:11 sec-js/no-new-buffer
```

### case: with an encoding

```js
const b = new Buffer(text, "hex");
```

```expect
1:11 sec-js/no-new-buffer
```

### case: write with noAssert

```js
buf.writeUInt8(1, 0, true);
```

```expect
1:1 sec-js/no-new-buffer
```
