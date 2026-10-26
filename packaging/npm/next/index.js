// @codelaw/next — `withCodelaw(nextConfig)` (W-49).
// Under webpack the config gets @codelaw/webpack-plugin's behaviour on the
// client compile. Turbopack has no webpack-plugin API, so there the watcher runs
// beside `next dev` and prints to the terminal, and `next build` runs one
// `check --profile ci` before it starts.
import { buildError, formatReport, partition, runCheck, startWatch } from "./core.js";
import { CodelawPlugin } from "./plugin.js";

const DEV = "phase-development-server";
const BUILD = "phase-production-build";
let started = false;

function turbopack() {
  return Boolean(process.env.TURBOPACK) || process.argv.some((a) => a === "--turbopack" || a === "--turbo");
}

function terminalWatch(root, options) {
  if (started) return;
  started = true;
  startWatch({
    root,
    binary: options.binary,
    profile: options.profile ?? "local",
    paths: options.paths,
    onReport(report, { changed }) {
      if (!changed) return;
      const text = formatReport(report);
      if (text) console.log(`[codelaw]\n${text}`);
      else console.log("[codelaw] clean");
    },
    onMessage: (m) => console.warn(`[codelaw] ${m}`),
  });
}

function buildCheck(root, options) {
  if (started) return;
  started = true;
  const report = runCheck({ root, binary: options.binary, profile: options.profile ?? options.buildProfile ?? "ci", paths: options.paths });
  const { errors, warnings } = partition(report);
  if (warnings.length) console.warn(`[codelaw]\n${formatReport({ diagnostics: warnings })}`);
  if (errors.length) {
    if ((options.failOnError ?? true) === true) throw buildError(report);
    console.warn(`[codelaw]\n${formatReport({ diagnostics: errors })}`);
  }
}

export function withCodelaw(nextConfig = {}, options = {}) {
  return async (phase, ctx) => {
    const base = typeof nextConfig === "function" ? await nextConfig(phase, ctx) : nextConfig;
    const root = process.cwd();
    if (turbopack()) {
      if (phase === DEV) terminalWatch(root, options);
      if (phase === BUILD) buildCheck(root, options);
      return base;
    }
    return {
      ...base,
      webpack(config, wctx) {
        const out = typeof base.webpack === "function" ? base.webpack(config, wctx) : config;
        // Next runs one webpack compiler per target; the client's carries the plugin, once.
        if (!wctx.isServer && !wctx.nextRuntime) {
          (out.plugins ??= []).push(new CodelawPlugin({ ...options, paths: options.paths }));
        }
        return out;
      },
    };
  };
}

export default withCodelaw;
