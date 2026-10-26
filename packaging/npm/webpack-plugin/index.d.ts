export interface CodelawPluginOptions {
  /** Profile when watching (`webpack serve`, `--watch`). Default `local`. */
  profile?: string;
  /** Profile for a one-off build. Default `ci`; `profile`, when set, wins. */
  buildProfile?: string;
  /** Report codelaw errors as compilation errors, which fail the build. Default true; false makes them warnings. */
  failOnError?: boolean;
  /** Paths to check, relative to the compiler context. Default `["."]`. */
  paths?: string[];
  /** The codelaw binary. Default: `CODELAW_BINARY`, the project's `codelaw`, then PATH. */
  binary?: string;
}
export class CodelawPlugin {
  constructor(options?: CodelawPluginOptions);
  apply(compiler: unknown): void;
}
export default CodelawPlugin;
