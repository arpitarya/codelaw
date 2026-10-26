---
id: sec-js/no-ssti
title: "No template compiled from a request, no template data taken whole from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which fields the template needs is a decision for the author"
stability: preview
security: { cwe: [1336, 73], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

A template engine's `compile` and `render` of a string are not given request data as the template, and `res.render` is not handed `req.query` or `req.body` as its data.

**Why:** A template is code: a request that supplies it runs on the server (CWE-1336), and request data as the whole data object lets a client set the engine's own options, such as EJS `settings`, and reach files or code (CWE-73).

```codelaw
match:
  any:
    - pattern: ejs.render($T)
    - pattern: ejs.render($T, $$$REST)
    - pattern: ejs.compile($T)
    - pattern: ejs.compile($T, $$$REST)
    - pattern: pug.compile($T)
    - pattern: pug.compile($T, $$$REST)
    - pattern: pug.render($T)
    - pattern: pug.render($T, $$$REST)
    - pattern: Handlebars.compile($T)
    - pattern: Handlebars.compile($T, $$$REST)
    - pattern: handlebars.compile($T)
    - pattern: handlebars.compile($T, $$$REST)
    - pattern: nunjucks.renderString($T)
    - pattern: nunjucks.renderString($T, $$$REST)
    - pattern: nunjucks.compile($T)
    - pattern: nunjucks.compile($T, $$$REST)
    - pattern: _.template($T)
    - pattern: _.template($T, $$$REST)
    - pattern: lodash.template($T)
    - pattern: lodash.template($T, $$$REST)
    - pattern: mustache.render($T)
    - pattern: mustache.render($T, $$$REST)
    - pattern: Mustache.render($T)
    - pattern: Mustache.render($T, $$$REST)
    - pattern: doT.template($T)
    - pattern: doT.template($T, $$$REST)
    - pattern: swig.render($T)
    - pattern: swig.render($T, $$$REST)
    - pattern: res.render($VIEW, $T)
    - pattern: res.render($VIEW, $T, $$$REST)
    - pattern: response.render($VIEW, $T)
    - pattern: response.render($VIEW, $T, $$$REST)
    - pattern: reply.view($VIEW, $T)
    - pattern: reply.view($VIEW, $T, $$$REST)
where:
  $T: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$T` comes from the request and reaches a template engine"
```

## How to fix

1. Compile templates from files or constants in the code, never from a request value.
2. Pass the data object field by field: `{ name: String(req.body.name) }`, not `req.body` or `req.query` whole.
3. Turn off an engine's code-running features (`ejs` `client`, `pug` `compileDebug`) where they are not needed.

## Examples

### JavaScript

#### Don't

```js
const html = ejs.render(req.query.template, { user });
res.render("profile", req.body);
```

#### Do

```js
const html = ejs.render(profileTemplate, { user });
res.render("profile", { name: String(req.body.name) });
```

### JSX

#### Don't

```jsx
const html = ejs.render(req.query.template, { user });
res.render("profile", req.body);
```

#### Do

```jsx
const html = ejs.render(profileTemplate, { user });
res.render("profile", { name: String(req.body.name) });
```

### TypeScript

#### Don't

```ts
const html = ejs.render(req.query.template, { user });
res.render("profile", req.body);
```

#### Do

```ts
const html = ejs.render(profileTemplate, { user });
res.render("profile", { name: String(req.body.name) });
```

### TSX

#### Don't

```tsx
const html = ejs.render(req.query.template, { user });
res.render("profile", req.body);
```

#### Do

```tsx
const html = ejs.render(profileTemplate, { user });
res.render("profile", { name: String(req.body.name) });
```
