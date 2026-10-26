# sec-py/no-weak-cipher — cases

### case: ECB

```py
cipher = AES.new(key, AES.MODE_ECB)
```

```expect
1:23 sec-py/no-weak-cipher
```

### case: DES

```py
DES.new(key, DES.MODE_CBC, iv)
```

```expect
1:1 sec-py/no-weak-cipher
```

### case: modes.ECB

```py
modes.ECB()
```

```expect
1:1 sec-py/no-weak-cipher
```

### case: GCM

```py
AES.new(key, AES.MODE_GCM)
```

```expect
(none)
```

### case: TripleDES

```py
algorithms.TripleDES(key)
```

```expect
1:1 sec-py/no-weak-cipher
```
