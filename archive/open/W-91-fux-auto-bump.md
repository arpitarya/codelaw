---
type: Handoff
name: W-91
description: "Arpit, 2026-10-27 (voice docket): fux-engine always at its latest release — ratified with no fixed version; a scheduled job bumps the pin and opens a PR."
item: W-91
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: done
---

# W-91 — fux-engine at its latest, every commit still pinned

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | A scheduled workflow (daily) asks PyPI for `fux-engine`'s latest version and compares it with `FUX_VERSION` in `scripts/fux-index.py` | a new fux is noticed within a day | ✅ 2026-10-27 — `.github/workflows/fux-bump.yml`, `fux-index.py latest` |
| 2 | On a newer one: bump `FUX_VERSION`, re-run `fux-index.py assemble` + `check`, commit on `deps/fux-<v>`, open or update one PR | the bump arrives as a reviewable change | ✅ 2026-10-27 live — run 40443728233 (`-f version=5.1.0`) opened PR #4 on `deps/fux-5.1.0`; run 40443835498 the same: *"already holds this tree — not pushed"*, still one PR |
| 3 | The PR's CI builds the indexes with the new fux and checks their digests reproduce | a bump that breaks the index is red, not merged | ✅ 2026-10-27 — `ci.yml` dispatched on the branch: run 40443784185 on `eaedb2c`, 27/27 green; PR #4 then closed with its branch (5.1.0 is older than the pin; it existed only to prove the job) |
| 4 | D-34's row says "unversioned; each commit pins `FUX_VERSION`, bumped by this job" | the record matches | ✅ 2026-10-27 (L11's list, D-34; W-90 moved it from the old root file) |

**Arpit, 2026-10-27 (voice docket):** *"For Fux, what I want is whenever there is a latest version available, we should be using that."* On the trade-off (CI grabbing "latest" would break W-34's reproducible digest): *"Go with the recommendation."* — ratify fux-engine with no fixed version; pin per commit; a scheduled job bumps the pin and opens a PR.

## What was built (2026-10-27, Claude Code)

- **`scripts/fux-index.py latest [--pypi-json F]`** prints `newer <v>` or `current <pin>`: PyPI's fux-engine (one read-only GET of `https://pypi.org/pypi/fux-engine/json`, or the file given) against `FUX_VERSION`, compared as final releases (a pre-release is refused). `$FUX_LATEST` forces the target — the workflow's `version` input, or a local test — and may move the pin down. **`bump <v>`** rewrites the one `FUX_VERSION` line.
- **`.github/workflows/fux-bump.yml`** — daily (04:00 UTC) and `workflow_dispatch` (`-f version=<v>` for an exact release). Main only. On `current` it writes nothing and is green. On `newer`: `bump`, a venv with `fux-engine==<v>`, `assemble --out dist`, `check dist`; commit on `deps/fux-<v>` (message ends `no SR affected` — the bump touches no rule); push **only if the remote branch's tree differs** (`--force` on the job's own branch, rebuilt from main each run); find the open PR **by head branch** — `gh pr edit` it, or `gh pr create`; close an open PR from an older `deps/fux-*` branch as superseded; then `gh workflow run ci.yml --ref deps/fux-<v>` when it pushed or opened. No new action or package: `gh`, `git`, stdlib Python, and `actions/checkout` / `actions/setup-python` at ci.yml's SHAs.
- **Token: `GITHUB_TOKEN` + `workflow_dispatch` on the branch** — no secret. A PR or push made with `GITHUB_TOKEN` starts no workflow; `workflow_dispatch` is the exception, so the job dispatches `ci.yml` itself (`actions: write`). Trade-off, over a personal token: the PR's CI is a `workflow_dispatch` run (every job fresh, checks on the head commit, linked from the PR body), not a `pull_request` run; a human push to the branch or a close/reopen starts the ordinary ones. Stated in the workflow header, RELEASING.md and SR-PACKAGES decision 14.
- **Verified locally:** `latest` against the real PyPI JSON (`current 5.1.1` — 5.1.1 is the latest), against faked newer (`9.9.9`, `5.10.0` beats `5.1.1`) and older JSON, a pre-release refused, `bump` round-trip. The workflow's two shell steps, extracted from the YAML, ran four times against a bare local `origin` with a stub `gh`: no new version → nothing; `5.1.0` → branch pushed, PR #1 opened, ci.yml dispatched; `5.1.0` again → **not pushed, PR #1 edited, no second PR**; `5.0.0` → PR #2 opened, #1 closed as superseded. The YAML parses (Ruby).
- **Found:** the repository setting *Allow GitHub Actions to create and approve pull requests* is **off** (`gh api repos/arpitarya/codelaw/actions/permissions/workflow` → `can_approve_pull_request_reviews: false`), so `gh pr create` would fail today. The job then fails red, naming the setting, with the branch pushed. That is the decider's — in the inbox.
- **Turned on by Arpit, 2026-10-27 08:44 IST** (`gh api -X PUT …/actions/permissions/workflow -F can_approve_pull_request_reviews=true`); read back: `default_workflow_permissions: read`, `can_approve_pull_request_reviews: true`.

## Left

Nothing. Closed 2026-10-27: a plain dispatch (run 40443668613) said *"fux-engine 5.1.1 (PyPI), pinned 5.1.1: nothing to do"*; the live PR test is steps 2–3 above.

## Record affected

L11's list D-34 (done; moved into [SR-LAW-11](../../records/0013_LAW-11-dependencies.md) by W-90); [SR-PACKAGES](../../records/0107_packages.md) decision 14 if it names the pin.

## Hazards

- No new third-party action (L11): use `gh` and `git` on the runner, not a marketplace PR action.
- The repository setting *Allow GitHub Actions to create pull requests* must be on — the decider's, if it is off.
- L11 decision 4 ("the tree as pinned that day") — W-90 states the unversioned ratification in the law.

**Model:** Sonnet.
