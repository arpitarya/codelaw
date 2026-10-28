---
type: Handoff
name: W-95
description: "Arpit, 2026-10-27 (W-36 gap G2): publish the built VS Code extension on the Marketplace and Open VSX."
item: W-95
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-28
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-95 — Publish the VS Code extension — Marketplace and Open VSX

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Package `packaging/vscode` per platform with its bundled binary (or the npm binary lookup); `vsce package` / `ovsx` dry runs in CI | publishable artifacts | ✅ 2026-10-27 — eight `.vsix` built and checked in CI by `scripts/vsix.py`, without `vsce` (see below); no `vsce`/`ovsx` dry run: neither is ratified |
| 2 | Arpit creates the Marketplace publisher and the Open VSX namespace, and stores the tokens (or a trusted-publishing equivalent) as GitHub secrets in the `vscode` environment | the right to publish | 🔴 Arpit — the publisher, namespace and tokens (the inbox row has every click); the upload tool ruled 2026-10-27: (a), `curl`, no dependency |
| 3 | `publish.yml` publishes the extension on a Release, after the engine | one release, every surface | ⏳ after step 2 — the job is drafted below; it uploads with `curl` (ruled 2026-10-27) |
| 4 | README / USAGE: install from the Marketplace; the preview note drops "the editor extension's Marketplace listing comes later" | users can install it | ⏳ after the first listing |

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

## Left

1. **The decider (step 2, the inbox row):** the GitHub environment `vscode` with a `v*` tag rule; the Marketplace publisher `codelaw` and a PAT → secret `VSCE_PAT`; the Open VSX account, Eclipse publisher agreement, a token → secret `OVSX_PAT`, and the namespace `codelaw`; **and the ruling: (a) own `curl` uploads, no dependency (recommended), or (b) amend L2 and ratify `@vscode/vsce` 4.0.0 and `ovsx` 1.3.0, CI-only.**
2. **Then an agent (step 3):** the job above into `publish.yml`, its upload verified on the first Release; `half-published` taught the extension; RELEASING.md's section turned from "built" to "published".
3. **Step 4:** README / USAGE install lines; the preview note's "Marketplace listing comes later" dropped.
4. **DoD, by hand on the first listing:** install from each registry in a clean VS Code and a clean VSCodium; it finds the engine (bundled, and the project's own), shows diagnostics, and fixes on save.

## Definition of done

- The extension installs from both registries, finds the engine, and shows diagnostics and fix-on-save in a clean VS Code and VSCodium.

## Record affected

SR-EDITOR · SR-PACKAGES (a new published artifact) · RELEASING.md.

## Hazards

- Marketplace tokens are long-lived secrets; scope them to the `vscode` environment with a `v*` tag rule, as `npm`/`pypi` are.
- Zed and JetBrains are later items, not this one.
