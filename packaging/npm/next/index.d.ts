export interface CodelawNextOptions {
  /** Profile for `next dev`. Default `local`. */
  profile?: string;
  /** Profile for `next build`. Default `ci`; `profile`, when set, wins. */
  buildProfile?: string;
  /** Fail `next build` on a codelaw error. Default true. */
  failOnError?: boolean;
  /** Paths to check, relative to the project. Default `["."]`. */
  paths?: string[];
  /** The codelaw binary. Default: `CODELAW_BINARY`, the project's `codelaw`, then PATH. */
  binary?: string;
}
type NextConfig = Record<string, unknown>;
export function withCodelaw(
  nextConfig?: NextConfig | ((phase: string, ctx: { defaultConfig: NextConfig; dir?: string }) => NextConfig | Promise<NextConfig>),
  options?: CodelawNextOptions,
): (phase: string, ctx: { defaultConfig: NextConfig; dir?: string }) => Promise<NextConfig>;
export default withCodelaw;
