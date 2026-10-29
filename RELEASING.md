# Releasing codelaw

**One release, both registries, one version** ([L6](records/0008_LAW-6-python-and-node-packages.md)
decision 9). What ships is [SR-PACKAGES](records/0107_packages.md) §1: npm
`codelaw` + eight `@codelaw/cli-*` + sixteen `@codelaw/rules-*` (`js`, `ts`,
`react`, `node`, `stencil`, `html`, `css`, `scss`, `css-modules`, `data`, `tanstack-query`,
`redux-toolkit`, `security-js`, `security-ts`, `security-html`, `spell`), PyPI `codelaw`
(eight wheels) + eleven `codelaw-rules-*` (`js`, `react`, `html`, `css`, `scss`,
`python`, `data`, `security-js`, `security-html`, `security-python`, `spell`). Which pack goes where is `FIRST_PARTY_PACKS` in
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
| `ci.yml` | every push | tests, clippy, the SR-guard replay; **8 platform builds in parallel**; the 24 packages (31 files: 17 npm tarballs, 14 wheels) assembled and checked, and on `main` attested; **the install smoke on all 8 platforms** | see [W-04](archive/open/W-04-release-pipeline.md) §Measured |
| `publish.yml` | a GitHub Release (both registries), or by hand on a `v*` tag (npm only) | the gate (CI green on this sha, on `main`; every file matches `SHA256SUMS` and ci.yml's build attestation for this commit), then npm and PyPI **in parallel**, then the checksums on the release | the first run by hand, npm only: 5 min 21 s (run 40207195373, `v0.2.0`, 2026-10-25) — 17 names published with provenance, the two hand-published skipped |

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

**The GitHub Action (W-97).** `action.yml` at the root makes every release tag
an Action: `uses: arpitarya/codelaw@v0.3.0` installs codelaw 0.3.0, because
`version.py --set` writes the Action's default `version` and `--check` fails
when it disagrees. So a release tag is never moved or re-pointed — it is the
Action's contract too ([SR-PACKAGES](records/0107_packages.md) decision 15).
`ci.yml`'s `action` job runs it on every push. Listing it on the GitHub
Marketplace is the decider's: accept the Marketplace Developer Agreement once,
then tick *Publish this Action to the GitHub Marketplace* when creating the
release (`gh release create` cannot tick it; use the web form or edit the
release after).

## One time, by hand (the decider)

In this order. npm binds a trusted publisher only to a package that already
exists, so the first npm publish cannot come from CI. PyPI can: a *pending*
publisher creates the project on its first upload.

**Done for npm on 2026-10-24** (steps 1–3, 0.1.0, [W-29](archive/open/W-29-first-npm-publish.md)). **Step 5 done 2026-10-24** (both environments, tag rule `v*`, [W-18](work/open/W-18-release-hardening.md)). Step 4 (PyPI) done 2026-10-29 for `codelaw` and the six first packs ([W-38](archive/open/W-38-pypi-0-2-0-by-hand.md)); the later packs follow W-88 and W-89.
Steps 2–3 come back for **every pack name new to npm**: the three dev-server plugins `@codelaw/vite-plugin`, `@codelaw/webpack-plugin` and `@codelaw/next` ([W-49](archive/open/W-49-dev-server-plugins.md), closed; SR-PACKAGES decision 3a), and at 0.2.0,
`@codelaw/rules-ts`, `-react`, `-node` and `-stencil` ([W-31](archive/open/W-31-registry-pack-split.md)), and
at the first release with the security packs `@codelaw/rules-security-js`, `-security-ts` and
`-security-html` ([W-53](work/open/W-53-security-packs-groundwork.md)), and at the first release with W-77
the skills package `@codelaw/skills` (`codelaw-skills-<v>.tgz`; it depends on `codelaw` at
the same version, so publish it after `codelaw` is on npm at that version), and at
the first release with W-103 the spell pack `@codelaw/rules-spell` —
publish just those tarballs by hand from the 0.2.0 `dist`, then run
`npm-trust.sh` again (a name already trusted answers E409, which counts as
trusted; `--fux-index` adds the Fux index packages); `publish.yml` then skips the versions already on npm.

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
4. **PyPI** — owner `arpitarya`, repository `codelaw`, workflow `publish.yml`,
   environment `pypi` for every project. PyPI allows **one** pending publisher per
   repository · workflow · environment, so:
   - `codelaw`: a pending publisher at <https://pypi.org/manage/account/publishing/>
     (done 2026-10-25); the first release's `publish.yml` creates the project.
   - `codelaw-rules-{js,react,html,css,python,data}`, and with W-53
     `codelaw-rules-{security-js,security-html,security-python}` (wheels
     `codelaw_rules_security_*.whl`), and with W-103 `codelaw-rules-spell`: the first version by hand,
     `twine upload dist/pypi/codelaw_rules_*.whl` from the release's `dist`
     with your account token, then a trusted publisher on each project's
     *Settings → Publishing*; `publish.yml` skips versions already uploaded.
     ([W-38](archive/open/W-38-pypi-0-2-0-by-hand.md); the decider, 2026-10-25.)
   - `codelaw-skills` (W-77, `codelaw_skills-<v>-py3-none-any.whl`): the same
     way — the first version by hand with `twine upload`, then a trusted
     publisher. Its `Requires-Dist: codelaw==<v>` means pip installs it only
     once that `codelaw` is on PyPI too.
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

**A PyPI project that does not exist yet is skipped, not failed (W-116).** The `pypi` job uploads a wheel only if its project is on PyPI or is `codelaw` (the pending publisher, step 4); every skipped project is named in the job summary. Upload those by hand (W-87–W-89, W-115) — the release after that, CI carries them.

**Cut the Release only on a commit whose `ci.yml` run has finished green on `main`.** `publish.yml`'s gate verifies every file's attestation against the release commit's sha; a Release created on a later commit fails there (v0.2.0's first run, 2026-10-28).

