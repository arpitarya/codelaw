# sec-html/iframe-allow-powerful — cases

### case: camera and microphone

```html
<iframe src="https://a.example/e" allow="camera; microphone; fullscreen"></iframe>
```

```expect
1:35 sec-html/iframe-allow-powerful
```

### case: withheld with none

```html
<iframe src="https://a.example/e" allow="camera 'none'"></iframe>
```

```expect
(none)
```

### case: same origin

```html
<iframe src="/e" allow="camera"></iframe>
```

```expect
(none)
```
