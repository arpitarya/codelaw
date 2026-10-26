# sec-py/no-sql-string — cases

### case: a literal statement

```py
cursor.execute("SELECT 1")
```

```expect
(none)
```

### case: a statement with parameters

```py
cursor.execute("SELECT * FROM t WHERE a = %s", (a,))
```

```expect
(none)
```

### case: percent formatting

```py
cursor.execute("SELECT * FROM t WHERE a = %s" % a)
```

```expect
1:1 sec-py/no-sql-string
```

### case: concatenation

```py
cursor.execute("SELECT * FROM t WHERE a = " + a)
```

```expect
1:1 sec-py/no-sql-string
```

### case: format

```py
db.execute("SELECT {}".format(col))
```

```expect
1:1 sec-py/no-sql-string
```

### case: Django raw with an f-string

```py
User.objects.raw(f"SELECT * FROM auth_user WHERE id = {uid}")
```

```expect
1:1 sec-py/no-sql-string
```

### case: a name

```py
cursor.execute(sql)
```

```expect
(none)
```

### case: extra with an interpolated where

```py
Entry.objects.extra(where=[f"id = {i}"])
```

```expect
1:1 sec-py/no-sql-string
```

### case: extra with parameters

```py
Entry.objects.extra(where=["id = %s"], params=[i])
```

```expect
(none)
```
