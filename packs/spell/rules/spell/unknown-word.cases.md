# Cases: spell/unknown-word

### case: the nearest word, then the commoner one

```js
// a responce and a wierd one
```

```expect
1:6 spell/unknown-word
1:21 spell/unknown-word
```

```fixed
// a response and a weird one
```

### case: acronyms, short words, known typos, names and keys are not read

```js
// HTTPS and FOOBAR, an xyz, a recieve, then fooBar and http://example.com/zzzz
const key = "zzzzzz";
```

```expect
(none)
```

### case: a word with no near word has no edit

```md
The qwxzvb step.
```

```expect
1:5 spell/unknown-word
```
