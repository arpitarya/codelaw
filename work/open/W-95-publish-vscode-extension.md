---
type: Handoff
name: W-95
description: "Arpit, 2026-10-27 (W-36 gap G2): the VS Code extension on the Visual Studio Marketplace (done) and Open VSX (was W-117: uploaded, inactive until the namespace claim is approved). Merged 2026-10-30."
item: W-95
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: arpit
---

# W-95 — The editor extension on both stores

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Package `packaging/vscode` per platform with its bundled binary (or the npm binary lookup); `vsce package` / `ovsx` dry runs in CI | publishable artifacts | ✅ 2026-10-27 — eight `.vsix` built and checked in CI by `scripts/vsix.py`, without `vsce` (see below); no `vsce`/`ovsx` dry run: neither is ratified |
| 2 | Arpit creates the Marketplace publisher and the Open VSX namespace, and stores the tokens (or a trusted-publishing equivalent) as GitHub secrets in the `vscode` environment | the right to publish | ✅ 2026-10-29 — publisher `codelaw`; a **Trusted Publishing** policy *codelaw publish.yml* (GitHub Actions · `arpitarya` #2118113 · `codelaw` #1424363292 · `publish.yml` · environment `vscode`), Active — no token, no Azure. Open VSX: W-117 |
| 3 | `publish.yml` publishes the extension on a Release, after the engine | one release, every surface | ✅ wired 2026-10-29 — `publish.yml` `vscode` job, `curl` uploads (ruled 2026-10-27); Marketplace by trusted publishing, Open VSX once W-117 sets `OVSX_PAT`; the environment `vscode` exists. Proven by the v0.3.0 Release |
| 4 | README / USAGE: install from the Marketplace; the preview note drops "the editor extension's Marketplace listing comes later" | users can install it | ✅ 2026-10-30 — README's badge and 0.3.0 lines → 0.4.0, the preview note and roadmap row point at the Marketplace listing; USAGE's *Editor* line names the extension and `code --install-extension codelaw.codelaw-vscode`; Open VSX "follows" (W-117) |
| W-117 | Open VSX for the codelaw extension — merged in; its steps under [§ W-117](#merged-in-w-117) | one item, not two | ⏳ — see § |

**Merged 2026-10-30 (Arpit, text):** *"There are too many work items. See if we can review all the work items and merge them into few of them."* → *"go ahead, merge them."* Merged in: W-117.

**Arpit, 2026-10-29 (voice docket; written on *"file it."*):** recommended Open VSX now (a GitHub login, no Azure) and the Visual Studio Marketplace by hand on the v0.3.0 release day: *"Yes."* Asked to set Open VSX up in the browser, then: *"That's okay. Let's put it as a work item and we'll work on it."* — [W-117](W-95-publish-vscode-extension.md). The Marketplace token and CI automation for it wait for a later ruling (global PATs end 2026-12-01).

**Arpit, 2026-10-27 (voice docket; written 2026-10-28):** *"A. A."* — option (a): `publish.yml` uploads the built `.vsix` files with `curl` to each registry's API. `@vscode/vsce` and `ovsx` are not ratified, and L2 is not amended.

**Arpit, 2026-10-27 (voice):** *"Yes to all four."* — on W-36's R1 (README fixes), R2 (file gaps G1–G8), R3 (reopen type-aware TS) and R4 (run the with/without-Fux experiment). [Proposal](../proposals/linter-for-ai-positioning.md).

**Model:** Sonnet.

Why: Biome, Ruff and oxc each ship first-party extensions (VS Code, Zed, JetBrains); codelaw's is built and unpublished ([proposal](../proposals/linter-for-ai-positioning.md) §3).

## What was built (2026-10-27, Claude Code)

- **The manifest is listing-ready:** `packaging/vscode/package.json` gained `icon`, `preview: true`, `pricing`, `homepage`, `bugs`, `galleryBanner`, `keywords`, and a user-facing `description`. Publisher `codelaw`, name `codelaw` — the id `codelaw.codelaw` SR-EDITOR decision 4 names, so the publisher the decider creates must be **`codelaw`**.
- **`packaging/vscode/icon.png`** — 256×256, `[ ✓ ]` on a dark square; a placeholder the decider may replace (any PNG ≥ 128×128).
- **The README is the listing page:** every link absolute (a relative one breaks on the Marketplace), *Developing* moved to the end. The licence is the repository's `LICENSE`, copied in at build time as `LICENSE.txt` — one source.
- **`scripts/vsix.py build | check`** (SR-PACKAGES decision 6) — writes the eight platform `.vsix` files with the standard library (zip, `extension.vsixmanifest` with `TargetPlatform`, `[Content_Types].xml`), deterministically, as `package.py` writes the npm tarballs; ~300 KiB each before the binary. The targets are VS Code's: `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`, `alpine-x64`, `alpine-arm64`, `win32-x64`, `win32-arm64`. `check` refuses a missing platform, another platform's binary, a binary that is not the build's, sources or dev dependencies, a wrong identity or version, a foreign licence, an icon under 128 px, a relative README link, a bad `SHA256SUMS`; `--release` also refuses a pre-release version (the Marketplace takes `x.y.z` only).
- **`ci.yml` job `vscode`** (needs `build`): `npm ci && npm test` in `packaging/vscode` (the extension's tests had run nowhere in CI), `vsix.py build`, `vsix.py check --bins`, artifact **`vscode`** (30 days). Beside `dist`, never in it: `publish.yml` is unchanged.
- **Verified locally:** `npm test` 13/13; `vsix.py build` + `check --release` green against stand-in binaries for the eight keys; two builds byte-identical (same `SHA256SUMS`); **VS Code 1.143.1 installed the `darwin-arm64` `.vsix`** (`code --extensions-dir <tmp> --install-extension …` → *successfully installed* as `codelaw.codelaw` 0.2.0, `bin/codelaw` executable). A local install does not read `TargetPlatform` (VS Code installed the `linux-x64` one too); the registries do, on upload.

## Found — L2 refuses both upload tools as they stand

- **`ovsx` 1.3.0** is **EPL-2.0** — weak copyleft, which [L2](../../records/0004_LAW-2-foss-dependencies.md) decision 2 names and refuses. It also depends on `@vscode/vsce` `^4.0.0`.
- **`@vscode/vsce` 4.0.0** is MIT, but depends on **`@vscode/vsce-sign` 2.1.0**, under *"MICROSOFT SOFTWARE LICENSE TERMS — MICROSOFT VSCE-SIGN"* (use only with Microsoft products; no reverse engineering) — proprietary, refused by L2 decision 3. 26 direct dependencies (`@azure/identity`, `@secretlint/*`, `azure-devops-node-api`, `@napi-rs/keyring`, …).
- Neither is in L11's list. So the ask is not only *ratify*: ratifying them means amending L2 for a CI-only scope. **Recommended instead: no dependency.** The `.vsix` files already exist, so publishing is two HTTP uploads with `curl` from `publish.yml` — Open VSX's documented `POST https://open-vsx.org/api/-/publish` with the token, and the Marketplace's gallery upload that `vsce publish` itself calls (`POST …/_apis/gallery/extensions` for the first version, `PUT …/_apis/gallery/publishers/codelaw/extensions/codelaw` after, Basic auth with the PAT) — verified against the live API on the first publish (step 3).

## Step 3 — the publish job, drafted (wired in only after the ruling)

```yaml
  vscode:
    # W-95: the eight .vsix ci.yml built (artifact `vscode`) to the Marketplace and Open VSX, after the engine.
    needs: [gate, npm]
    if: github.event_name == 'release'
    runs-on: ubuntu-latest
    timeout-minutes: 10
    permissions: { actions: read }
    environment:
      name: vscode # tag rule v*; holds VSCE_PAT and OVSX_PAT
      url: https://marketplace.visualstudio.com/items?itemName=codelaw.codelaw
    steps:
      - uses: actions/download-artifact@37930b1c2abaa49bbe596cd826c3c89aef350131 # v7.0.0
        with: { name: vscode, path: dist/vscode/, run-id: '${{ needs.gate.outputs.run }}', github-token: '${{ github.token }}' }
      - run: (cd dist/vscode && sha256sum --strict --quiet -c SHA256SUMS)
      - name: Open VSX, then the Marketplace — one .vsix per platform; a version already there is skipped
        env: { OVSX_PAT: '${{ secrets.OVSX_PAT }}', VSCE_PAT: '${{ secrets.VSCE_PAT }}' }
        run: |
          for f in dist/vscode/*.vsix; do
            # (recommended) curl to each registry's upload endpoint, as above; or, if L2 is amended and both are ratified:
            # npx --yes ovsx@<v> publish "$f" -p "$OVSX_PAT" --skip-duplicate
            # npx --yes @vscode/vsce@<v> publish --packagePath "$f" -p "$VSCE_PAT" --skip-duplicate
            :
          done
```

`half-published` (in `publish.yml`) would name the extension too; neither registry can unpublish one version by API, so a failed listing is withdrawn by hand (Marketplace *Manage* → Unpublish; Open VSX: an issue at EclipseFdn/open-vsx.org).

## Progress — 2026-10-28 (Arpit, with Cowork)

- **Marketplace publisher `codelaw` created** (marketplace.visualstudio.com/manage/publishers/codelaw), so `codelaw.codelaw` is his.
- **The Marketplace token is stuck:** creating an Azure DevOps organization through the Azure portal fails for his personal account (*"Selected user account does not exist in tenant 'Microsoft Services'"*); the direct route asks for an Azure subscription, and his SBI card was declined.
- **The token type is retiring:** Azure DevOps global PATs (*All accessible organizations*, which the Marketplace needs) stop on **2026-12-01** ([Microsoft](https://devblogs.microsoft.com/devops/retirement-of-global-personal-access-tokens-in-azure-devops)); VS Code's docs now point to Entra ID with workload identity federation, which needs an Azure subscription too ([VS Code docs](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)).
- **He asked to publish by hand for now** (voice, 2026-10-28: *"I want to publish the extension manually for now."*): the publisher page's *New extension → Visual Studio Code* takes a `.vsix`. Open, put to him: a `.vsix` from `main` ships code newer than npm's 0.2.0 (L6), so the first hand upload is best made from the next release's `vscode` artifact; whether the page takes platform-specific `.vsix` files is unverified.

## Progress — 2026-10-29: the Marketplace has trusted publishing

- **The Visual Studio Marketplace now offers Trusted Publishing from GitHub Actions** (publisher page → *Trusted Publishing*). Arpit created the policy **codelaw publish.yml** — owner `arpitarya` (#2118113), repository `codelaw` (#1424363292), workflow `publish.yml`, environment `vscode` — shown **Active**, last used *Never* (his screenshot, 01:03 IST). This replaces the PAT and the Azure DevOps organization: the 2026-12-01 retirement of global PATs no longer touches codelaw.
- **Arpit, 2026-10-29 (voice):** *"Now, I already published 0.2.0, then I unpublished it and I deleted it. Now, I want the package or VS Code. Visual Studio Code extension to be published through CI/CD pipeline. How do I put it as trusted publishing?"* — the deleted 0.2.0 is not verified: whether `codelaw.codelaw` can be published again after a delete is open until the first CI upload.
- **Left for an agent:** (1) the GitHub environment `vscode` (`gh api -X PUT repos/arpitarya/codelaw/environments/vscode`, tag rule `v*`, with Arpit's OK — W-117 step 1 is the same environment); (2) `publish.yml`'s job — `id-token: write` in environment `vscode`, the GitHub OIDC token exchanged for a Marketplace token per Microsoft's *Learn more* page, then the eight `.vsix` uploaded with `curl` (no `vsce`, L2/L11); (3) the first release proves it, including the deleted-name question.

**Found 2026-10-29 01:15 (Cowork):** the GitHub environment `vscode` exists (with `npm`, `pypi`); the `publish.yml` `vscode` job landed (`815f327`). Left: the first release run (W-116) proves the Marketplace upload and the deleted-0.2.0 question; Open VSX waits on `OVSX_PAT` (W-117); README/USAGE step 4.

**State, 2026-10-29 10:05 (Cowork):** v0.3.0's `vscode` job got a Marketplace token through **trusted publishing** — the OIDC exchange works — but the upload was refused (HTTP 400: the gallery wants `application/json`); fixed on `main` (`216bf43`). `publish.yml` can now run the extension's job alone from `main` (`390c44b`): `gh workflow run publish.yml -R arpitarya/codelaw --ref main -f only=vscode`. Open VSX was skipped (no `OVSX_PAT`, W-117). Whether the deleted 0.2.0 blocks the name is still unknown — the next run answers it.

**Arpit, 2026-10-29 (voice; written on *"file it."*):** asked for a package to check whether the extension is published; told `@vscode/vsce` needs an L2 exception and an L11 row: *"Let's add it as an exception. And only to be used for checking the VS Code extension and publishing the VS Code extension. That's it, nothing else."* Then on `ovsx`: *"Yes."* — ruled as L2 decision 8 (scoped exception), L11 D-46 and D-47. **So this item now:** move `publish.yml`'s extension job from `curl` to `vsce publish` (Marketplace — with its trusted-publishing credential, as the policy already set up allows) and `ovsx publish` (Open VSX — the first version with W-117's token, then `--trusted-publishing`); add the check command (`vsce show codelaw.codelaw`, and `ovsx`'s for Open VSX) — the only other use allowed; pin both, record their trees in D-46/D-47, and let `cargo test -p codelaw-cli --test licences` name them as decision 8's exception, not a breach. *Agent's reading, flagged:* switching the job to the tools follows from *"publishing the VS Code extension"* in the ruling; the drafted run of the `curl` job (the earlier inbox row) is dropped.

**Found 2026-10-29 21:55 (Arpit, by hand):** a manual *New extension* upload on the Marketplace answers *"The extension 'codelaw' already exists in the Marketplace. Please use a different 'name' in the package.json file"*, while the publisher's *Extensions* list is **empty**. The 0.2.0 Arpit published, unpublished and **removed** earlier holds the name: since the 2025 name-reuse reports, a removed extension's name stays blocked and *"Only Microsoft Marketplace Support can release a deleted extension name"* (Microsoft Q&A, April 2026). The id `codelaw.codelaw` (SR-EDITOR decision 4) is therefore blocked on the Marketplace until Support releases it — or a new `name` changes the id. Open VSX is unaffected. Put to Arpit.

**Arpit, 2026-10-29 21:57 (text):** *"write the email. You go ahead and write the email and send it out. In the meanwhile, let's go ahead and create the extension under a different name, so at least we can publish it."* — **(1) sent:** an email from arpitarya.me@gmail.com to vsmarketplace@microsoft.com asking Support to release the name `codelaw` for publisher `codelaw` (or restore the removed extension); **(2) ruled:** publish under a different name now — **`codelaw-vscode`**, id `codelaw.codelaw-vscode` (the name Claude proposed; *agent's choice, flagged*). For an agent, records first (SR-EDITOR decision 4, its ruled line): `packaging/vscode/package.json` `name`, `scripts/vsix.py`'s identity check, `publish.yml`'s extension job, `codelaw init`'s `.vscode/extensions.json` recommendation (SR-EDITOR decision 6, SR-CONSUMER-LAYOUT), RELEASING.md, CHANGELOG, README/USAGE; Open VSX takes the same id (one `package.json`). Then the job (with `vsce`/`ovsx`, D-46/D-47) publishes it.

**Built 2026-10-29 22:05 (Claude Code):** the rename — `packaging/vscode/package.json` (and its lock) `name` `codelaw-vscode`; `publish.yml`'s Marketplace URL and gallery path (`…/publishers/codelaw/extensions/codelaw-vscode`); SR-EDITOR decision 4 and 6, SR-PACKAGES, RELEASING.md, CHANGELOG. `vsix.py` reads the id from the manifest, so it needed only its docstring; `codelaw init` writes no `.vscode/extensions.json` yet (decision 6 says it does — not built, unchanged here). The eight `codelaw-vscode-<platform>-0.3.0.vsix` files are in `dist/vscode/`, built by `vsix.py build` from the v0.3.0 CI binaries (taken out of that run's `.vsix` files); `check --release --bins` green; VS Code installs the `darwin-arm64` one as `codelaw.codelaw-vscode@0.3.0`, its binary `codelaw 0.3.0`. **Next:** Arpit uploads them on the Marketplace by hand (the first as *New extension*, the other seven as that version's platform packages); then `vsce`/`ovsx` in `publish.yml` (D-46, D-47).

**Arpit, 2026-10-29 22:05 (text) — the display name:** a manual upload of `codelaw-vscode` answered *"This extension display name is taken. Please try a different one."* — the removed 0.2.0 also holds `displayName` `codelaw`, which the Marketplace keeps unique. Ruled: *"codelaw - Linter & Formatter"*, then — *"add ai n spell as well suggest more names"* — *"codelaw - Rules for AI Agents: Lint, Format, Spell"* (22:06). For an agent: `packaging/vscode/package.json` `displayName` → `codelaw - Rules for AI Agents: Lint, Format, Spell`, `scripts/vsix.py`'s manifest check if it reads it, SR-EDITOR's ruled line; then a fresh `.vsix` from CI (the display name is inside the file).

**Built 2026-10-29 22:12 (Claude Code):** `displayName` `codelaw - Rules for AI Agents: Lint, Format, Spell` (`vsix.py` writes and checks it from the manifest, unchanged); the eight `codelaw-vscode-<platform>-0.3.0.vsix` rebuilt into `dist/vscode/` from v0.3.0's CI binaries; `check --release --bins` green; VS Code installs `codelaw.codelaw-vscode@0.3.0` with that display name. **Next:** Arpit uploads them.

**Listed, 2026-10-29 (checked 23:30 IST, Cowork):** Arpit published `codelaw.codelaw-vscode` by hand — the Marketplace answers *"codelaw - Rules for AI Agents: Lint, Format, Spell"*, flags `validated, public, preview`, first published 2026-10-29 16:46 UTC (22:16 IST); its page is <https://marketplace.visualstudio.com/items?itemName=codelaw.codelaw-vscode>. **Only one of eight platforms is there: 0.3.0 `darwin-arm64`** — VS Code on Windows, Linux and Intel Macs finds no build to install. Left: the other seven (the next release's `publish.yml` job, or by hand with *Update* from `dist/vscode/`), then Open VSX (W-117), README/USAGE's install line.

**Published by CI, 2026-10-30:** v0.4.0's `publish.yml` (40932853880) uploaded all eight `codelaw-vscode-<platform>-0.4.0.vsix` to the Marketplace by trusted publishing (✓ each); Open VSX skipped, no `OVSX_PAT` (W-117).

## Left

**2026-10-30 (Claude Code):** step 4 done (README, USAGE). What remains is Open VSX (W-117, waiting on the namespace claim) and the DoD's by-hand install in a clean VS Code and VSCodium — VS Code can be checked now; VSCodium needs Open VSX.

*The list below is the 2026-10-27 plan, kept as written; items 1–3 are done (steps 2–3 above).*


1. **The decider (step 2, the inbox row):** the GitHub environment `vscode` with a `v*` tag rule; the Marketplace publisher `codelaw` and a PAT → secret `VSCE_PAT`; the Open VSX account, Eclipse publisher agreement, a token → secret `OVSX_PAT`, and the namespace `codelaw`; **and the ruling: (a) own `curl` uploads, no dependency (recommended), or (b) amend L2 and ratify `@vscode/vsce` 4.0.0 and `ovsx` 1.3.0, CI-only.**
2. **Then an agent (step 3):** the job above into `publish.yml`, its upload verified on the first Release; `half-published` taught the extension; RELEASING.md's section turned from "built" to "published".
3. **Step 4:** README / USAGE install lines; the preview note's "Marketplace listing comes later" dropped.
4. **DoD, by hand on the first listing:** install from each registry in a clean VS Code and a clean VSCodium; it finds the engine (bundled, and the project's own), shows diagnostics, and fixes on save.

## Checked 2026-10-30 (Claude Code) — a clean VS Code

VS Code 1.144.0 with an empty profile (`--user-data-dir`, `--extensions-dir` in a scratch folder): `code --install-extension codelaw.codelaw-vscode` installed **`codelaw.codelaw-vscode@0.4.0`, the `darwin-arm64` build**, from the Marketplace; its bundled `bin/codelaw` answers `codelaw 0.4.0` and, on the GitHub Action fixture (one `debugger`, one local rule), reports `src/app.js:2:3 fixture/no-debugger … fix: auto`, exit 1. **Not checked:** diagnostics drawn in the editor and fix-on-save (a person at the window) and VSCodium (Open VSX, the inbox row).

## Definition of done

- The extension installs from both registries, finds the engine, and shows diagnostics and fix-on-save in a clean VS Code and VSCodium.

## Record affected

SR-EDITOR · SR-PACKAGES (a new published artifact) · RELEASING.md.

## Hazards

- Marketplace tokens are long-lived secrets; scope them to the `vscode` environment with a `v*` tag rule, as `npm`/`pypi` are.
- Zed and JetBrains are later items, not this one.

## Merged in: W-117

<a id="merged-in-w-117"></a>

**W-117 — Open VSX for the codelaw extension**, merged into W-95 on 2026-10-30; its file is archived at [`archive/open/W-117-open-vsx-setup.md`](../../archive/open/W-117-open-vsx-setup.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | GitHub environment `vscode`, tag rule `v*` (as `npm`, `pypi`) | a place for the token that only release tags reach | ✅ 2026-10-29 — the environment `vscode` exists |
| 2 | open-vsx.org → log in with GitHub → link an Eclipse account → sign the Publisher Agreement | the right to publish | ✅ 2026-10-29 — logged in; agreement signed |
| 3 | An access token `codelaw-publish` → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw` | CI can upload | ~~🔴~~ dropped — ruled 2026-10-30: trusted publishing, no token (below) |
| 4 | Create the namespace `codelaw`; claim its ownership (a public GitHub issue) — required for trusted publishing | `codelaw.codelaw` on Open VSX | 🔄 created 2026-10-29; ⏳ claim [#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928) still **open**, no reply from Open VSX (checked 2026-10-30 10:30 IST) |
| 5 | Agent: `publish.yml`'s Open VSX upload with `curl` (W-95's drafted job, its Open VSX half) | every release lands there | ⏳ after the claim is approved and Arpit adds the trusted publisher |

**Arpit, 2026-10-29 (voice docket):** on W-95, Open VSX now and the Marketplace by hand on release day: *"Yes."* Asked what Open VSX is (the Eclipse Foundation's store, used by Cursor, VSCodium, Windsurf and Gitpod) — then *"You have the access to the browser. We will go ahead and implement Open VSX You can go ahead and create the ID and everything."* and, the browser being out of reach of the voice call, *"That's okay. Let's put it as a work item and we'll work on it."*

**Model:** Sonnet (step 5); steps 1–4 with Arpit in a text session, Claude in Chrome helping.

### Uploaded by hand, 2026-10-30 (Arpit, from his Mac)

- **`codelaw.codelaw-vscode` 0.3.0 and 0.4.0 are on Open VSX**, all eight platforms each (alpine-arm64, alpine-x64, darwin-arm64, darwin-x64, linux-arm64, linux-x64, win32-arm64, win32-x64), uploaded with an access token and `npx ovsx publish` from v0.4.0's `vscode` artifact (`ci` 40932052080). Seen on Arpit's *Settings → Extensions* page, 10:35 IST.
- **Inactive:** every platform is struck through and the status reads *"Namespace not verified"* — Open VSX keeps them hidden until claim [#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928) is approved. Total downloads 0.
- **This satisfies the wiki's "one version published with an access token first"** (finding below), so trusted publishing can follow approval directly.
- The token was pasted into a chat on 2026-10-30: **Arpit revokes it** (*Settings → Access Tokens*).

### Ruled 2026-10-30 — trusted publishing, no token

**Arpit, 2026-10-30 (voice):** *"No, no, token isn't done. We are going to go with trusted publisher. Open VSX has trusted publisher."* · *"The namespace isn't approved yet."*

- **Step 3 (the `OVSX_PAT` token) is dropped.** CI publishes through Open VSX trusted publishing only.
- **The claim is filed:** [EclipseFdn/open-vsx.org#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928) *Claiming namespace `codelaw`*, open (checked 2026-10-30 00:30).
- **Next:** on approval, Arpit adds the trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`); then step 5.
- **Hazard, to settle before step 5:** the wiki (2026-10-29 finding below) says the extension must already have one version published with an access token before trusted publishing works. If that still holds when the namespace is approved, the first upload needs a one-time token (or Open VSX staff) — bring that back to Arpit, do not create a token unasked.

### Progress and a finding — 2026-10-29 (Arpit, with Cowork and Claude in Chrome)

- ✅ **Open VSX login:** Arpit logged in with GitHub (`@arpitarya`).
- ✅ **Publisher Agreement signed** 2026-10-29 by Arpit (his Eclipse account linked to GitHub first).
- ✅ **Namespace `codelaw` created** 2026-10-29 (Claude in Chrome, *Settings → Namespaces → Create namespace* — no token needed). ⏳ **Not verified:** Open VSX says extensions there *"can stay inactive"* until ownership is claimed, and trusted publishing needs ownership. Claiming is a public issue on `EclipseFdn/open-vsx.org` (template `claim-namespace-ownership.yml`) — Arpit's to post or approve; he approved it 2026-10-29 (*"Go ahead, claim ownership. Do what's necessary."*).
- **The claim form's routes (2026-10-29):** `codelaw` is a Marketplace publisher with no extension, so **Option 2** applies — give the Open VSX admin ID `46461f8e-a0a8-6de8-9186-8ac72e28656d` temporary *Reader* access to the publisher (revoke after the issue closes). **Blocked:** signing in to marketplace.visualstudio.com/manage now ends in Azure DevOps' *"500 — Unable to complete authentication for user due to looping logins"* (twice, 2026-10-29 00:58). **Option 1** — the publisher has an extension whose `package.json` `repository` is a repo the requester owns (`arpitarya/codelaw`) — needs no Marketplace access, and applies once the first Marketplace upload is made (v0.3.0, W-116). GitHub account `arpitarya` dates from 2012 (the 12-month rule).
- **Open VSX has trusted publishing (OIDC)** — *Settings → Trusted Publishers*, per namespace: owner, repository, workflow file, optional environment ([wiki](https://github.com/eclipse-openvsx/openvsx/wiki/Trusted-Publishing)). The workflow sends its GitHub OIDC token to `POST /api/-/trusted-publishing/token` and gets a token scoped to one extension for minutes. **So CI needs no `OVSX_PAT`** — the same model as npm and PyPI here.
- **Its limits:** namespace *ownership* and the signed agreement first; the extension must already have **one version published with an access token**; one trusted publisher per extension. The wiki documents only `ovsx publish --trusted-publishing` (not ratified, L2 refuses it); the exchange's request body is not documented there — read from the registry's source before `curl` replaces it.
- **So the plan changes:** a token is needed once — to create the namespace and upload the first version by hand (at v0.3.0, with the Marketplace upload) — then a trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`), and `publish.yml` uses OIDC from the next release; the token can then be revoked.

**State, 2026-10-29 01:15 (Cowork):** environment ✅, login and agreement ✅, namespace ✅ (unverified). Left: (1) the ownership claim — no issue titled *codelaw* found on `EclipseFdn/open-vsx.org` (GitHub search, 01:15); (2) one access token → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw` for the first upload, which `publish.yml`'s `vscode` job uses; (3) after it, the trusted publisher and the token revoked.

**2026-10-29 (voice):** `ovsx` is ratified for this extension only (L2 decision 8, D-47): W-95's job publishes with it — the first version with `OVSX_PAT`, later ones with `ovsx publish --trusted-publishing` once the trusted publisher is registered.

### The commands (from W-95's inbox row)

```bash
gh api -X PUT repos/arpitarya/codelaw/environments/vscode -F 'deployment_branch_policy[protected_branches]=false' -F 'deployment_branch_policy[custom_branch_policies]=true' \
  && gh api -X POST repos/arpitarya/codelaw/environments/vscode/deployment-branch-policies -f name='v*' -f type=tag
gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw          # paste the token at the prompt
read -rs T && curl -fsS -X POST -H 'Content-Type: application/json' -d '{"name":"codelaw"}' "https://open-vsx.org/api/-/namespace/create?token=$T"
```

Verified badge (optional): *Claim namespace ownership* at <https://github.com/EclipseFdn/open-vsx.org/issues/new/choose>.

### Definition of done

- `codelaw.codelaw` is on open-vsx.org at the release's version, uploaded by `publish.yml`.

### Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the extension's upload, already `curl`, W-95's ruling) · RELEASING.md.
