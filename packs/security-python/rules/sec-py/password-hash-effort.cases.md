# sec-py/password-hash-effort — cases

### case: sha256 of a password

```py
hashlib.sha256(password.encode())
```

```expect
1:1 sec-py/password-hash-effort
```

### case: few iterations

```py
hashlib.pbkdf2_hmac("sha256", pw, salt, iterations=100000)
```

```expect
1:1 sec-py/password-hash-effort
```

### case: positional few iterations

```py
hashlib.pbkdf2_hmac("sha256", pw, salt, 1000)
```

```expect
1:1 sec-py/password-hash-effort
```

### case: enough iterations

```py
hashlib.pbkdf2_hmac("sha256", pw, salt, 600000)
```

```expect
(none)
```

### case: bcrypt rounds

```py
bcrypt.gensalt(rounds=4)
```

```expect
1:1 sec-py/password-hash-effort
```

### case: sha256 of a file

```py
hashlib.sha256(data)
```

```expect
(none)
```