## The optional Fux index packages (W-34) — built, not yet published

L6 decision 12: each pack that holds rules has a companion `@codelaw/rules-<name>-fux-index` (npm, fifteen today) and `codelaw-rules-<name>-fux-index` (PyPI, ten), which `codelaw init` never installs. `ci.yml`'s `package` job builds them with a pinned fux (`FUX_VERSION` in `scripts/fux-index.py`; `python3 scripts/fux-index.py assemble --out dist`, then `check dist`; `names` prints the twenty-five) into **`dist/fux-index/`** and uploads that folder as the artifact `fux-index`. **They are not in `dist/npm` or `dist/pypi`, so `publish.yml` neither uploads nor trust-checks them.**

**The pin moves by pull request, never by hand-edit on main (W-91; D-34).** `.github/workflows/fux-bump.yml` runs daily and on `gh workflow run fux-bump.yml` (`-f version=<v>` pins an exact release instead: a rollback, or a test of the job). `python3 scripts/fux-index.py latest` compares PyPI's fux-engine with `FUX_VERSION`; on a newer one the job runs `bump <v>`, `assemble` and `check` with that fux, commits on `deps/fux-<v>` and opens one PR, or updates the open one (and closes an older `deps/fux-*` PR as superseded). It uses `GITHUB_TOKEN` only, so it dispatches `ci.yml` on the branch itself: the PR's CI is a `workflow_dispatch` run on its head commit. Merge it when green. **It needs the repository setting *Allow GitHub Actions to create and approve pull requests* (Settings → Actions → General)**; while that is off the job pushes the branch and fails naming it.

**Step 5 of W-34, the decider's hand, is open.** Per name, as the pack names were done: npm first publish by hand from the `fux-index` artifact (`gh run download <run-id> -n fux-index`; `npm publish dist/fux-index/npm/<tarball> --access public`), then `scripts/npm-trust.sh --fux-index` for the new names; PyPI by `twine upload` of the ten wheels with your token (four new projects a day), then a trusted publisher on each project. Only after that does `publish.yml` learn to upload `dist/fux-index/` (a follow-up change to its `npm` and `pypi` jobs), so a release never waits on a name nobody reserved. **Built locally instead of from CI:** `--fux <path>` takes a fux at exactly `FUX_VERSION` (`pip install fux-engine==<FUX_VERSION>` in a venv of its own); any other version is refused.

## The VS Code extension (W-95) — published by `publish.yml` from v0.3.0

