// SR-EDITOR decision 4: codelaw.path, then node_modules/.bin, then
// .venv/bin (or the selected interpreter's), then the bundled binary.

import { test } from "node:test";
import * as assert from "node:assert/strict";
import { findBinary, Lookup } from "../src/binary";

function lookup(present: string[], over: Partial<Lookup> = {}): Lookup {
  const set = new Set(present);
  return { folders: ["/repo"], extensionDir: "/ext", platform: "linux", exists: (f) => set.has(f), ...over };
}

const ALL = ["/repo/node_modules/.bin/codelaw", "/repo/.venv/bin/codelaw", "/py/bin/codelaw", "/ext/bin/codelaw"];

test("the project's npm codelaw comes first", () => {
  assert.deepEqual(findBinary(lookup(ALL, { python: "/py/bin/python" })), { path: "/repo/node_modules/.bin/codelaw", source: "node_modules" });
});

test("then the project's .venv", () => {
  assert.deepEqual(findBinary(lookup(ALL.slice(1))), { path: "/repo/.venv/bin/codelaw", source: "venv" });
});

test("then the selected Python interpreter's", () => {
  assert.deepEqual(findBinary(lookup(ALL.slice(2), { python: "/py/bin/python" })), { path: "/py/bin/codelaw", source: "python" });
});

test("then the bundled binary", () => {
  assert.deepEqual(findBinary(lookup(ALL.slice(2))), { path: "/ext/bin/codelaw", source: "bundled" });
});

test("codelaw.path wins over everything, relative to the first folder", () => {
  assert.deepEqual(findBinary(lookup(ALL, { configured: "tools/codelaw" })), { path: "/repo/tools/codelaw", source: "codelaw.path" });
  assert.deepEqual(findBinary(lookup(ALL, { configured: "/opt/codelaw" })), { path: "/opt/codelaw", source: "codelaw.path" });
});

test("none anywhere is undefined", () => {
  assert.equal(findBinary(lookup([])), undefined);
});

test("Windows names: .cmd shim, Scripts, .exe", () => {
  const win = (present: string[]) => lookup(present, { folders: ["C:\\repo"], extensionDir: "C:\\ext", platform: "win32" });
  assert.equal(findBinary(win(["C:\\repo\\node_modules\\.bin\\codelaw.cmd"]))?.path, "C:\\repo\\node_modules\\.bin\\codelaw.cmd");
  assert.equal(findBinary(win(["C:\\repo\\.venv\\Scripts\\codelaw.exe"]))?.path, "C:\\repo\\.venv\\Scripts\\codelaw.exe");
  assert.equal(findBinary(win(["C:\\ext\\bin\\codelaw.exe"]))?.source, "bundled");
});

import { traceBinary } from "../src/binary";

test("the trace names each candidate and why it was or was not used", () => {
  const t = traceBinary(lookup(["/repo/.venv/bin/codelaw", "/ext/bin/codelaw"], { python: "/py/bin/python" }));
  assert.deepEqual(t.found, { path: "/repo/.venv/bin/codelaw", source: "venv" });
  assert.deepEqual(
    t.tried.map((x) => [x.source, x.verdict]),
    [
      ["node_modules", "missing"],
      ["venv", "used"],
      ["python", "skipped: an earlier one was used"],
      ["bundled", "skipped: an earlier one was used"],
    ],
  );
});

test("the trace of nothing found lists every miss; codelaw.path is used even when absent", () => {
  const none = traceBinary(lookup([]));
  assert.equal(none.found, undefined);
  assert.deepEqual(none.tried.map((x) => x.verdict), ["missing", "missing", "missing"]);
  const set = traceBinary(lookup([], { configured: "/opt/codelaw" }));
  assert.equal(set.found?.source, "codelaw.path");
  assert.equal(set.tried[0].verdict, "used (not found on disk)");
});
