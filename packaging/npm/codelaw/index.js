// codelaw on npm: find the platform binary and run it (SR-PACKAGES decision 3).
// No dependencies; the binary is one of the eight optionalDependencies
// `@codelaw/cli-<os>-<arch>`, of which npm installs exactly one.
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

/** The eight platforms codelaw ships a binary for (L6 decision 6). */
export const PLATFORMS = Object.freeze([
  "darwin-arm64",
  "darwin-x64",
  "linux-x64-gnu",
  "linux-arm64-gnu",
  "linux-x64-musl",
  "linux-arm64-musl",
  "win32-x64",
  "win32-arm64",
]);

function libc() {
  // glibc reports its runtime version; musl reports none.
  const header = process.report?.getReport?.().header ?? {};
  return header.glibcVersionRuntime ? "gnu" : "musl";
}

/** This machine's platform key, e.g. `linux-x64-gnu`. */
export function platformKey() {
  const base = `${process.platform}-${process.arch}`;
  return process.platform === "linux" ? `${base}-${libc()}` : base;
}

/** The absolute path of the codelaw binary for this machine. Throws on an unsupported platform (L6 decision 7). */
export function binaryPath() {
  if (process.env.CODELAW_BINARY) return process.env.CODELAW_BINARY;
  const key = platformKey();
  if (!PLATFORMS.includes(key)) {
    throw new Error(`codelaw has no binary for ${key}; supported: ${PLATFORMS.join(", ")}`);
  }
  const exe = process.platform === "win32" ? "codelaw.exe" : "codelaw";
  try {
    return require.resolve(`@codelaw/cli-${key}/${exe}`);
  } catch {
    throw new Error(
      `codelaw: @codelaw/cli-${key} is not installed. It is an optional dependency of codelaw — ` +
        `reinstall without --no-optional / --omit=optional.`,
    );
  }
}

function run(args) {
  const r = spawnSync(binaryPath(), args, { encoding: "utf8", maxBuffer: 1 << 30 });
  if (r.error) throw r.error;
  // 0: clean, 1: diagnostics over the threshold; 2: the run could not start.
  if (r.status !== 0 && r.status !== 1) {
    throw new Error(`codelaw ${args[0]} failed (exit ${r.status}): ${r.stderr.trim()}`);
  }
  return JSON.parse(r.stdout);
}

function common(o = {}) {
  const a = [];
  if (o.lang?.length) a.push("--lang", o.lang.join(","));
  if (o.rule?.length) a.push("--rule", o.rule.join(","));
  if (o.threads != null) a.push("--threads", String(o.threads));
  if (o.noCache) a.push("--no-cache");
  return a;
}

/** `codelaw check --format json`; returns the report. */
export function check(paths = [], options = {}) {
  return run(["check", "--format", "json", ...common(options), "--", ...paths]);
}

/** `codelaw fix --format json`; returns the report. */
export function fix(paths = [], options = {}) {
  const a = ["fix", "--format", "json", ...common(options)];
  if (options.unsafe) a.push("--unsafe");
  if (options.dryRun) a.push("--dry-run");
  return run([...a, "--", ...paths]);
}
