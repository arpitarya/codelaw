# sec-py/no-ldap-injection — cases

### case: an f-string filter

```py
conn.search_s(base, 2, f"(uid={name})")
```

```expect
1:1 sec-py/no-ldap-injection
```

### case: ldap3

```py
conn.search(base, "(cn=" + name + ")")
```

```expect
1:1 sec-py/no-ldap-injection
```

### case: a fixed filter

```py
conn.search(base, "(objectClass=*)")
```

```expect
(none)
```

### case: a re.search is not matched

```py
re.search("a", text)
```

```expect
(none)
```
