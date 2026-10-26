# sec-py/no-hardcoded-crypto-key — cases

### case: Fernet

```py
Fernet(b"abc")
```

```expect
1:1 sec-py/no-hardcoded-crypto-key
```

### case: AES

```py
AES.new(b"0123456789abcdef", AES.MODE_GCM)
```

```expect
1:1 sec-py/no-hardcoded-crypto-key
```

### case: a key from the environment

```py
Fernet(os.environ["K"])
```

```expect
(none)
```

### case: a fixed IV

```py
Cipher(alg, modes.CBC(b"0000000000000000"))
```

```expect
1:13 sec-py/no-hardcoded-crypto-key
```

### case: a keyword key

```py
sign(key="secret-key")
```

```expect
1:1 sec-py/no-hardcoded-crypto-key
```
