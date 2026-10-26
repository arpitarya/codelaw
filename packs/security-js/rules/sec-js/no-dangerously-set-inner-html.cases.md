# sec-js/no-dangerously-set-inner-html — cases

### case: in JSX

```jsx
const a = <div dangerouslySetInnerHTML={{ __html: html }} />;
```

```expect
1:43 sec-js/no-dangerously-set-inner-html
```

### case: a literal

```js
const p = { dangerouslySetInnerHTML: { __html: "<b>x</b>" } };
```

```expect
(none)
```

### case: an unsanitized call result

```js
const p = { dangerouslySetInnerHTML: { __html: render(md) } };
```

```expect
1:40 sec-js/no-dangerously-set-inner-html
```
