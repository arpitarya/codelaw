# sec-py/tarfile-filter — cases

### case: no filter

```py
tar.extractall(dest)
```

```expect
1:1 sec-py/tarfile-filter
```

### case: a filter

```py
tar.extractall(dest, filter="data")
```

```expect
(none)
```

### case: a zip file

```py
zf.extract(name)
```

```expect
(none)
```

### case: unpack_archive

```py
shutil.unpack_archive(path, dest)
```

```expect
1:1 sec-py/tarfile-filter
```

### case: unpack_archive with a filter

```py
shutil.unpack_archive(path, dest, filter="data")
```

```expect
(none)
```
