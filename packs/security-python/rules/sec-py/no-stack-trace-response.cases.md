# sec-py/no-stack-trace-response — cases

### case: str(e)

```py
def v():
    try:
        run()
    except Exception as e:
        return jsonify(error=str(e))
```

```expect
5:9 sec-py/no-stack-trace-response
```

### case: format_exc

```py
def v():
    return traceback.format_exc()
```

```expect
2:5 sec-py/no-stack-trace-response
```

### case: generic

```py
def v():
    return jsonify(error="internal error")
```

```expect
(none)
```

### case: a number

```py
def v(e):
    return len(e)
```

```expect
(none)
```
