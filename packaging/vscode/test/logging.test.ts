import { test } from "node:test";
import * as assert from "node:assert/strict";
import { fromClientLine, parseLevel, renderDebugInfo, Ring } from "../src/logging";

test("the client's prefix becomes the level; a bare server line is debug", () => {
  assert.deepEqual(fromClientLine("[Error - 10:01:02 AM] request failed"), { level: "error", text: "request failed" });
  assert.deepEqual(fromClientLine("[Warn  - 10:01:02] a"), { level: "warn", text: "a" });
  assert.deepEqual(fromClientLine("check /a.js: 3 rule(s)"), { level: "debug", text: "check /a.js: 3 rule(s)" });
});

test("an unknown level is info", () => {
  assert.equal(parseLevel("debug"), "debug");
  assert.equal(parseLevel("loud"), "info");
  assert.equal(parseLevel(undefined), "info");
});

test("the ring keeps the last lines", () => {
  const r = new Ring(3);
  for (const l of ["a", "b", "c", "d"]) r.push(l);
  assert.deepEqual(r.all, ["b", "c", "d"]);
});

test("debug info carries versions, binary, candidates, settings and the log", () => {
  const text = renderDebugInfo({
    extension: "0.2.0",
    vscode: "1.90.0",
    os: "darwin arm64",
    server: "0.2.0",
    binary: { path: "/x/codelaw", source: "venv" },
    tried: [{ source: "venv", path: "/x/codelaw", verdict: "used" }],
    folders: ["/x"],
    settings: { logLevel: "info" },
    log: ["line one"],
  });
  for (const want of ["extension: 0.2.0", "vscode: 1.90.0", "darwin arm64", "/x/codelaw (venv)", "venv: /x/codelaw - used", 'codelaw.logLevel: "info"', "line one"]) {
    assert.ok(text.includes(want), want);
  }
});
