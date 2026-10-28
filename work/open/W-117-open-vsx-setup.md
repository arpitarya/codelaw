---
type: Handoff
name: W-117
description: "Arpit, 2026-10-29 (voice): set up Open VSX for the codelaw extension — the namespace, the token, the vscode environment — then publish.yml uploads there with curl."
item: W-117
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: arpit
---

# W-117 — Open VSX for the codelaw extension

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | GitHub environment `vscode`, tag rule `v*` (as `npm`, `pypi`) | a place for the token that only release tags reach | 🔴 Arpit (or Claude in Chrome, with his OK) |
| 2 | open-vsx.org → log in with GitHub → link an Eclipse account → sign the Publisher Agreement | the right to publish | ✅ 2026-10-29 — logged in; agreement signed |
| 3 | An access token `codelaw-publish` → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw` | CI can upload | 🔴 Arpit — the token is his to handle |
| 4 | Create the namespace `codelaw`; claim its ownership (a public GitHub issue) — required for trusted publishing | `codelaw.codelaw` on Open VSX | 🔄 created 2026-10-29; ⏳ the ownership claim |
| 5 | Agent: `publish.yml`'s Open VSX upload with `curl` (W-95's drafted job, its Open VSX half) | every release lands there | ⏳ after step 3 |

**Arpit, 2026-10-29 (voice docket):** on W-95, Open VSX now and the Marketplace by hand on release day: *"Yes."* Asked what Open VSX is (the Eclipse Foundation's store, used by Cursor, VSCodium, Windsurf and Gitpod) — then *"You have the access to the browser. We will go ahead and implement Open VSX You can go ahead and create the ID and everything."* and, the browser being out of reach of the voice call, *"That's okay. Let's put it as a work item and we'll work on it."*

**Model:** Sonnet (step 5); steps 1–4 with Arpit in a text session, Claude in Chrome helping.

## Progress and a finding — 2026-10-29 (Arpit, with Cowork and Claude in Chrome)

- ✅ **Open VSX login:** Arpit logged in with GitHub (`@arpitarya`).
- ✅ **Publisher Agreement signed** 2026-10-29 by Arpit (his Eclipse account linked to GitHub first).
- ✅ **Namespace `codelaw` created** 2026-10-29 (Claude in Chrome, *Settings → Namespaces → Create namespace* — no token needed). ⏳ **Not verified:** Open VSX says extensions there *"can stay inactive"* until ownership is claimed, and trusted publishing needs ownership. Claiming is a public issue on `EclipseFdn/open-vsx.org` (template `claim-namespace-ownership.yml`) — Arpit's to post or approve; he approved it 2026-10-29 (*"Go ahead, claim ownership. Do what's necessary."*).
- **The claim form's routes (2026-10-29):** `codelaw` is a Marketplace publisher with no extension, so **Option 2** applies — give the Open VSX admin ID `46461f8e-a0a8-6de8-9186-8ac72e28656d` temporary *Reader* access to the publisher (revoke after the issue closes). **Blocked:** signing in to marketplace.visualstudio.com/manage now ends in Azure DevOps' *"500 — Unable to complete authentication for user due to looping logins"* (twice, 2026-10-29 00:58). **Option 1** — the publisher has an extension whose `package.json` `repository` is a repo the requester owns (`arpitarya/codelaw`) — needs no Marketplace access, and applies once the first Marketplace upload is made (v0.3.0, W-116). GitHub account `arpitarya` dates from 2012 (the 12-month rule).
- **Open VSX has trusted publishing (OIDC)** — *Settings → Trusted Publishers*, per namespace: owner, repository, workflow file, optional environment ([wiki](https://github.com/eclipse-openvsx/openvsx/wiki/Trusted-Publishing)). The workflow sends its GitHub OIDC token to `POST /api/-/trusted-publishing/token` and gets a token scoped to one extension for minutes. **So CI needs no `OVSX_PAT`** — the same model as npm and PyPI here.
- **Its limits:** namespace *ownership* and the signed agreement first; the extension must already have **one version published with an access token**; one trusted publisher per extension. The wiki documents only `ovsx publish --trusted-publishing` (not ratified, L2 refuses it); the exchange's request body is not documented there — read from the registry's source before `curl` replaces it.
- **So the plan changes:** a token is needed once — to create the namespace and upload the first version by hand (at v0.3.0, with the Marketplace upload) — then a trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`), and `publish.yml` uses OIDC from the next release; the token can then be revoked.

## The commands (from W-95's inbox row)

```bash
gh api -X PUT repos/arpitarya/codelaw/environments/vscode -F 'deployment_branch_policy[protected_branches]=false' -F 'deployment_branch_policy[custom_branch_policies]=true' \
  && gh api -X POST repos/arpitarya/codelaw/environments/vscode/deployment-branch-policies -f name='v*' -f type=tag
gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw          # paste the token at the prompt
read -rs T && curl -fsS -X POST -H 'Content-Type: application/json' -d '{"name":"codelaw"}' "https://open-vsx.org/api/-/namespace/create?token=$T"
```

Verified badge (optional): *Claim namespace ownership* at <https://github.com/EclipseFdn/open-vsx.org/issues/new/choose>.

## Definition of done

- `codelaw.codelaw` is on open-vsx.org at the release's version, uploaded by `publish.yml`.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the extension's upload, already `curl`, W-95's ruling) · RELEASING.md.
