---
type: Handoff
name: W-134
description: "Arpit, 2026-10-30 (voice): the deps pack's two PyPI names as pending trusted publishers, as their own item, done just before the deps release."
item: W-134
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-134 — PyPI: pending trusted publishers for the `deps` pack

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait until W-128 is ready to release | the names are bound when they are needed | ✅ W-128 built and closed 2026-10-30; the next release (after 0.4.0) carries `deps` |
| 2 | Claude, in Chrome: pypi.org → *Your account → Publishing → Add a pending publisher*, filled twice; Arpit checks and submits | `publish.yml` can create both projects | 🔄 `codelaw-rules-deps` saved 2026-10-30 10:20 IST (Claude in Chrome, on Arpit's ask); the Fux index refused — *"A pending trusted publisher matching this configuration has already been registered for a different [project]"* — until the first project exists |

**Arpit, 2026-10-30 (voice):** *"just before the deps release."* · *"I'm just talking about PyPI publishing and NPM publishing. Have those as two separate work items."*

**Model:** Cowork with Claude in Chrome — a browser form on Arpit's signed-in account; he submits.

## The form, twice

| field | value |
|---|---|
| PyPI project name | `codelaw-rules-deps`, then `codelaw-rules-deps-fux-index` |
| Owner | `arpitarya` |
| Repository | `codelaw` |
| Workflow | `publish.yml` |
| Environment | `pypi` |

## Ruled 2026-10-30 — CI creates it, never a local upload

**Arpit, 2026-10-30 (text):** *"I don't want to trigger the package from my local for PyPI. I'll trigger the pipeline, and that is going to publish the package."*

**Agent, before the next release:**
1. `publish.yml`'s PyPI `exists` step: upload a wheel whose project is missing when it is `codelaw` **or `codelaw-rules-deps`** (a named list beside the `codelaw` exception, each with its pending publisher); every other missing project is still skipped and named.
2. Amend [L6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 (*"PyPI uploads only what exists"*: the exception becomes the projects holding a pending publisher, with this ruling quoted), SR-PACKAGES (*"No new-project path in CI"* and the first-publish consequence), RELEASING.md step 4; CLAUDE.md's L6 line if it names the rule.
2b. **Asked the same day:** *"For the PyPy package, give me the GitHub command to trigger the pipeline just for that one package."* — no such trigger exists (`workflow_dispatch` takes `only: npm|vscode`; the PyPI job runs on a Release only). Add `only: pypi` with a `package` input, from a `v*` tag, uploading just that wheel from the tag's `dist`: `gh workflow run publish.yml -R arpitarya/codelaw --ref v<next> -f only=pypi -f package=codelaw-rules-deps`. W-125 removed a similar path (`only=pypi-new`); this is its narrower return, by this ruling — say so in SR-PACKAGES.
3. After the release that creates `codelaw-rules-deps`: Claude adds the pending publisher for `codelaw-rules-deps-fux-index` (PyPI allows one at a time) — it is uploaded only once [W-150](W-150-publish-dev-server-plugins.md) (was B-063) is done.

## Built 2026-10-30 (Claude Code, in W-150's change)

- **Step 1:** `publish.yml`'s `exists` step uploads a missing project when it is in `PENDING` (`codelaw`, `codelaw-rules-deps`); every other missing one is skipped and named.
- **Step 2:** L6 decision 9 amended with the ruling quoted; SR-PACKAGES decision 11's *"No new-project path in CI"* rewritten as *"A new PyPI project in CI only through a pending publisher"*; RELEASING.md's skipped-project paragraph. CLAUDE.md's L6 line does not name the rule — unchanged.
- **Step 2b:** `workflow_dispatch` `only: pypi` + `package`, from a `v*` tag only, uploads that one project's wheel from the tag's `dist`: `gh workflow run publish.yml -R arpitarya/codelaw --ref v0.4.1 -f only=pypi -f package=codelaw-rules-deps` — once v0.4.1 exists (W-151).
- **Left:** step 3 after v0.4.1.

## Found 2026-10-30 (Cowork) — a pending publisher alone does not publish

- **`publish.yml` skips a PyPI project that does not exist yet**, except `codelaw` (its `exists` step; RELEASING.md: *"its first upload is by hand"*). So the pending publisher only works if `publish.yml` also uploads `codelaw-rules-deps` (one line beside the `codelaw` exception), or its first version goes up by hand: `gh run download <ci-run-id> -R arpitarya/codelaw -n dist -D dist && twine upload dist/pypi/codelaw_rules_deps-<v>-py3-none-any.whl`.
- **Not before the next release:** the `deps` rules are native and post-date `v0.4.0` (`7b855499` is not in it), and `main` still says 0.4.0 — a wheel published now would claim a version whose engine lacks its rules.
- **All 24 existing codelaw PyPI projects already have an active trusted publisher** (checked on *Publishing*, 2026-10-30).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | Both pending publishers saved on PyPI | 🔄 `codelaw-rules-deps` saved 10-30; the Fux index's after the first exists (step 3) |
| 2 | The first release with the `deps` pack creates both projects | ⏳ v0.4.1, [W-151](W-151-release-0-4-1.md) |

## Blockers

None — W-128 closed 2026-10-30. Do it just before the next release.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) (the names) · RELEASING.md.

## Hazards

- A pending publisher reserves nothing: someone else can take the name first — do it close to the release, as ruled, but not after it.
- [W-150](W-150-publish-dev-server-plugins.md) (was B-063) built 2026-10-30: `publish.yml` uploads the Fux index wheels, so the second name is created by the release after its pending publisher is added.

## Reference

[W-128](W-128-deps-pack-and-vulnerabilities.md) · [proposal](../proposals/deps-flags-markers-reviews.md) §2.1 · [W-135](W-135-deps-npm-first-publish.md)
