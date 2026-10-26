# sec-py/upload-secure-filename — cases

### case: the raw name

```py
f.save(f.filename)
```

```expect
1:1 sec-py/upload-secure-filename
```

### case: joined

```py
file.save(os.path.join(UP, file.filename))
```

```expect
1:1 sec-py/upload-secure-filename
```

### case: secured

```py
f.save(os.path.join(UP, secure_filename(f.filename)))
```

```expect
(none)
```

### case: a generated name

```py
f.save(uuid4().hex)
```

```expect
(none)
```
