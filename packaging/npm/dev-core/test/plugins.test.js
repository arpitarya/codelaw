// The dev-server plugins against the real codelaw binary (W-49). Each host is a
// small fake of the API surface the plugin uses, because the hosts themselves
// are third-party code (L11). Run: CODELAW_BINARY=target/debug/codelaw node --test packaging/npm/dev-core/test
import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "../../../..");
const binary = process.env.CODELAW_BINARY;
const skip = binary ? false : "CODELAW_BINARY is not set";

import * as core from "../core.js";
import vite from "../../vite-plugin/index.js";
import { CodelawPlugin } from "../../webpack-plugin/index.js";
import { withCodelaw } from "../../next/index.js";

function project(code = "debugger;\n") {
  const d = mkdtempSync(join(tmpdir(), "codelaw-dev-"));
  mkdirSync(join(d, "src"));
  cpSync(join(repo, "packs/js"), join(d, "node_modules/@codelaw/rules-js"), { recursive: true });
  writeFileSync(join(d, "codelaw.toml"), 'extends = ["codelaw:js/recommended"]\npreview = true\n');
  writeFileSync(join(d, "package.json"), '{ "name": "p", "main": "src/main.js" }\n');
  writeFileSync(join(d, "src/main.js"), code);
  process.env.CODELAW_CACHE_DIR = d + ".cache";
  return d;
}

const until = async (f, ms = 15000) => {
  const t0 = Date.now();
  for (;;) {
    const v = f();
    if (v) return v;
    if (Date.now() - t0 > ms) throw new Error("timed out");
    await new Promise((r) => setTimeout(r, 50));
  }
};
const alive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
};

test("core: a ci check reports the error, and the watcher follows a save and dies on stop", { skip }, async () => {
  const d = project();
  const ci = core.runCheck({ root: d, binary });
  assert.equal(core.partition(ci).errors[0].rule, "js/no-debugger");
  assert.match(core.buildError(ci).message, /src\/main\.js:1:1 {2}error {2}js\/no-debugger/);

  const reports = [];
  const w = core.startWatch({ root: d, binary, onReport: (r, o) => o.changed && reports.push(r) });
  await until(() => reports.length === 1);
  assert.equal(core.partition(reports[0]).errors.length, 1);
  writeFileSync(join(d, "src/main.js"), "export const a = 1;\n");
  await until(() => reports.length === 2);
  assert.equal(core.partition(reports[1]).errors.length, 0);
  const pid = w.child.pid;
  w.stop();
  await until(() => !alive(pid));
  rmSync(d, { recursive: true, force: true });
});

test("vite: dev sends the error to the overlay, a fix stops it; build fails; closing the server kills the watcher", { skip }, async () => {
  const d = project();
  const sent = [];
  const logs = [];
  const http = new EventEmitter();
  const server = {
    config: { logger: { info: (m) => logs.push(m), warn: (m) => logs.push(m), error: (m) => logs.push(m) } },
    ws: { send: (m) => sent.push(m) },
    httpServer: http,
    close: async () => {},
  };
  const p = vite({ binary });
  p.configResolved({ root: d, command: "serve" });
  p.configureServer(server);
  await until(() => sent.length >= 1);
  assert.equal(sent[0].type, "error");
  assert.equal(sent[0].err.loc.line, 1);
  assert.match(sent[0].err.message, /js\/no-debugger/);
  writeFileSync(join(d, "src/main.js"), "export const a = 1;\n");
  await until(() => logs.some((l) => l.includes("clean")));
  http.emit("close");

  const b = vite({ binary });
  b.configResolved({ root: d, command: "build" });
  b.buildStart.call({ error: (m) => assert.fail(m), warn() {} }); // clean now: no throw
  writeFileSync(join(d, "src/main.js"), "debugger;\n");
  assert.throws(() => b.buildStart.call({ error: (m) => { throw new Error(m); }, warn() {} }), /js\/no-debugger/);
  rmSync(d, { recursive: true, force: true });
});

function compilerFor(d) {
  const taps = {};
  const hooks = new Proxy({}, { get: (_, name) => (taps[name] ??= { tap: (_n, fn) => ((taps[name].fn = fn), undefined) }) });
  return { context: d, hooks, taps, watching: { invalidated: 0, invalidate() { this.invalidated++; } } };
}

test("webpack and rspack: build errors fail, failOnError false warns; watch recompiles with the report", { skip }, async () => {
  const d = project();
  const c = compilerFor(d);
  new CodelawPlugin({ binary }).apply(c);
  c.taps.run.fn();
  const comp = { errors: [], warnings: [] };
  c.taps.thisCompilation.fn(comp);
  assert.equal(comp.errors.length, 1);
  assert.match(comp.errors[0].message, /js\/no-debugger/);

  const c2 = compilerFor(d);
  new CodelawPlugin({ binary, failOnError: false }).apply(c2);
  c2.taps.run.fn();
  const comp2 = { errors: [], warnings: [] };
  c2.taps.thisCompilation.fn(comp2);
  assert.deepEqual([comp2.errors.length, comp2.warnings.length], [0, 1]);

  const c3 = compilerFor(d);
  new CodelawPlugin({ binary }).apply(c3);
  c3.taps.watchRun.fn();
  c3.taps.thisCompilation.fn({ errors: [], warnings: [] }); // the save's compile: no report yet
  await until(() => c3.watching.invalidated >= 1);
  const comp3 = { errors: [], warnings: [] };
  c3.taps.thisCompilation.fn(comp3);
  assert.equal(comp3.errors.length, 1);
  assert.equal(c3.watching.invalidated, 1);
  c3.taps.watchClose.fn();
  rmSync(d, { recursive: true, force: true });
});

test("next: webpack gets the plugin on the client compile only; turbopack build fails before it starts", { skip }, async () => {
  const d = project();
  const prev = process.cwd();
  process.chdir(d);
  delete process.env.TURBOPACK;
  const cfg = await withCodelaw({ reactStrictMode: true, webpack: (c) => c }, { binary })("phase-development-server", {});
  assert.equal(cfg.reactStrictMode, true);
  const client = cfg.webpack({ plugins: [] }, { isServer: false });
  const server = cfg.webpack({ plugins: [] }, { isServer: true });
  assert.equal(client.plugins.length, 1);
  assert.equal(server.plugins.length, 0);

  process.env.TURBOPACK = "1";
  await assert.rejects(withCodelaw({}, { binary })("phase-production-build", {}), /js\/no-debugger/);
  delete process.env.TURBOPACK;
  process.chdir(prev);
  rmSync(d, { recursive: true, force: true });
});
