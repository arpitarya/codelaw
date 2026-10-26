# sec-py/no-world-writable — cases

### case: 777

```py
os.chmod(path, 0o777)
```

```expect
1:1 sec-py/no-world-writable
```

### case: 666

```py
os.chmod(path, 0o666)
```

```expect
1:1 sec-py/no-world-writable
```

### case: 600

```py
os.chmod(path, 0o600)
```

```expect
(none)
```

### case: 755

```py
os.chmod(path, 0o755)
```

```expect
(none)
```

### case: S_IWOTH

```py
os.chmod(path, stat.S_IWOTH)
```

```expect
1:1 sec-py/no-world-writable
```
