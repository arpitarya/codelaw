#!/usr/bin/env node
// W-70: the dev-server plugins on the REAL hosts. Usage:
//   node examples/dev-servers/run.mjs <vite|webpack|rspack|next|next-turbopack> --dist dist
// It copies examples/dev-servers/<host> into a temp directory, installs the host
// THERE (L11: no host is in any manifest or lockfile of this repo — the example
// folders carry no package.json), installs the plugin and `@codelaw/rules-js`
// from the release tarballs in <dist>/npm, starts the dev server, saves a file
// holding a `debugger`, and times the save to the finding. Node >= 22, no deps.
// Pins: the HOSTS table below; CODELAW_EXAMPLE_<HOST>=<range> overrides one.
import { spawn, spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const HOSTS = {
  vite: { dir: "vite", deps: { vite: "^7" }, plugin: "vite-plugin", bin: ["vite", "--port", "5173", "--strictPort"], url: "http://127.0.0.1:5173/", ws: { port: 5173, protocol: "vite-hmr", match: (m) => m.type === "error" } },
  webpack: { dir: "webpack", deps: { webpack: "^5", "webpack-cli": "^6", "webpack-dev-server": "^5" }, plugin: "webpack-plugin", bin: ["webpack", "serve", "--port", "8081"], url: "http://127.0.0.1:8081/", ws: { port: 8081, path: "/ws", match: (m) => m.type === "errors" || m.type === "warnings" } },
  rspack: { dir: "rspack", deps: { "@rspack/core": "^1", "@rspack/cli": "^1" }, plugin: "webpack-plugin", bin: ["rspack", "serve", "--port", "8082"], url: "http://127.0.0.1:8082/", ws: { port: 8082, path: "/ws", match: (m) => m.type === "errors" || m.type === "warnings" } },
  // Next.js under webpack: the compile prints the finding in the terminal; Turbopack has no webpack-plugin API, the plugin's own watcher prints it.
  next: { dir: "next", deps: { next: "^15", react: "^19", "react-dom": "^19" }, plugin: "next", bin: ["next", "dev", "--port", "3000"], url: "http://127.0.0.1:3000/" },
  "next-turbopack": { dir: "next", deps: { next: "^15", react: "^19", "react-dom": "^19" }, plugin: "next", bin: ["next", "dev", "--turbopack", "--port", "3001"], url: "http://127.0.0.1:3001/" },
};

const [name, ...rest] = process.argv.slice(2);
const host = HOSTS[name];
if (!host) {
  console.error(`usage: run.mjs <${Object.keys(HOSTS).join("|")}> [--dist dist] [--budget-ms 1000]`);
  process.exit(2);
}
const flag = (f, d) => (rest.includes(f) ? rest[rest.indexOf(f) + 1] : d);
const dist = resolve(flag("--dist", "dist"));
const budget = Number(flag("--budget-ms", "1000"));
const sh = (cmd, args, opts = {}) => {
  const r = spawnSync(cmd, args, { stdio: "inherit", ...opts });
  if (r.status !== 0) throw new Error(`${cmd} ${args.join(" ")} exited ${r.status}`);
};
const tgz = (stem) => {
  const re = new RegExp(`^${stem}-\\d.*\\.tgz$`);
  const f = readdirSync(join(dist, "npm")).find((n) => re.test(n));
  if (!f) throw new Error(`no ${stem}-<version>.tgz in ${join(dist, "npm")}`);
  return join(dist, "npm", f);
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const work = mkdtempSync(join(tmpdir(), `codelaw-example-${name}-`));
let child;
let ok = false;
try {
  const app = join(work, "app");
  cpSync(join(here, host.dir), app, { recursive: true });
  const deps = Object.fromEntries(Object.entries(host.deps).map(([k, v]) => [k, process.env[`CODELAW_EXAMPLE_${k.replace(/\W/g, "_").toUpperCase()}`] ?? v]));
  writeFileSync(join(app, "package.json"), JSON.stringify({ name: "codelaw-example", private: true, type: "module", dependencies: deps }, null, 2));
  // the binary: out of the linux-x64-gnu CLI tarball of the same run (L6 decision 2)
  const unpack = join(work, "bin");
  sh("mkdir", ["-p", unpack]);
  sh("tar", ["-xzf", tgz("codelaw-cli-linux-x64-gnu"), "-C", unpack]);
  const binary = join(unpack, "package", "codelaw");
  if (!existsSync(binary)) throw new Error(`no binary at ${binary}`);
  sh("npm", ["install", "--no-audit", "--no-fund", "--no-package-lock", tgz(`codelaw-${host.plugin}`), tgz("codelaw-rules-js")], { cwd: app });
  const env = { ...process.env, CODELAW_BINARY: binary, CODELAW_CACHE_DIR: join(work, "cache"), NEXT_TELEMETRY_DISABLED: "1", FORCE_COLOR: "0", CI: "1" };

  let out = "";
  child = spawn(join(app, "node_modules", ".bin", host.bin[0]), host.bin.slice(1), { cwd: app, env, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout.on("data", (d) => (out += d));
  child.stderr.on("data", (d) => (out += d));
  const waitFor = async (pred, ms, what) => {
    const t0 = Date.now();
    while (!pred()) {
      if (Date.now() - t0 > ms) throw new Error(`timed out (${ms} ms) waiting for ${what}\n--- host output ---\n${out}`);
      await sleep(25);
    }
  };
  // up: the server answers, and the plugin has printed its first (clean) report
  await waitFor(() => /\[codelaw\]/.test(out) || /ready|Local:|compiled|Ready/i.test(out), 180_000, "the dev server to start");
  for (let i = 0; i < 1200; i++) {
    try {
      if ((await fetch(host.url)).status < 500) break;
    } catch { /* not listening yet */ }
    await sleep(100);
  }
  await sleep(1500); // let the watcher settle on the clean tree

  // overlay channel: the dev server's own websocket, where it has one
  let overlay = null;
  if (host.ws) {
    const u = `ws://127.0.0.1:${host.ws.port}${host.ws.path ?? "/"}`;
    const ws = host.ws.protocol ? new WebSocket(u, host.ws.protocol) : new WebSocket(u);
    ws.onmessage = (e) => {
      try {
        const m = JSON.parse(e.data);
        if (host.ws.match(m) && /debugger/i.test(e.data) && overlay === null) overlay = performance.now();
      } catch { /* not JSON */ }
    };
    await new Promise((res, rej) => ((ws.onopen = res), (ws.onerror = () => rej(new Error(`websocket ${u} failed`)))));
  }

  const before = out.length;
  const t0 = performance.now();
  writeFileSync(join(app, "src", "main.js"), 'export const hello = "hello";\ndebugger;\n');
  const sawTerminal = () => /debugger/i.test(out.slice(before));
  let terminal = null;
  const poll = setInterval(() => terminal === null && sawTerminal() && (terminal = performance.now()), 5);
  await waitFor(() => (host.ws ? overlay !== null && terminal !== null : terminal !== null), 15_000, "the finding");
  clearInterval(poll);
  const tOverlay = host.ws ? overlay - t0 : null;
  const tTerminal = terminal - t0;
  console.log(`${name}: finding in the terminal after ${tTerminal.toFixed(0)} ms${host.ws ? `, in the overlay's websocket after ${tOverlay.toFixed(0)} ms` : ""} (${process.platform}-${process.arch}, budget ${budget} ms)`);
  const worst = Math.max(tTerminal, tOverlay ?? 0);
  if (worst > budget) throw new Error(`${name}: ${worst.toFixed(0)} ms is over the ${budget} ms budget`);
  ok = true;
} catch (e) {
  console.error(`✗ ${e.message}`);
} finally {
  child?.kill("SIGTERM");
  await sleep(300);
  rmSync(work, { recursive: true, force: true });
}
process.exit(ok ? 0 : 1);
