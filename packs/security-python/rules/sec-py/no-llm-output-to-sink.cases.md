# sec-py/no-llm-output-to-sink — cases

### case: exec of a completion

```py
exec(completion.choices[0].message.content)
```

```expect
1:1 sec-py/no-llm-output-to-sink
```

### case: a subprocess

```py
subprocess.run(llm_output, shell=True)
```

```expect
1:1 sec-py/no-llm-output-to-sink
```

### case: a lookup

```py
ACTIONS[reply]()
```

```expect
(none)
```
