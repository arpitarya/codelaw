// @codelaw/webpack-plugin — codelaw's findings in webpack's (and Rspack's) errors
// and warnings (W-49). Rspack runs this plugin unchanged: it uses only the
// hooks and `compilation.errors` / `compilation.warnings` that both have.
// Watch: `check --watch --profile local`; build: one `check --profile ci`.
import { formatDiagnostic, partition, runCheck, signature, startWatch } from "./core.js";

const NAME = "CodelawPlugin";

function toError(compiler, d) {
  const WebpackError = compiler.webpack?.WebpackError ?? Error;
  const e = new WebpackError(formatDiagnostic(d));
  e.name = "CodelawError";
  e.file = d.file;
  return e;
}

export class CodelawPlugin {
  constructor(options = {}) {
    this.options = options;
  }

  apply(compiler) {
    const { profile, buildProfile = "ci", failOnError = true, paths, binary } = this.options;
    const root = compiler.context ?? process.cwd();
    let report = null; // the latest report
    let injected = ""; // the signature the last compilation carried
    let watcher = null;

    const inject = (compilation) => {
      if (!report) return;
      const { errors, warnings } = partition(report);
      for (const d of warnings) compilation.warnings.push(toError(compiler, d));
      for (const d of errors) (failOnError ? compilation.errors : compilation.warnings).push(toError(compiler, d));
      injected = signature(report);
    };

    compiler.hooks.run.tap(NAME, () => {
      report = runCheck({ root, binary, profile: profile ?? buildProfile, paths });
    });

    compiler.hooks.watchRun.tap(NAME, () => {
      if (watcher) return;
      watcher = startWatch({
        root,
        binary,
        profile: profile ?? "local",
        paths,
        onReport(r) {
          report = r;
          // A report that arrives after the save's own compile: compile again with it.
          if (signature(r) !== injected) compiler.watching?.invalidate();
        },
        onMessage: (m) => console.warn(`[codelaw] ${m}`),
      });
    });

    compiler.hooks.thisCompilation.tap(NAME, inject);
    for (const hook of ["watchClose", "shutdown"]) compiler.hooks[hook]?.tap(NAME, () => watcher?.stop());
  }
}

export default CodelawPlugin;
