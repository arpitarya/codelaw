# sec-js/no-llm-output-to-sink — cases

### case: innerHTML

```js
const t = response.content[0].text;
el.innerHTML = t;
```

```expect
2:1 sec-js/no-llm-output-to-sink
```

### case: a SQL call

```js
const q = completion.choices[0].message.content;
db.query(q);
```

```expect
2:1 sec-js/no-llm-output-to-sink
```

### case: a shell call

```js
const c = result.text;
exec(c);
```

```expect
2:1 sec-js/no-llm-output-to-sink
```
