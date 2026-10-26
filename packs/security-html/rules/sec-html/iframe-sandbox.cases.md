# sec-html/iframe-sandbox — cases

### case: no sandbox

```html
<iframe src="/a"></iframe>
```

```expect
1:1 sec-html/iframe-sandbox
```

```fixed
<iframe sandbox src="/a"></iframe>
```

### case: a sandbox with other tokens is fine

```html
<iframe sandbox="allow-forms" src="/a"></iframe>
```

```expect
(none)
```

### case: scripts and the same origin together

```html
<iframe sandbox="allow-scripts allow-same-origin" src="/a"></iframe>
```

```expect
1:9 sec-html/iframe-sandbox
```

```fixed
<iframe sandbox="allow-scripts" src="/a"></iframe>
```

### case: the same origin token first

```html
<iframe sandbox="allow-same-origin allow-scripts"></iframe>
```

```expect
1:9 sec-html/iframe-sandbox
```

```fixed
<iframe sandbox="allow-scripts"></iframe>
```

### case: a templated sandbox is not judged

```html
<iframe sandbox="{{ tokens }}" src="/a"></iframe>
```

```expect
(none)
```
