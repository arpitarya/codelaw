# sec-py/flask-debug — cases

### case: debug=True

```py
app.run(host="127.0.0.1", debug=True)
```

```expect
1:1 sec-py/flask-debug
```

### case: debug=False

```py
app.run(debug=False)
```

```expect
(none)
```

### case: no debug

```py
app.run()
```

```expect
(none)
```
