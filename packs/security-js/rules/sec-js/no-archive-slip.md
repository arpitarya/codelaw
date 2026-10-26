---
id: sec-js/no-archive-slip
title: "No archive entry name written to disk unchecked"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what to do with an entry that leaves the folder (skip, stop, report) is a decision for the author"
stability: preview
ported_from: [bandit/B202, ruff/S202]
security: { cwe: [22], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

A file written while unpacking an archive is not named by the entry's own path or name without a check that it stays inside the target folder.

**Why:** An entry named `../../etc/cron.d/x` or an absolute path writes outside the folder the code meant: the zip-slip bug (CWE-22).

```codelaw
match:
  any:
    - pattern: $FS.createWriteStream($P)
    - pattern: $FS.createWriteStream($P, $$$REST)
    - pattern: $FS.writeFile($P)
    - pattern: $FS.writeFile($P, $$$REST)
    - pattern: $FS.writeFileSync($P)
    - pattern: $FS.writeFileSync($P, $$$REST)
    - pattern: $FS.mkdir($P)
    - pattern: $FS.mkdir($P, $$$REST)
    - pattern: $FS.mkdirSync($P)
    - pattern: $FS.mkdirSync($P, $$$REST)
    - pattern: $FS.appendFile($P)
    - pattern: $FS.appendFile($P, $$$REST)
    - pattern: $FS.copyFile($P)
    - pattern: $FS.copyFile($P, $$$REST)
    - pattern: $FS.rename($P)
    - pattern: $FS.rename($P, $$$REST)
constraints:
  FS: { regex: '^(fs|fsp|fsPromises|fs\.promises|fse|fsExtra)$' }
  P: { regex: '\b(entry|zipEntry|header|file|item|member)\.(path|fileName|entryName|name|filename)\b|\bentry\.entryName\b' }
message: "An archive entry's own name chooses where this file is written"
```

## How to fix

1. Resolve the entry's path against the target folder and reject the entry when the result does not start with `target + path.sep`.
2. Skip symbolic links and absolute entry names; set a limit on the number and total size of entries (a zip bomb).

## Examples

### JavaScript

#### Don't

```js
entry.pipe(fs.createWriteStream(path.join(dest, entry.path)));
```

#### Do

```js
const target = path.resolve(dest, entry.path);
if (!target.startsWith(dest + path.sep)) {
  throw new Error("archive entry leaves the folder");
}
entry.pipe(fs.createWriteStream(target));
```

### JSX

#### Don't

```jsx
entry.pipe(fs.createWriteStream(path.join(dest, entry.path)));
```

#### Do

```jsx
const target = path.resolve(dest, entry.path);
if (!target.startsWith(dest + path.sep)) {
  throw new Error("archive entry leaves the folder");
}
entry.pipe(fs.createWriteStream(target));
```

### TypeScript

#### Don't

```ts
entry.pipe(fs.createWriteStream(path.join(dest, entry.path)));
```

#### Do

```ts
const target = path.resolve(dest, entry.path);
if (!target.startsWith(dest + path.sep)) {
  throw new Error("archive entry leaves the folder");
}
entry.pipe(fs.createWriteStream(target));
```

### TSX

#### Don't

```tsx
entry.pipe(fs.createWriteStream(path.join(dest, entry.path)));
```

#### Do

```tsx
const target = path.resolve(dest, entry.path);
if (!target.startsWith(dest + path.sep)) {
  throw new Error("archive entry leaves the folder");
}
entry.pipe(fs.createWriteStream(target));
```
