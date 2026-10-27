# Cases: spell/known-typo

### case: a typo in a comment

```js
// we recieve it here
```

```expect
1:7 spell/known-typo
```

### case: a string with no whitespace is a name, not prose

```js
const key = "recieve";
```

```expect
(none)
```

### case: a template literal with a substitution is not read

```ts
const s = `untill ${now} later`;
```

```expect
(none)
```

### case: URLs, paths, identifiers and calls are skipped

```js
// see http://example.com/recieve, src/recieve.ts, recieve_all, recieveAll and recieve()
```

```expect
(none)
```

### case: a JSON key is a name; a value with whitespace is prose

```json
{ "recieve": "the adress of it" }
```

```expect
1:19 spell/known-typo
```

### case: YAML and TOML values and comments

```yaml
# untill now
title: the lenght of it
recieve: x
```

```expect
1:3 spell/known-typo
2:12 spell/known-typo
```

### case: HTML text and the attributes people read

```html
<img alt="the adress" class="recieve all"><p>lenght here</p>
```

```expect
1:15 spell/known-typo
1:46 spell/known-typo
```

```fixed
<img alt="the address" class="recieve all"><p>length here</p>
```

### case: Markdown code spans and link targets are not prose

```md
The `recieve` call and [the docs](https://example.com/recieve) are fine; the adress is not.
```

```expect
1:78 spell/known-typo
```

```fixed
The `recieve` call and [the docs](https://example.com/recieve) are fine; the address is not.
```

### case: the case of the word is kept

```py
# Recieve and RECIEVE
```

```expect
1:3 spell/known-typo
1:15 spell/known-typo
```

```fixed
# Receive and RECEIVE
```

### case: CSS and SCSS comments

```scss
// untill
.a { content: "the adress"; }
```

```expect
1:4 spell/known-typo
```

### case: a generated entry has one correction, applied in place

```yaml
note: the occured event
```

```expect
1:11 spell/known-typo
```

```fixed
note: the occurred event
```
