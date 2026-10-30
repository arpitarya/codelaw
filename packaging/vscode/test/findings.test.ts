import { test } from "node:test";
import * as assert from "node:assert/strict";
import {
  actionsOf,
  CARD_COMMANDS,
  badgeOf,
  categoryOf,
  commentLeader,
  countsOf,
  DEFAULT_VIEWS,
  ERROR,
  fileBadge,
  Finding,
  FindingData,
  footerOf,
  HINT,
  iconOf,
  INFO,
  markerOf,
  missingSources,
  parseView,
  rowOf,
  SECTIONS,
  squiggleOf,
  toneOf,
  viewFor,
  WARNING,
} from "../src/findings";

function finding(data: FindingData, severity = WARNING, uri = "file:///p/src/a.ts", line = 0, message = "m"): Finding {
  return { uri, line, character: 0, endLine: line, endCharacter: 1, severity, message, rule: data.rule ?? "js/x", data };
}

test("the three views and their defaults (board L)", () => {
  assert.deepEqual(DEFAULT_VIEWS, { code: "squiggles", manifests: "badges", markers: "badges" });
  assert.equal(viewFor(categoryOf({ view: "code" }, "a.ts"), DEFAULT_VIEWS), "squiggles");
  assert.equal(viewFor(categoryOf({ view: "manifest" }, "package.json"), DEFAULT_VIEWS), "badges");
  assert.equal(viewFor(categoryOf({ view: "marker" }, "a.ts"), DEFAULT_VIEWS), "badges");
  assert.equal(viewFor("code", { ...DEFAULT_VIEWS, code: "badges" }), "badges");
  assert.equal(parseView("badges", "squiggles"), "badges");
  assert.equal(parseView("loud", "squiggles"), "squiggles");
});

test("an older server's data (no view, no kind) falls back to the file and the severity", () => {
  assert.equal(categoryOf({}, "/p/web/package.json"), "manifest");
  assert.equal(categoryOf({}, "/p/.nvmrc"), "manifest");
  assert.equal(categoryOf({}, "/p/src/a.py"), "code");
  assert.equal(iconOf({}, ERROR), "error");
  assert.equal(iconOf({}, WARNING), "warning");
  assert.equal(iconOf({ kind: "brand-new-kind" }, INFO), "info");
  assert.equal(badgeOf({ data: {}, rule: "js/x" }), "js/x");
});

test("each kind has its icon, and the tone follows the severity", () => {
  const icons: Record<string, string> = {
    vulnerable: "shield",
    deprecated: "circle-slash",
    outdated: "arrow-up",
    clean: "check",
    due: "clock",
    expired: "clock",
    flag: "flag",
    focused: "target",
    skipped: "debug-pause",
    runtime: "warning",
  };
  for (const [kind, icon] of Object.entries(icons)) assert.equal(iconOf({ kind }, ERROR), icon, kind);
  assert.equal(toneOf({ kind: "vulnerable" }, ERROR), "error");
  assert.equal(toneOf({ kind: "outdated" }, INFO), "info");
  assert.equal(toneOf({ kind: "clean" }, INFO), "ok");
  assert.equal(toneOf({ kind: "code", baseline: true }, ERROR), "muted");
  assert.equal(iconOf({ kind: "code", baseline: true }, ERROR), "circle-large-outline");
});

test("green squiggle only when codelaw fixes it on save; dotted grey for the baseline", () => {
  assert.equal(squiggleOf({ fix: "auto", fixable: true }, true), "fixOnSave");
  assert.equal(squiggleOf({ fix: "auto", fixable: true }, false), "severity");
  assert.equal(squiggleOf({ fix: "suggest", fixable: true }, true), "severity");
  assert.equal(squiggleOf({ fix: "auto", fixable: true, baseline: true }, true), "baseline");
});

test("every sidebar group's kinds are distinct", () => {
  const kinds = SECTIONS.flatMap((s) => s.groups.flatMap((g) => g.kinds));
  assert.equal(new Set(kinds).size, kinds.length);
  assert.deepEqual(SECTIONS.map((s) => s.label), ["Dependencies", "Deadlines", "Tests", "Ownership"]);
});

test("sidebar rows name the package and its target, or the marker and its date", () => {
  const dep = finding({ kind: "vulnerable", package: "lodash", version: "4.17.15", safe: "4.17.21", grade: "high" }, ERROR, "file:///p/package.json", 5);
  assert.deepEqual(rowOf(dep), { label: "lodash 4.17.15 → 4.17.21", description: "package.json:6", grade: "HIGH" });
  const todo = finding({ kind: "expired", tag: "TODO", ticket: "PAY-388", days: -29 }, ERROR, "file:///p/src/coupon.ts", 8);
  assert.deepEqual(rowOf(todo), { label: "TODO PAY-388", description: "coupon.ts:9 · 29 d ago" });
  const flag = finding({ kind: "flag", flag: "new-checkout", days: 2, owner: "@payments-team" }, WARNING);
  assert.equal(rowOf(flag).description, "a.ts:1 · in 2 d · @payments-team");
});

test("the status bar counts and the scan banner", () => {
  const c = countsOf([
    finding({ kind: "vulnerable" }),
    finding({ kind: "vulnerable" }),
    finding({ kind: "outdated" }),
    finding({ kind: "deprecated" }),
    finding({ kind: "expired" }),
    finding({ kind: "due" }),
    finding({ kind: "flag" }),
    finding({ kind: "scan" }, WARNING, "file:///p/package.json", 0, "the scan is incomplete: `osv` (HTTP 403, on 2026-10-29) unavailable"),
  ]);
  assert.deepEqual({ ...c, scan: Boolean(c.scan) }, { vulnerable: 2, outdated: 2, expired: 1, due: 2, scan: true });
  assert.deepEqual(missingSources(c.scan ?? ""), ["osv"]);
});

