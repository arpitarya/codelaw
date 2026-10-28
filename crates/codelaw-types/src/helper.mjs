// codelaw-types — the type helper (SR-TYPES decision 2). codelaw's own
// JavaScript, carried inside the engine and run with `node --input-type=module
// --eval`; it asks the project's own TypeScript (>= 7.1, `typescript/sync`)
// and nothing else. It fetches nothing (L5 decision 3).
//
// The protocol is codelaw's (decision 3): one JSON object per line on stdin,
// one answer per line on stdout, in order. The first exchange carries the
// protocol version; an engine that speaks another one refuses the helper.
//
//   -> {"op":"hello","protocol":1,"cwd":"/repo"}
//   <- {"protocol":1,"typescript":"7.1.0-beta"}
//   -> {"op":"check","file":"/repo/a.ts","text":"…","questions":[{"start":0,"end":3,"q":"any"}],"diagnostics":true}
//   <- {"answers":[true],"diagnostics":[{"start":0,"end":3,"code":2322,"category":"error","message":"…"}]}
//   -> {"op":"changed","files":["/repo/a.ts"],"created":[],"deleted":[]}
//   <- {"ok":true}
//   -> {"op":"refresh"}          a warm program re-reads what changed on disk
//   <- {"ok":true,"changed":1}
//   -> {"op":"close"}
//
// Offsets on the wire are UTF-8 bytes into `text`, as the engine counts them;
// TypeScript counts UTF-16 units, so every offset is converted here. Every
// snapshot taken is disposed: the API leaks on its Go side otherwise.

import { readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { pathToFileURL } from "node:url";

const PROTOCOL = 1;
const tsDir = process.argv[process.argv.length - 1];

function send(value) {
    process.stdout.write(JSON.stringify(value) + "\n");
}

/** The module a package exports at `subpath`, as a file path. */
function exported(pkg, subpath) {
    let e = pkg.exports && pkg.exports[subpath];
    while (e && typeof e === "object") e = e.import ?? e.default ?? e.node;
    if (typeof e !== "string") throw new Error(`typescript does not export ${subpath}`);
    return join(tsDir, e);
}

let ts;
let tsVersion;
let SyntaxKind;
let loadError;
let cwd = process.cwd();
try {
    const pkg = JSON.parse(readFileSync(join(tsDir, "package.json"), "utf8"));
    tsVersion = pkg.version;
    ts = await import(pathToFileURL(exported(pkg, "./sync")).href);
    ({ SyntaxKind } = await import(pathToFileURL(exported(pkg, "./ast")).href));
} catch (e) {
    ts = undefined;
    tsVersion = tsVersion ?? null;
    loadError = String(e && e.message ? e.message : e);
}

let api;
let base;

/** The snapshot every question starts from; files open into it. */
function baseSnapshot(cwd) {
    if (!base) {
        api = new ts.API({ cwd });
        base = api.createSnapshot();
    }
    return base;
}

function replaceBase(next) {
    if (next !== base) {
        base.dispose();
        base = next;
    }
}

/** Each file of a loaded program and its modification time, for `refresh`. */
const stamps = new Map();
const stamped = new Set();

function stamp(file) {
    try {
        return statSync(file).mtimeMs;
    } catch {
        return undefined;
    }
}

function stampProject(project) {
    if (stamped.has(project.id)) return;
    stamped.add(project.id);
    for (const f of [...project.program.getSourceFileNames(), ...project.program.getConfigFileNames()]) {
        if (!stamps.has(f)) stamps.set(f, stamp(f));
    }
}

/** The project that holds `file` in the base snapshot, opening it there. */
function projectFor(file) {
    let project = base.getDefaultProjectForFile(file);
    if (!project) {
        replaceBase(base.update({ openFiles: [file], ensurePrograms: true }));
        project = base.getDefaultProjectForFile(file);
    }
    if (project) stampProject(project);
    return project;
}

function refresh() {
    if (!base) return { ok: true, changed: 0 };
    const changed = [];
    const deleted = [];
    for (const [f, was] of stamps) {
        const now = stamp(f);
        if (now === was) continue;
        stamps.set(f, now);
        (now === undefined ? deleted : changed).push(f);
    }
    if (changed.length || deleted.length) {
        replaceBase(base.update({ fileNotifications: { changed, deleted }, ensurePrograms: true }));
        // A program that changed may hold files it did not before.
        stamped.clear();
        for (const p of base.getProjects()) stampProject(p);
    }
    return { ok: true, changed: changed.length + deleted.length };
}

/** UTF-8 byte offset -> UTF-16 offset, for every byte offset in `text`. */
function byteToUtf16(text) {
    const map = new Map();
    let bytes = 0;
    let units = 0;
    for (const ch of text) {
        map.set(bytes, units);
        const cp = ch.codePointAt(0);
        bytes += cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
        units += cp >= 0x10000 ? 2 : 1;
    }
    map.set(bytes, units);
    return map;
}

function utf16ToByte(text) {
    const out = new Array(text.length + 1);
    let bytes = 0;
    let i = 0;
    for (const ch of text) {
        const cp = ch.codePointAt(0);
        const w = cp >= 0x10000 ? 2 : 1;
        for (let k = 0; k < w; k++) out[i + k] = bytes;
        i += w;
        bytes += cp < 0x80 ? 1 : cp < 0x800 ? 2 : cp < 0x10000 ? 3 : 4;
    }
    out[i] = bytes;
    return out;
}

/** The deepest node spanning exactly [start, end), else the smallest one that contains it. */
function nodeAt(sf, start, end) {
    let exact;
    let around;
    const visit = (n) => {
        const s = n.getStart(sf);
        if (s > start || n.end < end) return;
        if (s === start && n.end === end && n.kind !== SyntaxKind.ExpressionStatement) exact = n;
        if (!exact) around = n;
        n.forEachChild(visit);
    };
    sf.forEachChild(visit);
    return exact ?? around;
}

function parts(type) {
    return type.isUnionType() ? type.getTypes() ?? [type] : [type];
}

function callable(checker, type) {
    return parts(type).some((p) => p.getCallSignatures().length > 0);
}

/** A part whose `then` is callable, and, for `promise-like`, takes a callback first (the shape of `Promise`). */
function thenable(checker, type, promiseShape) {
    for (const part of parts(checker.getApparentType(type))) {
        const then = part.getProperty("then");
        if (!then) continue;
        for (const sig of parts(checker.getTypeOfSymbol(then)).flatMap((t) => t.getCallSignatures())) {
            if (!promiseShape) return true;
            const first = sig.getParameters()[0];
            if (first && callable(checker, checker.getNonNullableType(checker.getTypeOfSymbol(first)))) return true;
        }
    }
    return false;
}

/** SR-TYPES decision 4's vocabulary. A question outside it is an error. */
function holds(checker, node, q) {
    const F = ts.TypeFlags;
    if (q === "deprecated") {
        let sym = checker.getSymbolAtLocation(node);
        if (sym && sym.flags & ts.SymbolFlags.Alias) sym = checker.getAliasedSymbol(sym);
        return !!sym && sym.getJsDocTags(checker).some((t) => t.name === "deprecated");
    }
    const type = checker.getTypeAtLocation(node);
    switch (q) {
        case "any":
            return (type.flags & F.Any) !== 0;
        case "never":
            return (type.flags & F.Never) !== 0;
        case "nullable":
            return parts(type).some((p) => (p.flags & (F.Null | F.Undefined | F.Void)) !== 0);
        case "union":
            return type.isUnionType();
        case "thenable":
            return thenable(checker, type, false);
        case "promise-like":
            return thenable(checker, type, true);
        default:
            throw new Error(`unknown type question \`${q}\``);
    }
}

const CATEGORY = ["warning", "error", "suggestion", "message"];

function check(req) {
    baseSnapshot(cwd);
    const file = req.file;
    let project;
    try {
        project = projectFor(file);
    } catch {
        // Not on disk yet (an editor's new buffer, a rule's example): the
        // layer below opens it.
        project = undefined;
    }
    let overlay;
    // The engine's text wins over the disk's (an editor's unsaved buffer, a
    // rule's example): the file is read from a layer in a snapshot of its own.
    let sf = project && project.program.getSourceFile(file);
    if (!sf || sf.text !== req.text) {
        overlay = base.update({ openFiles: [file], fileSystem: { kind: "layer", files: { [file]: req.text } }, ensurePrograms: true });
        project = overlay.getDefaultProjectForFile(file);
        sf = project && project.program.getSourceFile(file);
    }
    try {
        if (!project || !sf) throw new Error("no TypeScript program holds this file");
        const checker = project.checker;
        const toUnits = byteToUtf16(sf.text);
        const answers = (req.questions ?? []).map(({ start, end, q }) => {
            const s = toUnits.get(start);
            const e = toUnits.get(end);
            if (s === undefined || e === undefined) return null;
            const node = nodeAt(sf, s, e);
            return node ? holds(checker, node, q) : null;
        });
        const out = { answers };
        if (req.diagnostics) {
            const toBytes = utf16ToByte(sf.text);
            const flat = (d) => (d.messageChain && d.messageChain.length ? [d.text, ...d.messageChain.map(flat)].join(" ") : d.text);
            out.diagnostics = [...project.program.getSyntacticDiagnostics(file), ...project.program.getSemanticDiagnostics(file)]
                .filter((d) => d.category === 0 || d.category === 1)
                .map((d) => ({
                    start: toBytes[Math.min(d.pos, sf.text.length)],
                    end: toBytes[Math.min(d.end, sf.text.length)],
                    code: d.code,
                    category: CATEGORY[d.category],
                    message: flat(d),
                }));
        }
        return out;
    } finally {
        if (overlay && overlay !== base) overlay.dispose();
    }
}

function changed(req) {
    if (!base) return { ok: true };
    replaceBase(
        base.update({
            fileNotifications: { changed: req.files ?? [], created: req.created ?? [], deleted: req.deleted ?? [] },
            ensurePrograms: true,
        }),
    );
    return { ok: true };
}

const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });
let greeted = false;
for await (const line of lines) {
    if (!line.trim()) continue;
    let req;
    try {
        req = JSON.parse(line);
    } catch (e) {
        send({ error: `not JSON: ${e.message}` });
        continue;
    }
    if (!greeted) {
        greeted = true;
        if (req.op !== "hello" || req.protocol !== PROTOCOL) {
            send({ protocol: PROTOCOL, error: `the engine speaks protocol ${req.protocol}, this helper ${PROTOCOL}` });
            break;
        }
        if (!ts) {
            send({ protocol: PROTOCOL, typescript: tsVersion, error: `cannot load typescript/sync: ${loadError}` });
            break;
        }
        if (typeof req.cwd === "string") cwd = req.cwd;
        send({ protocol: PROTOCOL, typescript: tsVersion });
        continue;
    }
    if (req.op === "close") break;
    try {
        if (req.op === "check") send(check(req));
        else if (req.op === "changed") send(changed(req));
        else if (req.op === "refresh") send(refresh());
        else send({ error: `unknown op \`${req.op}\`` });
    } catch (e) {
        send({ error: String(e && e.message ? e.message : e) });
    }
}
if (base) base.dispose();
if (api) api.close();
// stdin may still be open (a `close` request): nothing else is left to do.
process.exit(0);
