# sec-js/no-bidi-characters — cases

### case: a string with a bidi character has no mechanical fix

```js
const user = "admin‮ ";
```

```expect
1:14 sec-js/no-bidi-characters
```

### case: an escape is not a hidden character

```js
const mark = "\u202E";
```

```expect
(none)
```
