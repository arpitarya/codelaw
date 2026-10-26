# sec-ts/angular-native-element-html — cases

### case: Renderer2

```ts
this.renderer.setProperty(el, "innerHTML", html);
```

```expect
1:1 sec-ts/angular-native-element-html
```

### case: textContent is fine

```ts
this.renderer.setProperty(el, "textContent", text);
```

```expect
(none)
```

### case: outerHTML

```ts
this.host.nativeElement.outerHTML = html;
```

```expect
1:1 sec-ts/angular-native-element-html
```

### case: insertAdjacentHTML

```ts
this.host.nativeElement.insertAdjacentHTML("beforeend", html);
```

```expect
1:1 sec-ts/angular-native-element-html
```
