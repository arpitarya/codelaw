---
type: Handoff
name: W-151
description: "Arpit, 2026-10-30: release v0.4.1 from CI, with everything it needs — publish.yml uploads the dev-server plugins and Fux index packages (was W-150), codelaw-rules-deps on PyPI through its pending publisher (was W-134) and on npm by hand first (was W-135). Merged 2026-10-30."
item: W-151
filed: 2026-10-30
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: arpit
---

# W-151 — Release v0.4.1, and everything it needs

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `publish.yml` uploads every package a release builds: the dev-server plugins (W-150), the Fux index set (B-063), `codelaw-rules-deps` through its pending publisher (W-134) | nothing is left behind again | ✅ 2026-10-30 (W-150's change) |
| 2 | Agent: `python3 scripts/version.py --set 0.4.1`; CHANGELOG's *Unreleased* becomes *0.4.1*; `ci.yml` green on that commit on `main` | a releasable commit | ✅ 2026-10-30 — `c4d519c7` (bump + *0.4.1 — 2026-10-30*), `ci` **41007205048 green** (05:13–05:32 UTC); its `dist` and `fux-index` artifacts are 0.4.1 and attested |
| 3 | **Arpit:** `scripts/npm-trust.sh --fux-index` (W-85's *Pending validation* configurations lapse after ~2 days, and no CI publish ever validated the index names); then **W-135** — `npm publish` `@codelaw/rules-deps` and `@codelaw/rules-deps-fux-index` at 0.4.1 by hand from step 2's run, and `scripts/npm-trust.sh` for both | npm trusts every name, or `trust` stops the release before anything is written | 🔴 Arpit |
| 4 | **Arpit:** `gh release create v0.4.1` with CHANGELOG's section as notes (RELEASING.md step 3) | every package at 0.4.1 on both registries | 🔴 Arpit |
| 5 | Agent: every name checked at 0.4.1 on npm and PyPI; `v0` moved to `v0.4.1` (W-97); the run timed for W-126; W-134 step 3 | proof | ⏳ |
| W-150 | `publish.yml` uploads the dev-server plugins and the Fux index packages — merged in; its steps under [§ W-150](#merged-in-w-150) | one item, not two | ⏳ — see § |
| W-134 | PyPI: pending trusted publishers for the `deps` pack — merged in; its steps under [§ W-134](#merged-in-w-134) | one item, not two | ⏳ — see § |
| W-135 | npm: first publish of the `deps` pack's names — merged in; its steps under [§ W-135](#merged-in-w-135) | one item, not two | ⏳ — see § |

**Merged 2026-10-30 (Arpit, text):** *"There are too many work items. See if we can review all the work items and merge them into few of them."* → *"go ahead, merge them."* Merged in: W-150, W-134, W-135.

**Arpit, 2026-10-30 (Claude Code), asked how the packages behind 0.4.0 should catch up — CI cannot upload them at 0.4.0 (the `npm`/`pypi` environments take `v*` tags only; v0.4.0's own `publish.yml` has no tier for them) and this machine has no npm login:** *"Release v0.4.1 from CI"* — over *"You log in, I publish"* and *"Wire CI only, skip catch-up"*. Asked the same turn: *"close out item 149, item 126, item 95, item 97. Make sure the pipeline is green completely. And publish the packages that haven't been upgraded to .4.0 version."*

**Model:** Sonnet (Claude Code) for steps 2 and 5; steps 3–4 are the decider's hand.

## Behind 0.4.0 (checked 2026-10-30 on both registries)

| registry | packages | at | since |
|---|---|---|---|
| npm | `@codelaw/vite-plugin`, `@codelaw/webpack-plugin`, `@codelaw/next` | 0.2.0 | ✅ 0.4.0 — Arpit, by hand, 10-30 10:30 IST (W-150 step 1) |
| npm | `@codelaw/rules-<name>-fux-index` × 16 | 0.2.0; `spell`'s 0.3.0 | ✅ 0.4.0 — Arpit, by hand, 10-30 (all 16 on the registry, rechecked 10:40) |
| PyPI | `codelaw-rules-<name>-fux-index` × 11 | 0.2.0; the security three and `spell`'s 0.3.0 | ⏳ this release |

Every other name is at 0.4.0. v0.4.0's CI run (40932052080) holds the PyPI wheels at 0.4.0 until 2026-11-28, but no workflow may upload them at that version. **So this release now carries:** PyPI's eleven index wheels, the `deps` pack and its index (new), and everything else at 0.4.1.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | npm and PyPI show every codelaw name at 0.4.1, `deps` and its index included (PyPI's `deps` index once W-134 step 3 adds its pending publisher) | ⏳ |
| 2 | `publish.yml`'s run green, its time recorded in W-126 | ⏳ |

## Blockers

Steps 3–4: the decider's npm login and the Release.

## Record affected

None beyond W-150's change (SR-PACKAGES decisions 3a, 11, 14; L6 decision 9; RELEASING.md).

## Hazards

- `trust` checks every npm name before either registry is written: one lapsed or missing name (the `deps` pair, a lapsed index name) stops the whole release — step 3 first.
- 0.4.1 carries everything since 0.4.0 (the `deps` pack, flags, markers, complexity, W-137's editor), not only the catch-up.

## Reference

[W-150](W-151-release-0-4-1.md) · [W-134](W-151-release-0-4-1.md) · [W-135](W-151-release-0-4-1.md) · [W-126](W-126-fast-pipeline.md) · [W-97](W-97-github-action.md) · RELEASING.md

## Merged in: W-150

<a id="merged-in-w-150"></a>

**W-150 — `publish.yml` uploads the dev-server plugins and the Fux index packages**, merged into W-151 on 2026-10-30; its file is archived at [`archive/open/W-150-publish-dev-server-plugins.md`](../../archive/open/W-150-publish-dev-server-plugins.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Arpit publishes the three at 0.4.0 by hand from v0.4.0's `dist` (`ci` run 40932052080) | they catch up now | ✅ 2026-10-30 10:30 IST — Arpit, `npm publish` from v0.4.0's `dist` (`ci` 40932052080, checksums verified); all three at 0.4.0 on the registry |
| 2 | `publish.yml`'s npm job: a tier for `codelaw-vite-plugin-*.tgz`, `codelaw-webpack-plugin-*.tgz`, `codelaw-next-*.tgz` (after `codelaw`, since they spawn it) | every release ships them | ✅ 2026-10-30 — in `@codelaw/skills`' tier, after `codelaw`; the Fux index packages (B-063) a last tier |
| 3 | `scripts/package.py check` (or the gate) fails when `dist/npm` holds a tarball no tier uploads | this cannot happen again | ✅ 2026-10-30 — `package.py check` reads `publish.yml`'s `tier` globs (`publish_tiers()`); on v0.4.0's real `dist/npm` the old tiers leave exactly the three untiered, the new ones none |
| 4 | B-063: the Fux index packages from `dist/fux-index/` — npm (after their pack's tier) and PyPI (the pypi job; all eleven projects already have a trusted publisher, checked 2026-10-30) | every release ships them at its version | ✅ 2026-10-30 — attested by `ci.yml`'s `fux-index` job, checked and verified in the gate, trust-checked, npm's last tier (after every pack), PyPI with the rest; a missing PyPI project skipped and named |
| 5 | Arpit: `scripts/npm-trust.sh --fux-index` (one security-key tap per name) before the release that first uploads them — the trust check stops a release otherwise | CI may publish them | 🔴 Arpit, before that release |

**Asked 2026-10-30 (Arpit, text):** *"What are the packages that aren't published? Check on all of them."* → *"For npm, give me the command so that I can publish them right away from my local."*

**Model:** Sonnet (Claude Code): a workflow tier and one check.

**Arpit, 2026-10-30 (Claude Code), on how the packages still behind 0.4.0 catch up:** *"Release v0.4.1 from CI"* — [W-151](W-151-release-0-4-1.md). (The npm ones were published by hand the same morning, step 1; PyPI's Fux index wheels and the `deps` pack wait for that release.)

**Arpit, 2026-10-30 (text):** *"if it is published once then we will publish it from ci"* — every name above already exists on its registry, so from the next release CI publishes them; nothing more by hand.

### Found

- npm: all three at **0.2.0** (2026-10-27, by hand, W-49); every release since built them into `dist/npm/` (`scripts/package.py`, `DEV_PLUGINS`) and listed them in `package.py names`, so `npm-trust.sh` and the trust check cover them — but the tiers upload only `codelaw-cli-*`, `codelaw-rules-*`, `codelaw-[0-9]*` and `codelaw-skills-*`.

### Definition of done

| DoD | what | state |
|---|---|---|
| 1 | npm shows the three at the current release | ✅ 0.4.0, 2026-10-30 (by hand) |
| 2 | the next release uploads them from CI; the new check is tested | 🔄 check tested 2026-10-30; the upload is W-151's release |

### Blockers

None for step 2.

### Record affected

[SR-PACKAGES](../../records/0107_packages.md) decision 3a · RELEASING.md.

### Found 2026-10-30 — `@codelaw/rules-stencil-fux-index` 0.4.0 is *staged*

Arpit's `npm publish` printed `+ …@0.4.0`, but the registry shows only `0.0.0-stage` and `0.2.0`; a re-run answered *E409 "Cannot publish over previously staged version 0.4.0"*. A staged version is in the registry but not public until `npm stage approve` ([npm stage](https://docs.npmjs.com/cli/v11/commands/npm-stage)). Left as is by the ruling above: the next release's CI upload is a new version. The odd `0.0.0-stage` hints an earlier staging run — find what stages, and that CI's upload never does.

### Hazards

- They run `codelaw`; upload them after the `codelaw` tier, as `codelaw-skills` is.

### Reference

W-49 (`archive/open/`) · [W-126](W-126-fast-pipeline.md) (the tiers)

## Merged in: W-134

<a id="merged-in-w-134"></a>

**W-134 — PyPI: pending trusted publishers for the `deps` pack**, merged into W-151 on 2026-10-30; its file is archived at [`archive/open/W-134-deps-pypi-pending-publishers.md`](../../archive/open/W-134-deps-pypi-pending-publishers.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | ✅ W-128 built and closed 2026-10-30; the next release (after 0.4.0) carries `deps` |
| 2 | Claude, in Chrome: pypi.org → *Your account → Publishing → Add a pending publisher*, filled twice; Arpit checks and submits | `publish.yml` can create both projects | 🔄 `codelaw-rules-deps` saved 2026-10-30 10:20 IST (Claude in Chrome, on Arpit's ask); the Fux index refused — *"A pending trusted publisher matching this configuration has already been registered for a different [project]"* — until the first project exists |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** Cowork with Claude in Chrome — a browser form on Arpit's signed-in account; he submits.

### The form, twice

| field | value |
|---|---|
| PyPI project name | `codelaw-rules-deps`, then `codelaw-rules-deps-fux-index` |
| Owner | `arpitarya` |
| Repository | `codelaw` |
| Workflow | `publish.yml` |
| Environment | `pypi` |

### Ruled 2026-10-30 — CI creates it, never a local upload

**Arpit, 2026-10-30 (text):** *"I don't want to trigger the package from my local for PyPI. I'll trigger the pipeline, and that is going to publish the package."*

**Agent, before the next release:**
1. `publish.yml`'s PyPI `exists` step: upload a wheel whose project is missing when it is `codelaw` **or `codelaw-rules-deps`** (a named list beside the `codelaw` exception, each with its pending publisher); every other missing project is still skipped and named.
2. Amend [L6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 (*"PyPI uploads only what exists"*: the exception becomes the projects holding a pending publisher, with this ruling quoted), SR-PACKAGES (*"No new-project path in CI"* and the first-publish consequence), RELEASING.md step 4; CLAUDE.md's L6 line if it names the rule.
2b. **Asked the same day:** *"For the PyPy package, give me the GitHub command to trigger the pipeline just for that one package."* — no such trigger exists (`workflow_dispatch` takes `only: npm|vscode`; the PyPI job runs on a Release only). Add `only: pypi` with a `package` input, from a `v*` tag, uploading just that wheel from the tag's `dist`: `gh workflow run publish.yml -R arpitarya/codelaw --ref v<next> -f only=pypi -f package=codelaw-rules-deps`. W-125 removed a similar path (`only=pypi-new`); this is its narrower return, by this ruling — say so in SR-PACKAGES.
3. After the release that creates `codelaw-rules-deps`: Claude adds the pending publisher for `codelaw-rules-deps-fux-index` (PyPI allows one at a time) — it is uploaded only once [W-150](W-151-release-0-4-1.md) (was B-063) is done.

### Built 2026-10-30 (Claude Code, in W-150's change)

- **Step 1:** `publish.yml`'s `exists` step uploads a missing project when it is in `PENDING` (`codelaw`, `codelaw-rules-deps`); every other missing one is skipped and named.
- **Step 2:** L6 decision 9 amended with the ruling quoted; SR-PACKAGES decision 11's *"No new-project path in CI"* rewritten as *"A new PyPI project in CI only through a pending publisher"*; RELEASING.md's skipped-project paragraph. CLAUDE.md's L6 line does not name the rule — unchanged.
- **Step 2b:** `workflow_dispatch` `only: pypi` + `package`, from a `v*` tag only, uploads that one project's wheel from the tag's `dist`: `gh workflow run publish.yml -R arpitarya/codelaw --ref v0.4.1 -f only=pypi -f package=codelaw-rules-deps` — once v0.4.1 exists (W-151).
- **Left:** step 3 after v0.4.1.

### Found 2026-10-30 (Cowork) — a pending publisher alone does not publish

- **`publish.yml` skips a PyPI project that does not exist yet**, except `codelaw` (its `exists` step; RELEASING.md: *"its first upload is by hand"*). So the pending publisher only works if `publish.yml` also uploads `codelaw-rules-deps` (one line beside the `codelaw` exception), or its first version goes up by hand: `gh run download <ci-run-id> -R arpitarya/codelaw -n dist -D dist && twine upload dist/pypi/codelaw_rules_deps-<v>-py3-none-any.whl`.
- **Not before the next release:** the `deps` rules are native and post-date `v0.4.0` (`7b855499` is not in it), and `main` still says 0.4.0 — a wheel published now would claim a version whose engine lacks its rules.
- **All 24 existing codelaw PyPI projects already have an active trusted publisher** (checked on *Publishing*, 2026-10-30).

### Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both pending publishers saved on PyPI | 🔄 `codelaw-rules-deps` saved 10-30; the Fux index's after the first exists (step 3) |
| 2 | The first release with the `deps` pack creates both projects | ⏳ v0.4.1, [W-151](W-151-release-0-4-1.md) |

### Blockers

None — W-128 closed 2026-10-30. Do it just before the next release.

### Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

### Hazards

- A pending publisher reserves nothing: someone else can take the name first — do it close to the release, as ruled, but not after it.
- [W-150](W-151-release-0-4-1.md) (was B-063) built 2026-10-30: `publish.yml` uploads the Fux index wheels, so the second name is created by the release after its pending publisher is added.

### Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [proposal](../proposals/deps-flags-markers-reviews.md) §2.1 · [W-135](W-151-release-0-4-1.md)

## Merged in: W-135

<a id="merged-in-w-135"></a>

**W-135 — npm: first publish of the `deps` pack's names**, merged into W-151 on 2026-10-30; its file is archived at [`archive/open/W-135-deps-npm-first-publish.md`](../../archive/open/W-135-deps-npm-first-publish.md). As it stood:

### At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | ✅ W-128 built and closed 2026-10-30; the next release (after 0.4.0) carries `deps` |
| 2 | Arpit publishes `@codelaw/rules-deps` and `@codelaw/rules-deps-fux-index` once by hand, as RELEASING.md does for a new pack | the names exist on npm | 🔴 Arpit |
| 3 | Add `arpitarya/codelaw` · `publish.yml` · `npm` as each package's trusted publisher | every later release publishes them from CI | 🔴 Arpit |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** NONE for steps 2–3 — npm has no pending publisher; only Arpit's npm login can publish a new name.

### Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both names on npm with a trusted publisher | ⏳ |
| 2 | The next release publishes both from `publish.yml` | ⏳ |

### Blockers

None — W-128 closed 2026-10-30. Do it just before the next release.

### Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

### Hazards

- [W-150](W-151-release-0-4-1.md) (was B-063): `publish.yml` does not upload Fux-index packages yet; the fux-index name is bound by hand regardless.

### Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [W-134](W-151-release-0-4-1.md) · RELEASING.md
