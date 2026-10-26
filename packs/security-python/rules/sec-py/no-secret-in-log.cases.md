# sec-py/no-secret-in-log — cases

### case: a password

```py
logger.info("pw=%s", password)
```

```expect
1:1 sec-py/no-secret-in-log
```

### case: an f-string

```py
log.debug(f"token {token}")
```

```expect
1:1 sec-py/no-secret-in-log
```

### case: a user name

```py
logger.info("user %s", username)
```

```expect
(none)
```

### case: print

```py
print(api_key)
```

```expect
1:1 sec-py/no-secret-in-log
```
