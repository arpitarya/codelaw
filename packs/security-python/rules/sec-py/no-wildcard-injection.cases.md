# sec-py/no-wildcard-injection — cases

### case: tar *

```py
os.system("tar cf a.tar *")
```

```expect
1:1 sec-py/no-wildcard-injection
```

### case: chmod *

```py
subprocess.run("chmod 600 *", shell=True)
```

```expect
1:1 sec-py/no-wildcard-injection
```

### case: ls *

```py
os.system("ls *")
```

```expect
(none)
```

### case: a list

```py
subprocess.run(["tar", "cf", "a"])
```

```expect
(none)
```
