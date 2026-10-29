# react/max-props — cases

### case: destructured props over the limit

```jsx options={"max":2}
function Row({ id, name, email }) {
  return <tr>{id}{name}{email}</tr>;
}
```

```expect
1:10 react/max-props
```

### case: at the limit is fine

```jsx options={"max":3}
function Row({ id, name, email }) {
  return <tr>{id}{name}{email}</tr>;
}
```

```expect
```

### case: a default value and a rename each count once, a rest does not

```jsx options={"max":2}
function Row({ id = 1, name: label, ...rest }) {
  return <tr {...rest}>{id}{label}</tr>;
}
```

```expect
```

### case: a declared interface in the same file

```tsx options={"max":2}
interface RowProps {
  id: string;
  name: string;
  onSelect(): void;
}
const Row = (props: RowProps) => <tr>{props.id}</tr>;
```

```expect
6:7 react/max-props
```

### case: an inline object type

```tsx options={"max":2}
const Row = (props: { id: string; name: string; email: string }) => <tr>{props.id}</tr>;
```

```expect
1:7 react/max-props
```

### case: a function that is not a component is not counted

```jsx options={"max":1}
function pick({ a, b }) {
  return a + b;
}
```

```expect
```

### case: ignore_names exempts a component

```jsx options={"max":1,"ignore_names":"^Row"}
function Row({ id, name }) {
  return <tr>{id}{name}</tr>;
}
```

```expect
```
