# sec-py/weak-key-size — cases

### case: 1024

```py
rsa.generate_private_key(65537, key_size=1024)
```

```expect
1:1 sec-py/weak-key-size
```

### case: 2048

```py
rsa.generate_private_key(65537, key_size=2048)
```

```expect
(none)
```

### case: pycrypto

```py
RSA.generate(1024)
```

```expect
1:1 sec-py/weak-key-size
```

### case: a small curve

```py
ec.generate_private_key(ec.SECP192R1())
```

```expect
1:25 sec-py/weak-key-size
```

### case: a good curve

```py
ec.generate_private_key(ec.SECP256R1())
```

```expect
(none)
```
