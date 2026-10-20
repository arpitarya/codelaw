# Releasing codelaw

**One release, both registries, one version** ([L6](records/0008_LAW-6-python-and-node-packages.md)
decision 9). What ships is [SR-PACKAGES](records/0107_packages.md) §1: npm
`codelaw` + eight `@codelaw/cli-*` + five `@codelaw/rules-*`, PyPI `codelaw`
(eight wheels) + five `codelaw-rules-*`.

**Publishing is only `.github/workflows/publish.yml`**, run by a published
GitHub Release. It compiles nothing: it uploads the `dist` artifact that
`ci.yml` assembled and checked on the release commit. No token exists
anywhere — both registries use trusted publishing (OIDC).

> **Not releasable yet (2026-10-20).** The engine has no `format`, `mcp` or `lsp`
> command, and [L7](records/0009_LAW-7-linter-and-formatter.md) and
> [L6](records/0008_LAW-6-python-and-node-packages.md) decision 3 forbid shipping
> without them. `python3 scripts/law-gate.py dist/` says what is missing;
> W-07, W-08 and W-09 build them.

## The pipeline

| workflow | when | what | time |
|---|---|---|---|
| `ci.yml` | every push | tests, clippy, the SR-guard replay; **8 platform builds in parallel**; the 27 packages assembled, checked and install-smoked | see [W-04](work/open/W-04-release-pipeline.md) §Measured |
| `publish.yml` | a GitHub Release | the gate (CI green on this sha, on `main`), then npm and PyPI **in parallel**, then the checksums on the release | see W-04 §Measured |

Every check `ci.yml` runs on the packages is `scripts/package.py check`:
only the SR-PACKAGES names (B-020), one version (W-18), zero runtime
dependencies, the licence in every package, **the same binary on npm and PyPI**
for each platform, and each pack **byte-equal** on both registries.
`scripts/smoke.sh` installs them with no registry and checks that npm (no Python
on `PATH`) and PyPI (no Node on `PATH`) give byte-equal output.

## One time, by hand (the decider)

In this order. npm binds a trusted publisher only to a package that already
exists, so the first npm publish cannot come from CI. PyPI can: a *pending*
publisher creates the project on its first upload.

1. **npm:** create the `@codelaw` organisation (account with 2FA).
2. **npm, first publish by hand** — from the `dist` artifact of a green
   `ci.yml` run on `main` (`gh run download <run-id> -n dist`). **First run
   `python3 scripts/law-gate.py dist`; publish nothing unless it passes** (L7,
   L6 decision 3). Then:
   `for t in npm/*.tgz; do npm publish "$t" --access public; done`. npm opens a
   browser sign-in to approve the writes.
3. **npm trusted publishers:** `scripts/npm-trust.sh` (npm 12, your login) —
   every package trusts `arpitarya/codelaw` · `publish.yml` · environment `npm`,
   with *Allow npm publish* ticked.
4. **PyPI pending publishers** — <https://pypi.org/manage/account/publishing/>,
   one per project: `codelaw` and `codelaw-rules-{js,html,css,python,data}`;
   owner `arpitarya`, repository `codelaw`, workflow `publish.yml`, environment
   `pypi`.
5. **GitHub environments** `npm` and `pypi` (*Settings → Environments*). Add
   yourself as a required reviewer to approve each publish, if you want one
   (`npm` is asked twice: the trust preflight and the publish both use it).

After this, no release needs a token, a code or your machine.

## Every release

1. **Bump:** `python3 scripts/version.py --set 0.2.0` — Cargo.toml, Cargo.lock
   and the five pack manifests; every npm and PyPI manifest is generated from
   it. Commit and push to `main`.
2. **Wait for `ci.yml` to go green on that commit, on `main`.** `publish.yml`
   refuses anything else.
3. **Publish:** `gh release create v0.2.0 --title "v0.2.0 — <what it is>"
   --notes "<what changed>"`. A version with a `-` (`0.2.0-beta.1`) goes to
   npm's `next` tag and is a PyPI pre-release.
4. A failed job is **re-run, not re-released**: versions already on a registry
   are skipped.
5. **`dist` lives 30 days.** For an older commit, re-run its `ci.yml` push run
   first (`gh run rerun <run-id>`), then re-run the release's `publish.yml`.

## When a release fails part-way

L6 decision 9: what reached a registry is withdrawn and the version is not
announced. The `half-published` job lists exactly what got out. OIDC tokens
can publish but cannot deprecate, and PyPI has no yank API, so the withdrawal
is yours: `scripts/withdraw.sh 0.2.0` deprecates every npm package of that
version and prints each PyPI yank page.
