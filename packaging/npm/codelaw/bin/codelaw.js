#!/usr/bin/env node
// The `codelaw` command on npm: run the platform binary with the same
// arguments, stdio and exit code (SR-PACKAGES decision 3).
import { spawnSync } from "node:child_process";
import { binaryPath } from "../index.js";

let bin;
try {
  bin = binaryPath();
} catch (e) {
  console.error(e.message);
  process.exit(2);
}
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) {
  console.error(`codelaw: could not run ${bin}: ${r.error.message}`);
  process.exit(2);
}
process.exit(r.status ?? 2);
