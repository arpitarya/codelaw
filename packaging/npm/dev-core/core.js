// The core the codelaw dev-server plugins share (W-49, SR-PACKAGES decision 3a).
// It never lints: it spawns the codelaw binary and reads its JSON, so a finding
// is byte-identical to the CLI's (L5). No dependencies. This file is the source
// of the `core.js` that each plugin package carries; `scripts/package.py
// sync-dev-core` copies it and `check` refuses a copy that differs.
import { spawn, spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { join } from "node:path";

/** The codelaw binary for a project: `CODELAW_BINARY`, else the project's own `codelaw` package, else `codelaw` on PATH. */
export function resolveBinary(root = process.cwd()) {
  if (process.env.CODELAW_BINARY) return process.env.CODELAW_BINARY;
  try {
    const from = createRequire(join(root, "package.json"));
    const pkg = from.resolve("codelaw/package.json");
    const key = `${process.platform}-${process.arch}`;
    const exe = process.platform === "win32" ? "codelaw.exe" : "codelaw";
    const candidates = process.platform === "linux" ? [`${key}-gnu`, `${key}-musl`] : [key];
    for (const k of candidates) {
      try {
        return createRequire(pkg).resolve(`@codelaw/cli-${k}/${exe}`);
      } catch {
        /* try the next libc */
      }
    }
  } catch {
    /* codelaw is not a dependency of the project */
  }
  return "codelaw";
}

/** `file:line:col  severity  rule  message` — the one line every host prints. */
export function formatDiagnostic(d) {
  const at = d.range?.start ? `${d.file}:${d.range.start.line}:${d.range.start.column}` : d.file;
  return `${at}  ${d.severity}  ${d.rule}  ${d.message}`;
}

/** Errors and warnings of a report; `info` findings are not shown by a dev server. */
export function partition(report) {
  const ds = report?.diagnostics ?? [];
  return { errors: ds.filter((d) => d.severity === "error"), warnings: ds.filter((d) => d.severity === "warning") };
}

/** A string that is equal for two reports that show the same findings: a run that changed nothing is not shown again. */
export function signature(report) {
  const { errors, warnings } = partition(report);
  return [...errors, ...warnings].map(formatDiagnostic).sort().join("\n");
}

/** The text of the findings, errors first. */
export function formatReport(report, { warnings = true } = {}) {
  const p = partition(report);
  return [...p.errors, ...(warnings ? p.warnings : [])].map(formatDiagnostic).join("\n");
}

/**
 * `codelaw check --watch --format json` as a child process (W-44). `onReport`
 * gets each run's report whose findings differ from the last one's; `onMessage`
 * an `error` line (an unloadable config). The child is killed when this process
 * exits (a host that ends on Ctrl-C ends through `exit`) and by `stop()`.
 */
export function startWatch({ root = process.cwd(), binary, profile = "local", paths = ["."], onReport, onMessage, onExit } = {}) {
  const bin = binary ?? resolveBinary(root);
  const child = spawn(bin, ["check", "--watch", "--format", "json", "--profile", profile, ...paths], {
    cwd: root,
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });
  let last = null;
  let buf = "";
  let stopped = false;
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    buf += chunk;
    let i;
    while ((i = buf.indexOf("\n")) >= 0) {
      const line = buf.slice(0, i).trim();
      buf = buf.slice(i + 1);
      if (!line) continue;
      let obj;
      try {
        obj = JSON.parse(line);
      } catch {
        continue;
      }
      if (obj.error !== undefined && !obj.diagnostics) {
        onMessage?.(String(obj.error));
        continue;
      }
      const sig = signature(obj);
      const changed = sig !== last;
      last = sig;
      onReport?.(obj, { changed });
    }
  });
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (t) => onMessage?.(t.trim()));
  child.on("error", (e) => onMessage?.(`could not start codelaw (${bin}): ${e.message}`));
  child.on("exit", (code) => {
    if (!stopped) onExit?.(code);
  });
  const kill = () => {
    stopped = true;
    if (child.exitCode === null && !child.killed) child.kill();
  };
  process.once("exit", kill);
  return {
    child,
    stop() {
      process.removeListener("exit", kill);
      kill();
    },
  };
}

/** One `codelaw check --profile ci` over the project, for a build (W-49 step 6). */
export function runCheck({ root = process.cwd(), binary, profile = "ci", paths = ["."] } = {}) {
  const bin = binary ?? resolveBinary(root);
  const r = spawnSync(bin, ["check", "--format", "json", "--profile", profile, ...paths], { cwd: root, encoding: "utf8", maxBuffer: 1 << 30, windowsHide: true });
  if (r.error) throw new Error(`could not run codelaw (${bin}): ${r.error.message}`);
  if (r.status !== 0 && r.status !== 1) throw new Error(`codelaw check failed (exit ${r.status}): ${(r.stderr || "").trim()}`);
  return JSON.parse(r.stdout);
}

/** The error a failing build throws: every error finding, one per line. */
export function buildError(report) {
  const { errors } = partition(report);
  return new Error(`codelaw found ${errors.length} error${errors.length === 1 ? "" : "s"}:\n${formatReport(report, { warnings: false })}`);
}
