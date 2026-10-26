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

### case: extend: the project function passes

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "@acme/security";
this.el.nativeElement.innerHTML = cleanHtml(x);
```

```expect
(none)
```

### case: extend: the built-in DOMPurify still passes

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import DOMPurify from "dompurify";
this.el.nativeElement.innerHTML = DOMPurify.sanitize(x);
```

```expect
(none)
```

### case: replace: the project function passes

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import { cleanHtml } from "@acme/security";
this.el.nativeElement.innerHTML = cleanHtml(x);
```

```expect
(none)
```

### case: replace: DOMPurify is reported

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import DOMPurify from "dompurify";
this.el.nativeElement.innerHTML = DOMPurify.sanitize(x);
```

```expect
2:1 sec-ts/angular-native-element-html
```

### case: an aliased import passes

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml as clean } from "@acme/security";
this.el.nativeElement.innerHTML = clean(x);
```

```expect
(none)
```

### case: a same-named local function is reported

```ts
function sanitizeHtml(x) { return x; }
this.el.nativeElement.innerHTML = sanitizeHtml(x);
```

```expect
2:1 sec-ts/angular-native-element-html
```

### case: a function of the same name from another module is reported

```ts options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "left-pad";
this.el.nativeElement.innerHTML = cleanHtml(x);
```

```expect
2:1 sec-ts/angular-native-element-html
```

### case: the global DOMPurify passes

```ts
this.el.nativeElement.innerHTML = DOMPurify.sanitize(x);
```

```expect
(none)
```

### case: the built-in sanitize-html import passes

```ts
import sanitizeHtml from "sanitize-html";
this.el.nativeElement.innerHTML = sanitizeHtml(x);
```

```expect
(none)
```