test("a tab's badge counts what is shown and takes the worst tone", () => {
  assert.equal(fileBadge([]), undefined);
  assert.deepEqual(fileBadge([finding({}, WARNING), finding({}, ERROR)]), { badge: "2", tone: "error" });
  assert.equal(fileBadge([finding({ baseline: true }, ERROR), finding({}, HINT)]), undefined);
  assert.equal(fileBadge(Array.from({ length: 12 }, () => finding({}, INFO)))?.badge, "9+");
});

test("each card has one primary action, as board K draws it", () => {
  const first = (d: FindingData, path = "src/a.ts", fixOnSave = false) => actionsOf(d, path, fixOnSave)[0].title;
  assert.equal(first({ kind: "security", fix: "manual" }), "Ask agent to fix");
  assert.equal(first({ kind: "code", fix: "auto", fixable: true }), "Apply fix");
  assert.equal(first({ kind: "code", fix: "auto", fixable: true }, "a.py", true), "Fix now");
  assert.equal(first({ kind: "code", baseline: true }), "Ask agent to fix");
  assert.equal(first({ kind: "vulnerable", safe: "4.17.21", fixable: true }, "package.json"), "Change to 4.17.21");
  assert.equal(first({ kind: "vulnerable", safe: "4.17.21" }, "src/a.ts"), "Change to 4.17.21…");
  assert.equal(first({ kind: "expired" }), "Change date…");
  assert.equal(first({ kind: "ticket" }), "Add ticket…");
  assert.equal(first({ kind: "flag-expired" }), "Pick a branch…");
  assert.equal(first({ kind: "focused", fixable: true }), "Apply fix");
  assert.equal(first({ kind: "skipped" }), "Add reason…");
  assert.equal(first({ kind: "complexity" }), "Ask agent to split");
  // Not to build: no "Fix both" anywhere.
  const all = ["vulnerable", "outdated", "deprecated", "code", "security", "scan"].flatMap((k) => actionsOf({ kind: k }, "package.json", false));
  assert.ok(all.every((a) => !/both/i.test(a.title)));
});

test("the footer links only trusted commands, the first in bold", () => {
  const args = { uri: "file:///a.ts", line: 1, character: 2, endLine: 1, endCharacter: 4, rule: "js/x", message: "m" };
  const md = footerOf([{ title: "Apply fix", command: "codelaw.applyFix" }, { title: "Rule page", command: "codelaw.explainRule" }], args);
  assert.ok(md.startsWith("**[Apply fix](command:codelaw.applyFix?"), md);
  assert.ok(md.includes(encodeURIComponent(JSON.stringify([args]))), md);
});

test("markers: the tag, the ticket and the date chips sit inside comments only", () => {
  assert.deepEqual(markerOf("// TODO(PAY-412, 2026-11-15): drop it"), { tag: [3, 7], ticket: [8, 15], date: [17, 27] });
  assert.deepEqual(markerOf("    # FIXME: rounding"), { tag: [6, 11] });
  assert.equal(markerOf("const TODO = 1;"), undefined);
  assert.deepEqual(markerOf("  // SKIP(PAY-390, 2026-12-01): flaky"), { tag: [5, 9], ticket: [10, 17], date: [19, 29] });
});

test("an inserted line uses the language's comment", () => {
  assert.deepEqual(commentLeader("python"), { open: "# ", close: "" });
  assert.deepEqual(commentLeader("typescript"), { open: "// ", close: "" });
  assert.deepEqual(commentLeader("css"), { open: "/* ", close: " */" });
  assert.equal(commentLeader("json"), undefined);
});

test("every codelaw colour the sources name is contributed, and the settings have the design's defaults", () => {
  const fs = require("node:fs") as typeof import("node:fs");
  const path = require("node:path") as typeof import("node:path");
  const root = path.join(__dirname, "..", "..");
  const pkg = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  const contributed = new Set<string>(pkg.contributes.colors.map((c: { id: string }) => c.id));
  const src = ["decorations.ts", "sidebar.ts", "status.ts"].map((f) => fs.readFileSync(path.join(root, "src", f), "utf8")).join("\n");
  const named = new Set<string>();
  for (const m of src.matchAll(/"(codelaw\.[a-zA-Z]+)"/g)) if (/(Foreground|Background|Border|file[A-Z]|squiggle)/.test(m[1])) named.add(m[1]);
  for (const m of src.matchAll(/`codelaw\.badge\$\{id\}(Foreground|Background|Border)`/g)) for (const t of ["Error", "Warning", "Info", "Ok", "Muted"]) named.add(`codelaw.badge${t}${m[1]}`);
  named.add("codelaw.squiggleFixOnSave");
  named.add("codelaw.squiggleBaseline");
  for (const id of named) assert.ok(contributed.has(id), `${id} is not in contributes.colors`);
  for (const c of pkg.contributes.colors) for (const theme of ["dark", "light", "highContrast", "highContrastLight"]) assert.ok(c.defaults[theme], `${c.id} has no ${theme} default`);
  const props = pkg.contributes.configuration.properties;
  assert.equal(props["codelaw.view.code"].default, "squiggles");
  assert.equal(props["codelaw.view.manifests"].default, "badges");
  assert.equal(props["codelaw.view.markers"].default, "badges");
  assert.equal(props["codelaw.sidebar"].default, true);
  assert.equal(props["codelaw.badges.clean"].default, true);
  const commands = new Set<string>(pkg.contributes.commands.map((c: { command: string }) => c.command));
  for (const id of CARD_COMMANDS) assert.ok(commands.has(id), `${id} is not contributed`);
});
