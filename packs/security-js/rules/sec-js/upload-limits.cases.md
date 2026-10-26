# sec-js/upload-limits — cases

### case: a shorthand fileFilter property

```js
multer({ limits: { fileSize: 10 }, fileFilter });
```

```expect
(none)
```

### case: no options

```js
const upload = multer();
```

```expect
1:16 sec-js/upload-limits
```
