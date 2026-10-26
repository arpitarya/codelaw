# sec-py/no-shell — cases

### case: shell=True with a string

```py
import subprocess

subprocess.check_output(cmd, shell=True)
```

```expect
3:1 sec-py/no-shell
```

### case: shell=False

```py
import subprocess

subprocess.run(["ls"], shell=False)
```

```expect
(none)
```

### case: os.system of a literal

```py
os.system("clear")
```

```expect
(none)
```

### case: os.system of an f-string

```py
os.system(f"rm {path}")
```

```expect
1:1 sec-py/no-shell
```

### case: os.popen of a name

```py
os.popen(cmd)
```

```expect
1:1 sec-py/no-shell
```
