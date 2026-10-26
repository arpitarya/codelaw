---
id: sec-js/no-path-traversal
title: "No file path taken from a request"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "which folder the value must stay in is a decision for the author"
stability: preview
ported_from: [eslint-plugin-security/detect-non-literal-fs-filename]
security: { cwe: [22], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

A file is not opened, written, listed or sent from a path built from request data: `fs.*`, `path.join` and `res.sendFile` do not take a request value.

**Why:** A value with `../` in it, or an absolute path, leaves the folder the code meant and reads or overwrites any file the process can (CWE-22).

```codelaw
match:
  any:
    - pattern: $FS.readFile($P)
    - pattern: $FS.readFile($P, $$$REST)
    - pattern: $FS.readFileSync($P)
    - pattern: $FS.readFileSync($P, $$$REST)
    - pattern: $FS.createReadStream($P)
    - pattern: $FS.createReadStream($P, $$$REST)
    - pattern: $FS.writeFile($P)
    - pattern: $FS.writeFile($P, $$$REST)
    - pattern: $FS.writeFileSync($P)
    - pattern: $FS.writeFileSync($P, $$$REST)
    - pattern: $FS.appendFile($P)
    - pattern: $FS.appendFile($P, $$$REST)
    - pattern: $FS.appendFileSync($P)
    - pattern: $FS.appendFileSync($P, $$$REST)
    - pattern: $FS.createWriteStream($P)
    - pattern: $FS.createWriteStream($P, $$$REST)
    - pattern: $FS.unlink($P)
    - pattern: $FS.unlink($P, $$$REST)
    - pattern: $FS.unlinkSync($P)
    - pattern: $FS.unlinkSync($P, $$$REST)
    - pattern: $FS.readdir($P)
    - pattern: $FS.readdir($P, $$$REST)
    - pattern: $FS.readdirSync($P)
    - pattern: $FS.readdirSync($P, $$$REST)
    - pattern: $FS.stat($P)
    - pattern: $FS.stat($P, $$$REST)
    - pattern: $FS.statSync($P)
    - pattern: $FS.statSync($P, $$$REST)
    - pattern: $FS.lstat($P)
    - pattern: $FS.lstat($P, $$$REST)
    - pattern: $FS.lstatSync($P)
    - pattern: $FS.lstatSync($P, $$$REST)
    - pattern: $FS.access($P)
    - pattern: $FS.access($P, $$$REST)
    - pattern: $FS.accessSync($P)
    - pattern: $FS.accessSync($P, $$$REST)
    - pattern: $FS.open($P)
    - pattern: $FS.open($P, $$$REST)
    - pattern: $FS.openSync($P)
    - pattern: $FS.openSync($P, $$$REST)
    - pattern: $FS.rm($P)
    - pattern: $FS.rm($P, $$$REST)
    - pattern: $FS.rmSync($P)
    - pattern: $FS.rmSync($P, $$$REST)
    - pattern: $FS.rmdir($P)
    - pattern: $FS.rmdir($P, $$$REST)
    - pattern: $FS.rmdirSync($P)
    - pattern: $FS.rmdirSync($P, $$$REST)
    - pattern: $FS.mkdir($P)
    - pattern: $FS.mkdir($P, $$$REST)
    - pattern: $FS.mkdirSync($P)
    - pattern: $FS.mkdirSync($P, $$$REST)
    - pattern: $FS.copyFile($P)
    - pattern: $FS.copyFile($P, $$$REST)
    - pattern: $FS.copyFileSync($P)
    - pattern: $FS.copyFileSync($P, $$$REST)
    - pattern: $FS.realpath($P)
    - pattern: $FS.realpath($P, $$$REST)
    - pattern: $FS.realpathSync($P)
    - pattern: $FS.realpathSync($P, $$$REST)
    - pattern: $FS.opendir($P)
    - pattern: $FS.opendir($P, $$$REST)
    - pattern: $FS.truncate($P)
    - pattern: $FS.truncate($P, $$$REST)
    - pattern: res.sendFile($P)
    - pattern: res.sendFile($P, $$$REST)
    - pattern: res.download($P)
    - pattern: res.download($P, $$$REST)
    - pattern: response.sendFile($P)
    - pattern: response.sendFile($P, $$$REST)
    - pattern: reply.sendFile($P)
    - pattern: reply.sendFile($P, $$$REST)
    - pattern: ctx.sendFile($P)
    - pattern: ctx.sendFile($P, $$$REST)
    - pattern: send($P)
    - pattern: send($P, $$$REST)
    - all:
        - pattern: path.join($$$JARGS)
        - has:
            stopBy: end
            pattern: $P
            regex: '^(req|request|ctx\.request)\.(params|query|body|headers|cookies)\b|^ctx\.(params|query)\b'
constraints:
  FS: { regex: '^(fs|fsp|fsPromises|fs\.promises|fse|fsExtra)$' }
where:
  $P: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$P` comes from the request and chooses which file is used"
```

## How to fix

1. Resolve the path (`path.resolve(root, value)`) and check the result starts with `root + path.sep` before it is used.
2. Or look the file up by an id in a table, so no part of the path comes from the request.
3. With `res.sendFile`, pass the `root` option and the bare name, and reject a name that contains a separator.

## Examples

### JavaScript

#### Don't

```js
const data = fs.readFileSync(path.join(root, req.params.name));
```

#### Do

```js
const full = path.resolve(root, req.params.name);
if (!full.startsWith(root + path.sep)) {
  throw new Error("bad path");
}
const data = fs.readFileSync(full);
```

### JSX

#### Don't

```jsx
const data = fs.readFileSync(path.join(root, req.params.name));
```

#### Do

```jsx
const full = path.resolve(root, req.params.name);
if (!full.startsWith(root + path.sep)) {
  throw new Error("bad path");
}
const data = fs.readFileSync(full);
```

### TypeScript

#### Don't

```ts
const data = fs.readFileSync(path.join(root, req.params.name));
```

#### Do

```ts
const full = path.resolve(root, req.params.name);
if (!full.startsWith(root + path.sep)) {
  throw new Error("bad path");
}
const data = fs.readFileSync(full);
```

### TSX

#### Don't

```tsx
const data = fs.readFileSync(path.join(root, req.params.name));
```

#### Do

```tsx
const full = path.resolve(root, req.params.name);
if (!full.startsWith(root + path.sep)) {
  throw new Error("bad path");
}
const data = fs.readFileSync(full);
```
