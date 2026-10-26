# sec-html/referrer-policy — cases

### case: an attribute

```html
<a href="/a" referrerpolicy="unsafe-url">a</a>
```

```expect
1:14 sec-html/referrer-policy
```

```fixed
<a href="/a" referrerpolicy="strict-origin-when-cross-origin">a</a>
```

### case: a meta tag

```html
<meta name="referrer" content="no-referrer-when-downgrade">
```

```expect
1:23 sec-html/referrer-policy
```

```fixed
<meta name="referrer" content="strict-origin-when-cross-origin">
```

### case: no-referrer is safe

```html
<meta name="referrer" content="no-referrer">
```

```expect
(none)
```
