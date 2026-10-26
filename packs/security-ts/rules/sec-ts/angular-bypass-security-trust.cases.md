# sec-ts/angular-bypass-security-trust — cases

### case: a URL

```ts
const u = this.s.bypassSecurityTrustUrl(link);
```

```expect
1:11 sec-ts/angular-bypass-security-trust
```

### case: a resource URL

```ts
this.s.bypassSecurityTrustResourceUrl(src);
```

```expect
1:1 sec-ts/angular-bypass-security-trust
```

### case: a literal

```ts
this.s.bypassSecurityTrustHtml("<b>hi</b>");
```

```expect
(none)
```
