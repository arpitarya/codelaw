# sec-js/angularjs-sce-disabled — cases

### case: resource whitelist

```js
$sceDelegateProvider.resourceUrlWhitelist(["**"]);
```

```expect
1:1 sec-js/angularjs-sce-disabled
```

### case: trustAsHtml

```js
$sce.trustAsHtml(html);
```

```expect
1:1 sec-js/angularjs-sce-disabled
```

### case: href whitelist

```js
$compileProvider.aHrefSanitizationWhitelist(/./);
```

```expect
1:1 sec-js/angularjs-sce-disabled
```

### case: extend: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "@acme/security";
$sce.trustAsHtml(cleanHtml(x));
```

```expect
(none)
```

### case: extend: the built-in DOMPurify still passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import DOMPurify from "dompurify";
$sce.trustAsHtml(DOMPurify.sanitize(x));
```

```expect
(none)
```

### case: replace: the project function passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import { cleanHtml } from "@acme/security";
$sce.trustAsHtml(cleanHtml(x));
```

```expect
(none)
```

### case: replace: DOMPurify is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}],"sanitizers_mode":"replace"}
import DOMPurify from "dompurify";
$sce.trustAsHtml(DOMPurify.sanitize(x));
```

```expect
2:1 sec-js/angularjs-sce-disabled
```

### case: an aliased import passes

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml as clean } from "@acme/security";
$sce.trustAsHtml(clean(x));
```

```expect
(none)
```

### case: a same-named local function is reported

```js
function sanitizeHtml(x) { return x; }
$sce.trustAsHtml(sanitizeHtml(x));
```

```expect
2:1 sec-js/angularjs-sce-disabled
```

### case: a function of the same name from another module is reported

```js options={"sanitizers":[{"name":"cleanHtml","module":"@acme/security"}]}
import { cleanHtml } from "left-pad";
$sce.trustAsHtml(cleanHtml(x));
```

```expect
2:1 sec-js/angularjs-sce-disabled
```

### case: the global DOMPurify passes

```js
$sce.trustAsHtml(DOMPurify.sanitize(x));
```

```expect
(none)
```

### case: the built-in sanitize-html import passes

```js
import sanitizeHtml from "sanitize-html";
$sce.trustAsHtml(sanitizeHtml(x));
```

```expect
(none)
```
