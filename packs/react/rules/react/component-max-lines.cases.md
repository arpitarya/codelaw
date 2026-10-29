# react/component-max-lines — cases

### case: a component over the limit is reported on its name

```jsx options={"max":3}
function Card() {
  const a = 1;
  const b = 2;
  return <div>{a}{b}</div>;
}
```

```expect
1:10 react/component-max-lines
```

### case: at the limit is fine

```jsx options={"max":5}
function Card() {
  const a = 1;
  const b = 2;
  return <div>{a}{b}</div>;
}
```

```expect
```

### case: an arrow component named by its variable

```jsx options={"max":3}
const Card = () => {
  const a = 1;
  const b = 2;
  return <div>{a}{b}</div>;
};
```

```expect
1:7 react/component-max-lines
```

### case: memo and forwardRef wrap a component

```jsx options={"max":3}
const Card = memo(
  forwardRef((props, ref) => {
    const a = 1;
    return <div ref={ref}>{a}</div>;
  }),
);
```

```expect
1:7 react/component-max-lines
```

### case: a class component

```jsx options={"max":3}
class Card extends React.Component {
  render() {
    const a = 1;
    return <div>{a}</div>;
  }
}
```

```expect
1:7 react/component-max-lines
```

### case: a lower-case function that returns JSX is not a component

```jsx options={"max":3}
function renderCard() {
  const a = 1;
  const b = 2;
  return <div>{a}{b}</div>;
}
```

```expect
```

### case: blank lines and comments are not counted

```jsx options={"max":4}
function Card() {
  // note

  const a = 1;
  return <div>{a}</div>;
}
```

```expect
```

### case: ignore_names exempts a component

```jsx options={"max":3,"ignore_names":"^Card"}
function Card() {
  const a = 1;
  const b = 2;
  return <div>{a}{b}</div>;
}
```

```expect
```
