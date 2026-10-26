# sec-py/no-weak-hash — cases

### case: md5

```py
hashlib.md5(data)
```

```expect
1:1 sec-py/no-weak-hash
```

### case: sha1

```py
hashlib.sha1(data)
```

```expect
1:1 sec-py/no-weak-hash
```

### case: not for security

```py
hashlib.md5(data, usedforsecurity=False)
```

```expect
(none)
```

### case: sha256

```py
hashlib.sha256(data)
```

```expect
(none)
```
