// @codelaw/vite-plugin — codelaw's findings in Vite's overlay and terminal (W-49).
// Dev: `check --watch --profile local`; errors go to the overlay, warnings to
// the terminal. Build: one `check --profile ci`, failing on errors.
import { buildError, formatReport, partition, runCheck, startWatch } from "./core.js";

export default function codelaw(options = {}) {
  const { profile, buildProfile = "ci", failOnError = true, paths, binary } = options;
  let root = process.cwd();
  let command = "serve";
  return {
    name: "codelaw",
    configResolved(config) {
      root = config.root;
      command = config.command;
    },
    configureServer(server) {
      const log = server.config.logger;
      const w = startWatch({
        root,
        binary,
        profile: profile ?? "local",
        paths,
        onReport(report, { changed }) {
          if (!changed) return;
          const { errors, warnings } = partition(report);
          if (warnings.length) log.warn(`[codelaw]\n${formatReport({ diagnostics: warnings })}`, { timestamp: true });
          if (errors.length) {
            const first = errors[0];
            server.ws.send({
              type: "error",
              err: {
                message: `[codelaw] ${errors.length} error${errors.length === 1 ? "" : "s"}\n${formatReport({ diagnostics: errors })}`,
                stack: "",
                id: first.file,
                plugin: "codelaw",
                loc: first.range?.start ? { file: first.file, line: first.range.start.line, column: first.range.start.column } : undefined,
              },
            });
            log.error(`[codelaw]\n${formatReport({ diagnostics: errors })}`, { timestamp: true });
          } else if (warnings.length === 0) {
            log.info("[codelaw] clean", { timestamp: true });
          }
        },
        onMessage: (m) => log.warn(`[codelaw] ${m}`, { timestamp: true }),
      });
      server.httpServer?.once("close", () => w.stop());
      const close = server.close.bind(server);
      server.close = async () => (w.stop(), close());
    },
    buildStart() {
      if (command !== "build") return;
      const report = runCheck({ root, binary, profile: profile ?? buildProfile, paths });
      const { errors, warnings } = partition(report);
      if (warnings.length) this.warn(`[codelaw]\n${formatReport({ diagnostics: warnings })}`);
      if (errors.length) {
        if (failOnError) this.error(buildError(report).message);
        this.warn(`[codelaw]\n${formatReport({ diagnostics: errors })}`);
      }
    },
  };
}

export { codelaw };
