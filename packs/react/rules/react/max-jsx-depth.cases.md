# react/max-jsx-depth — cases

### case: the first element past the limit is reported

```jsx options={"max":2}
const Page = () => (
  <a>
    <b>
      <c>
        <d />
      </c>
    </b>
  </a>
);
```

```expect
4:7 react/max-jsx-depth
```

### case: at the limit is fine

```jsx options={"max":4}
const Page = () => (
  <a>
    <b>
      <c>
        <d />
      </c>
    </b>
  </a>
);
```

```expect
```

### case: a fragment counts

```jsx options={"max":1}
const Page = () => (
  <>
    <a>
      <b />
    </a>
  </>
);
```

```expect
3:5 react/max-jsx-depth
```

### case: nesting goes through a map callback

```jsx options={"max":2}
const Page = () => (
  <ul>
    {xs.map((x) => (
      <li>
        <b>{x}</b>
      </li>
    ))}
  </ul>
);
```

```expect
5:9 react/max-jsx-depth
```

### case: two deep branches are two findings

```jsx options={"max":1}
const Page = () => (
  <a>
    <b>
      <c />
    </b>
    <d>
      <e />
    </d>
  </a>
);
```

```expect
3:5 react/max-jsx-depth
6:5 react/max-jsx-depth
```
