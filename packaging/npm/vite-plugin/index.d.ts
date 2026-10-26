export interface CodelawOptions {
  /** Profile for `vite` (dev). Default `local`. */
  profile?: string;
  /** Profile for `vite build`. Default `ci`; `profile`, when set, wins. */
  buildProfile?: string;
  /** Fail the build on a codelaw error. Default true. */
  failOnError?: boolean;
  /** Paths to check, relative to the Vite root. Default `["."]`. */
  paths?: string[];
  /** The codelaw binary. Default: `CODELAW_BINARY`, the project's `codelaw`, then PATH. */
  binary?: string;
}
export interface VitePlugin {
  name: string;
  [hook: string]: unknown;
}
export default function codelaw(options?: CodelawOptions): VitePlugin;
export { codelaw };
