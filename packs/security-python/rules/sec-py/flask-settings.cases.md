# sec-py/flask-settings — cases

### case: a literal key

```py
app.config["SECRET_KEY"] = "dev-secret-key"
```

```expect
1:1 sec-py/flask-settings
```

### case: csrf off

```py
app.config["WTF_CSRF_ENABLED"] = False
```

```expect
1:1 sec-py/flask-settings
```

### case: update

```py
app.config.update(SECRET_KEY="abcdef")
```

```expect
1:1 sec-py/flask-settings
```

### case: an environment key

```py
app.config["SECRET_KEY"] = os.environ["K"]
```

```expect
(none)
```

### case: a secure cookie

```py
app.config["SESSION_COOKIE_SECURE"] = True
```

```expect
(none)
```
