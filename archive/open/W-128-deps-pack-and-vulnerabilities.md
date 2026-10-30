---
type: Handoff
name: W-128
description: "Arpit asked 2026-10-29 (Cowork, voice) for codelaw to tell him and his agents which npm and Python dependencies are vulnerable and the exact version to move to, from configurable merged sources, in a new deps pack."
item: W-128
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-128 — The `deps` pack and vulnerable dependencies

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write the records: L5 and L6 amended, SR-RULE-PACKS decision 15 moved to the `deps` pack | the law allows the build | ✅ 2026-10-30 |
| 2 | Create the `deps` pack; rename the two `data/` dependency rules into it | one pack for everything dependency | ✅ 2026-10-30 |
| 3 | Teach `deps refresh` to fetch advisories from a list of on/off sources and merge them | vulnerability facts in the committed snapshot | ✅ 2026-10-30 |
| 4 | Build `deps/vulnerable` (direct and transitive) and `deps/incomplete-scan` | each vulnerable package named, with the safe version | ✅ 2026-10-30 |
| 5 | Tell agents: MCP `check_dependency`, the hook (inform or block), `rules_for_path`, the skills | the agent uses the fixed version unprompted | ✅ 2026-10-30 |
| 6 | The two PyPI and two npm names — [W-134](../../work/open/W-134-deps-pypi-pending-publishers.md), [W-135](../../work/open/W-135-deps-npm-first-publish.md), just before the release | the pack can be released | ↗ split out 2026-10-30 |

**Arpit, 2026-10-29:** *"I want the URLs configurable … multiple sources … I can mark them as true or false … find a URL that will definitely not be blocked and keep it as the first one."* · *"Merge all the enabled ones."* · *"We build everything in single shot."*

**Model:** Opus (Claude Code): new network plumbing, a merge with a safety property, a pack split and four agent surfaces in one item.

## Goal

The design is [the proposal](../../work/proposals/deps-flags-markers-reviews.md) §2.1–2.4. Build it as written; where it is silent, decide and record the decision in the record.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · records | L5 decision 3 names the advisory and licence hosts; L6 → nineteen packs (17 npm, 12 PyPI); SR-RULE-PACKS decision 15 rewritten for the `deps` pack; SR-CONFIG `[deps] advisory_sources`, `severity` (four advisory grades), `agent`, `ignore` with `advisory`; SR-CLI-MCP `check_dependency`; SR-AGENT-SURFACES hook and `rules_for_path`; register rows; CLAUDE.md's L6 line | ✅ 2026-10-30 |
| 2 · pack | `packs/deps` on both registries, `deps/recommended`; `data/outdated-dependency` → `deps/outdated`, `data/stale-deps-snapshot` → `deps/stale-snapshot` through the deprecation path; `init` adds `deps` beside a manifest; `pack.rs` FIRST_PARTY_PACKS, packaging, docs-sync | ✅ 2026-10-30 |
| 3 · sources | `registry` (npm bulk advisories, PyPI per-version JSON — in-use, each `fixed_in`, latest) and `osv` (`/v1/query` by package) kinds; on/off; merge by aliases, safer answer wins; `sources.<name>` state in the snapshot; exit 2 only when every enabled source fails; licence metadata for versions in use (W-129 reads it) | ✅ 2026-10-30 |
| 4 · rules | `deps/vulnerable`: target = lowest known-clean ≥ in use, current major first, release-age and yanked rules kept; direct → range edit, transitive → npm/yarn/pnpm/uv override (Poetry manual); no clean version → manual; `deps/incomplete-scan` once per project | ✅ 2026-10-30 |
| 5 · agents | MCP `check_dependency` offline; post-edit hook message; `[deps] agent` inform/block per severity (default inform; never changes CI); `rules_for_path` names vulnerable and major-behind imports (B-052 promoted here); the skills package's instruction | ✅ 2026-10-30 |
| 6 · tests | sources served on 127.0.0.1 (one blocked, one disagreeing); the merge's safety cases; the network fence case extended; corpus counts; L8 budget measured | ✅ 2026-10-30 |
| 7 · docs | rule docs, USAGE, CHANGELOG, GLOSSARY (`advisory source`, `known-clean`), README counts | ✅ 2026-10-30 |
| 8 · names | moved to W-134 (PyPI) and W-135 (npm), 2026-10-30 | ↗ |

## Blockers

None. The release waits on W-134 and W-135. The yarn classic `yarn.lock` copy is built on W-136's `elsewhere` (landed first).

## Closed 2026-10-30 — what was decided where the proposal was silent

- **The rule rename** goes through a new `RENAMED_RULES` table in `crates/codelaw-config` (SR-CONFIG decision 5): an old id still loads, its setting moved to the new id, with a warning, for one minor release.
- **`uv.lock` and `poetry.lock` are not L3 files** (`.lock` is no TOML extension), so their transitive findings sit on the `pyproject.toml` in the lockfile's folder, as yarn's sit on `package.json`; uv's edit adds `[tool.uv] override-dependencies`, Poetry's is manual. The design said "uv/poetry TOML"; detection by extension does not read them.
- **A lockfile-located finding (package-lock, pnpm) carries no edit** — an edit can only be in the file the finding is on; the message names the exact `overrides` / `pnpm.overrides` text. Overrides pin the exact target, never a range, so nothing unverified can resolve.
- **An advisory no source grades counts as `high`** (the safer answer); the message says no source graded it.
- **Coverage is what a source was asked:** npm's bulk endpoint is sent every kept version of a vulnerable package in a second pass, and PyPI's JSON is asked for each fixed version and the latest, three rounds at most.
- **`[deps] agent`:** `block` makes the pre-write Gate refuse a write that adds the finding and the post-edit hook say so; `inform` never refuses on its account. CI never changes.
- **`registry` sources take optional `npm_url` / `pypi_url`;** `osv` takes `url` (default `https://api.osv.dev`). PyPI's JSON API is the index URL with `simple/` replaced by `pypi/`.
- **L8:** `codelaw check` on a 3,000-entry `package-lock.json` with 300 vulnerable packages took 0.55 s with the debug build (2026-10-30, a machine shared with five agents' builds); no release build was made (L9 disk).
- **Corpus:** `deps` is in `scripts/corpus.py` and covers four corpus repos; its counts are 0 by construction — no corpus repo commits a snapshot — so no corpus run was made.

## Record affected

L5 decision 3 · L6 decisions 11–12 · SR-RULE-PACKS decision 15 · SR-CONFIG §`[deps]` · SR-CLI-MCP decisions 1 and 5 · SR-AGENT-SURFACES · SR-PACKAGES (names).

## Hazards

- **The safety property is the point:** a version is offered only if every merged advisory leaves it clean and a source covers it. Test the case where PyPI's registry source has not queried the candidate.
- npm's bulk endpoint gives no fixed version — derive it from `vulnerable_versions`; private mirrors may not proxy it (→ `unavailable`, not empty).
- OSV is refused on some networks (seen on the decider's Mac); the registry source must work alone.
- Overrides change the install of the whole tree — the message says so; never edit a lockfile.
- L11: no new crate (semver, PEP 440 exist); L2: OSV/GHSA data is CC-BY 4.0 — the snapshot keeps ids, ranges, urls and a source line, not advisory text bodies.
- The renames break configs naming `data/…` — the deprecation path must report them with the fix.

## Reference

[Proposal](../../work/proposals/deps-flags-markers-reviews.md) · `archive/open/W-52-dependency-latest-versions.md` · `crates/codelaw-cli/src/deps_cmd.rs` · `crates/codelaw-native/src/deps/`