SR-PACKAGES decision 6: `codelaw.codelaw` ships as eight platform `.vsix` files, each with that platform's binary as `bin/codelaw`. `ci.yml`'s `vscode` job compiles and tests `packaging/vscode`, writes them with `python3 scripts/vsix.py build --bins out/bin --out dist` (the standard library; no `vsce`) and checks them with `scripts/vsix.py check dist --bins out/bin`, into **`dist/vscode/`**, uploaded as the artifact **`vscode`** — beside `dist`, never in it, **`publish.yml`'s `vscode` job (environment `vscode`, tag rule `v*`) uploads them with `curl` after the engine**: to the Visual Studio Marketplace by trusted publishing (the GitHub OIDC token, audience `marketplace.visualstudio.com`, exchanged at `/_apis/gallery/token` for a short-lived credential — no token stored), and to Open VSX when `OVSX_PAT` is set in the environment (W-117; skipped and named in the job summary while it is not). The job is `continue-on-error`: a listing problem never fails or withdraws the engine's release — read its summary. The first release proves the upload endpoints and whether the deleted `codelaw.codelaw` can be listed again. To try one: `gh run download <run-id> -n vscode`, then `code --install-extension codelaw-<target>-<v>.vsix` (`darwin-arm64`, `linux-x64`, `alpine-x64`, `win32-x64`, …).

**The decider's one-time steps are W-95's inbox row in [`work/OPEN-WORK.md`](work/OPEN-WORK.md):** the GitHub environment `vscode` (tag rule `v*`, like `npm` and `pypi`), the Marketplace publisher `codelaw` and its token as the environment secret `VSCE_PAT`, the Open VSX namespace `codelaw` and its token as `OVSX_PAT`, and a ruling on the upload tool. `@vscode/vsce` and `ovsx` are not ratified (L11), and L2 refuses both as they stand: `ovsx` is EPL-2.0 (copyleft), and `vsce` brings `@vscode/vsce-sign` under Microsoft's proprietary licence. Only after that ruling does `publish.yml` gain its `vscode` job (W-95 step 3), after the engine's.

## Every release

0. **Re-check the npm trusted publishers.** npm lists a new trusted publisher
   as *Pending validation* until a publish uses it, with a deadline (*"Publish
   once before … to validate this configuration"*, about two days); past it,
   the configuration lapses. A name that is already on npm at the release's
   version is skipped by `publish.yml`, so nothing validates it — the Fux
   index packages above are never published by CI yet. Read each package's
   *Trusted Publisher* on npmjs.com (or let `publish.yml`'s `trust` job
   report), then re-add what lapsed: `scripts/npm-trust.sh --fux-index`. A name
   still trusted answers E409 and counts as trusted; any other failure is
   listed at the end and the script exits 1.
   ([W-85](archive/open/W-85-npm-first-publish.md).)
1. **Bump:** `python3 scripts/version.py --set 0.2.0` — Cargo.toml, Cargo.lock
   and the eighteen pack manifests; every npm and PyPI manifest is generated from
   it — the skills package's and its `codelaw-skills.json` too (W-77). Commit and push to `main`.
2. **Wait for `ci.yml` to go green on that commit, on `main`.** `publish.yml`
   refuses anything else.
3. **Publish:** `gh release create v0.2.0 --title "v0.2.0 — <what it is>"
   --notes-file notes.md`. **The notes are CHANGELOG's section for that
   version, word for word** — a bold lead line, the install commands, a
   *Before you upgrade* paragraph when anything breaks, then *Added* /
   *Changed* / *Fixed*; the title's `<what it is>` is the lead line. Write the
   section before tagging, then take it out with:
   `awk '/^## 0\.2\.0/{f=1;next} /^## /{f=0} f' CHANGELOG.md | awk 'NF||p{p=1;print}' > notes.md`.
   A version with a `-` (`0.2.0-beta.1`) goes to
   npm's `next` tag and is a PyPI pre-release.
   **npm first, PyPI later** (L6 decision 9, ruled 2026-10-25): tag the
   green commit and run `publish.yml` by hand on the tag — it publishes npm
   only. When PyPI is ready, step 3 on the same tag runs both registries and
   skips what npm already has:

   ```bash
   git tag v0.2.0 <green-commit> && git push origin v0.2.0
   gh workflow run publish.yml -R arpitarya/codelaw --ref v0.2.0   # npm only
   # … PyPI's hand uploads and trusted publishers (W-38) …
   gh release create v0.2.0 --verify-tag --title "v0.2.0 — the rule packs, one per ecosystem" --notes-file notes.md   # both; npm skipped
   ```

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
