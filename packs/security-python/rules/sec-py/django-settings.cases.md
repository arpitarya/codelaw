# sec-py/django-settings — cases

### case: DEBUG

```py path=settings.py
DEBUG = True
```

```expect
1:1 sec-py/django-settings
```

### case: wildcard hosts

```py path=settings.py
ALLOWED_HOSTS = ["*"]
```

```expect
1:1 sec-py/django-settings
```

### case: a literal key

```py path=settings.py
SECRET_KEY = "django-insecure-abc123"
```

```expect
1:1 sec-py/django-settings
```

### case: a key from the environment

```py path=settings.py
SECRET_KEY = os.environ["KEY"]
```

```expect
(none)
```

### case: an insecure cookie

```py path=settings.py
SESSION_COOKIE_SECURE = False
```

```expect
1:1 sec-py/django-settings
```

### case: a secure cookie

```py path=settings.py
CSRF_COOKIE_SECURE = True
```

```expect
(none)
```
