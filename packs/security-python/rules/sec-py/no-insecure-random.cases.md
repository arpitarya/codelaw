# sec-py/no-insecure-random — cases

### case: a token

```py
session_token = random.randint(0, 10**9)
```

```expect
1:1 sec-py/no-insecure-random
```

### case: a password

```py
password = random.choice(chars)
```

```expect
1:1 sec-py/no-insecure-random
```

### case: a dice roll

```py
roll = random.randint(1, 6)
```

```expect
(none)
```

### case: secrets

```py
token = secrets.token_hex(16)
```

```expect
(none)
```
