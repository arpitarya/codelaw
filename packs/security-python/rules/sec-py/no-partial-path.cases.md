# sec-py/no-partial-path — cases

### case: a bare list

```py
subprocess.run(["git", "status"])
```

```expect
1:1 sec-py/no-partial-path
```

### case: a bare string

```py
subprocess.run("git status", shell=True)
```

```expect
1:1 sec-py/no-partial-path
```

### case: a full path

```py
subprocess.run(["/usr/bin/git", "status"])
```

```expect
(none)
```

### case: a variable

```py
subprocess.run([GIT, "status"])
```

```expect
(none)
```
