# Releasing codelaw

**One release, both registries, one version** ([L6](records/0008_LAW-6-python-and-node-packages.md)
decision 9). What ships is [SR-PACKAGES](records/0107_packages.md) §1: npm
`codelaw` + eight `@codelaw/cli-*` + eight `@codelaw/rules-*` (`js`, `ts`,
`react`, `node`, `stencil`, `html`, `css`, `data`), PyPI `codelaw` (eight
wheels) + six `codelaw-rules-*` (`js`, `react`, `html`, `css`, `python`,
`data`). Which pack goes where is `FIRST_PARTY_PACKS` in
`crates/codelaw-rules/src/pack.rs`; `python3 scripts/package.py names` prints
every name.

**Publishing is only `.github/workflows/publish.yml`**, run by a published
GitHub Release. It compiles nothing: it uploads the `dist` artifact that
`ci.yml` assembled and checked on the release commit. No token exists
anywhere — both registries use trusted publishing (OIDC).

> **The engine is complete (2026-10-21).** `format`, `mcp` and `lsp` are all in
> the binary, as [L7](records/0009_LAW-7-linter-and-formatter.md) and
> [L6](records/0008_LAW-6-python-and-node-packages.md) decision 3 require, and
> `python3 scripts/law-gate.py dist/` lists nothing. The first release still
> waits on the one-time setup below and the decider's go.

## The pipeline

| workflow | when | what | time |
|---|---|---|---|
| `ci.yml` | every push | tests, clippy, the SR-guard replay; **8 platform builds in parallel**; the 24 packages (31 files: 17 npm tarballs, 14 wheels) assembled and checked, and on `main` attested; **the install smoke on all 8 platforms** | see [W-04](work/open/W-04-release-pipeline.md) §Measured |
| `publish.yml` | a GitHub Release | the gate (CI green on this sha, on `main`; every file matches `SHA256SUMS` and ci.yml's build attestation for this commit), then npm and PyPI **in parallel**, then the checksums on the release | see W-04 §Measured |

Every check `ci.yml` runs on the packages is `scripts/package.py check`:
only the SR-PACKAGES names (B-020), one version (W-18), zero runtime
dependencies, the licence in every package, **the same binary on npm and PyPI**
for each platform, and each pack **byte-equal** on both registries.
`scripts/smoke.sh` publishes them to a **staging index** — a throwaway npm
registry and PyPI simple index on `127.0.0.1` (`scripts/staging-index.py`, the
Python standard library only) — installs `codelaw` from it by name, as a user
does (npm, npx, pip, uvx), and checks that npm (no Python on `PATH`) and PyPI
(no Node on `PATH`) give byte-equal output — on each of the eight platforms, the
musl two in a clean Alpine container. Nothing reaches a real registry before
`publish.yml`. To run it on your machine, against a `dist` built for it:
`bash scripts/smoke.sh dist` (needs node, npm and Python 3.11+).

**Hardening (W-18).** Every workflow `run:` is `bash -eo pipefail`. Every action
is pinned by commit SHA (the comment names the release; bump both together), and
the OIDC jobs install an exact npm (`NPM_VERSION` in `publish.yml`), never
`npm@latest`. `ci.yml` attests every file of `dist` on `main`; `publish.yml`
uploads nothing whose digest is not in `SHA256SUMS` and not attested by `ci.yml`
on the release commit. A version older than npm's current `latest` (a backport)
goes to the dist-tag `release-<major>.<minor>`, so `latest` stays on the newest line.

## One time, by hand (the decider)

In this order. npm binds a trusted publisher only to a package that already
exists, so the first npm publish cannot come from CI. PyPI can: a *pending*
publisher creates the project on its first upload.

**Done for npm on 2026-10-24** (steps 1–3, 0.1.0, [W-29](work/open/W-29-first-npm-publish.md)).
Steps 2–3 come back for **every pack name new to npm**: at 0.2.0,
`@codelaw/rules-ts`, `-react`, `-node` and `-stencil` ([W-31](work/open/W-31-registry-pack-split.md)) —
publish just those tarballs by hand from the 0.2.0 `dist`, then run
`npm-trust.sh` again (it skips nothing, and re-binding an existing package is
harmless); `publish.yml` then skips the versions already on npm.

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
   one per project: `codelaw` and `codelaw-rules-{js,react,html,css,python,data}`;
   owner `arpitarya`, repository `codelaw`, workflow `publish.yml`, environment
   `pypi`.
5. **GitHub environments** `npm` and `pypi` (*Settings → Environments*) —
   **required, both:**
   - *Deployment branches and tags* → **Selected branches and tags** → add one
     **tag** rule, `v*`, and **no branch rule**. A trusted publisher checks the
     repository, the workflow file and the environment, never the ref, so this
     rule is what makes a published release the only way to publish: a
     `publish.yml` run from any branch is refused at the environment.
   - Optionally, yourself as a required reviewer, to approve each publish (`npm`
     is asked twice: the trust preflight and the publish both use it).

After this, no release needs a token, a code or your machine.

## Every release

1. **Bump:** `python3 scripts/version.py --set 0.2.0` — Cargo.toml, Cargo.lock
   and the nine pack manifests; every npm and PyPI manifest is generated from
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
