# sec-js/no-prompt-injection — cases

### case: the system option of the Vercel AI SDK

```js
const r = await generateText({ model, system: "You help " + req.body.who, prompt });
```

```expect
1:39 sec-js/no-prompt-injection
```

### case: a user message from the request is fine

```js
const m = { role: "user", content: req.body.q };
```

```expect
(none)
```

### case: the instructions option

```js
run({ instructions: `Help ${req.query.n}` });
```

```expect
1:7 sec-js/no-prompt-injection
```
