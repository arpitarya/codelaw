# sec-py/no-prompt-injection — cases

### case: an anthropic system argument

```py
def v(request):
    t = request.GET["t"]
    return client.messages.create(system=f"About {t}")
```

```expect
3:35 sec-py/no-prompt-injection
```

### case: an openai system message

```py
def v(request):
    m = {"role": "system", "content": request.GET["p"]}
```

```expect
2:9 sec-py/no-prompt-injection
```

### case: a constant

```py
def v(request):
    m = {"role": "system", "content": PROMPT}
```

```expect
(none)
```

### case: a user message

```py
def v(request):
    m = {"role": "user", "content": request.GET["p"]}
```

```expect
(none)
```
