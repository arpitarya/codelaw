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
