---
id: sec-js/no-non-literal-fs-filename
title: "No file path that is not a literal"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "whether a computed path is safe depends on where it comes from"
stability: preview
ported_from: [eslint-plugin-security/detect-non-literal-fs-filename]
security: { cwe: [22], owasp: ["A01:2025"], severity: low }
since: 0.3.0
---

`fs.*` and its promise and `fs-extra` forms are given a literal path.

**Why:** A path computed at run time is where traversal bugs come from; flagging every one lets a reviewer check that each is safe (CWE-22). This is the broad cousin of `sec-js/no-path-traversal`.

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
constraints:
  FS: { regex: '^(fs|fsp|fsPromises|fs\.promises|fse|fsExtra)$' }
  P: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "The path `$P` is not a literal; check that no request value reaches it"
```

## How to fix

1. Use a literal where the file is fixed.
2. Where the path is computed, resolve it against a base folder and check the result stays inside it, then suppress this finding with a reason that names the check.

## Examples

### JavaScript

#### Don't

```js
const text = fs.readFileSync(file, "utf8");
```

#### Do

```js
const text = fs.readFileSync("./config.json", "utf8");
```

### JSX

#### Don't

```jsx
const text = fs.readFileSync(file, "utf8");
```

#### Do

```jsx
const text = fs.readFileSync("./config.json", "utf8");
```

### TypeScript

#### Don't

```ts
const text = fs.readFileSync(file, "utf8");
```

#### Do

```ts
const text = fs.readFileSync("./config.json", "utf8");
```

### TSX

#### Don't

```tsx
const text = fs.readFileSync(file, "utf8");
```

#### Do

```tsx
const text = fs.readFileSync("./config.json", "utf8");
```
