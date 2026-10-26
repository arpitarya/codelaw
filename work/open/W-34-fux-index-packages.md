---
type: Handoff
name: W-34
description: "Arpit, 2026-10-25 (voice): each rule pack gets an optional companion package holding a Fux index of its rules — @codelaw/rules-<name>-fux-index on npm, codelaw-rules-<name>-fux-index on PyPI — never inside the pack and never installed by default."
item: W-34
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Sonnet 5.5)"
ball: arpit
---

# W-34 — An optional Fux index package beside each rule pack

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Wait for Fux's spike (20 plain-words questions over the JS rules; bar 16 of 20 in the top 5) and Fux's design of a shipped index | proof that free-text search over the rules is worth a package | ✅ done — fux item 299 closed (fux 5.1.0; SR-PKG-INDEX, `fux pack`) |
| 2 | Amend the records: L6 names a third kind of package on both registries; SR-RULE-PACKS and SR-PACKAGES say what it holds and how it is built | the packages are legal | ✅ L6 decision 12, SR-PACKAGES decision 14, SR-RULE-PACKS decision 6 |
| 3 | Build: CI runs Fux over each pack's `rules/` and packs the index as `@codelaw/rules-<name>-fux-index` and `codelaw-rules-<name>-fux-index`, same version as the pack | 10 npm + 6 PyPI optional packages | ✅ `scripts/fux-index.py`; `ci.yml`'s `package` job; `dist/fux-index/`, not in the release set |
| 4 | `codelaw init` leaves them out; it only prints how to add them when the repo uses Fux | nobody downloads an index they did not ask for | ✅ `crates/codelaw-cli/tests/fux_index.rs` |
| 5 | Reserve the 16 names and publish with the next release | teams can install them | 🔴 Arpit's hand — open; RELEASING.md §"The optional Fux index packages" |

**Arpit, 2026-10-25 (voice):** *"I want codelaw to have a separate package for
Fux index. It shouldn't be added as a default one."* → *"I would like to go with
@codelaw/rules-js-fux-index."* → on PyPI: *"Both. It should be NPM as well as
PyPI."*

**Model:** Opus for step 2 (a law amendment); Sonnet for steps 3–4 once Fux item 298
has fixed the segment format.

## Goal

A team that uses Fux can add one package per pack and have `fux_search` find
codelaw's rules from plain words. A team that does not use Fux never downloads a
byte of it. The rules themselves stay in the rule pack; nothing is copied into
the consumer's repo.

## The shape ruled

- **One index package per pack**, released with its pack at the same version
  ([SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9).
  - npm: `@codelaw/rules-<name>-fux-index` for each of decision 11's ten npm packs.
  - PyPI: `codelaw-rules-<name>-fux-index` for each of its six PyPI packs.
- **Never inside the rule pack, never a dependency of it, never installed by
  `codelaw init` by default.**
- **Pure data, zero runtime dependencies** — the Fux index of that pack's
  `rules/`, nothing else. Fux reads it; codelaw's engine never does
  ([SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md): codelaw is
  complete with no Fux).
- **Fux is a build-time tool of codelaw's CI only** (`fux-engine`, MIT —
  [L2](../../records/0004_LAW-2-foss-dependencies.md) holds), pinned at one
  version so the index bytes are reproducible.

## What this needs that does not exist yet

- **Fux item 298** (in the Fux repo): how a package's index is pinned, how one query
  searches several indexes, and how a hit in the index package resolves to the
  rule file in the rule pack. Until it lands there is no format to build.
- **An L6 amendment.** Decision 10 says PyPI carries only `codelaw` and the six
  `codelaw-rules-<name>`, and *"no other codelaw distribution is published"*;
  decision 11's table lists only rule packs. Both change on this ruling.

## Questions (step 2), answered 2026-10-26 by the agent that built it

- **Does `codelaw init` print the index-package lines whenever `.fux/` exists, or only with a flag?** Whenever `.fux/` exists, under every flag, never when it does not. No new flag, so SR-CLI-MCP is unchanged. *The decider may overrule.*
- **Is the index package's version checked against the pack's?** By Fux's own pin: the pin row holds the index package's version and digest, and Fux's `pkg:` passage check (its item 301) refuses a docs package at another version than the manifest's `content.version`. `scripts/fux-index.py check` also refuses a manifest whose `content` is not the pack at the release version.

## What was built, and what is not the plan's

- **16, not 22.** The four security packs hold no rule file yet, Fux refuses an empty index, so they get theirs with their first rule: ten npm and six PyPI, the number this item always said.
- **Out of the release set.** `publish.yml` uploads `dist/npm` and `dist/pypi` whole and its preflight demands a trusted publisher per tarball, so an unreserved index name would stop a release. They build into `dist/fux-index/` and are a CI artifact (`fux-index`); step 5 ends with a change to `publish.yml`.
- **A manifest per registry.** `fux pack` writes both registry names; a pack on one registry has the other removed from the manifest's `content` (the manifest is outside the digest), so Fux never looks for a package that does not exist.
- **`fux-engine` is a dependency under L11** (D-33, `pending` in `DEPENDENCIES.md`): named by this item's own ruling as a build tool of CI only, but ratified by name only on the decider's say.
- **Proven end to end:** the js wheel and `codelaw-rules-js` installed beside fux 5.1.0: `fux setup --index` pinned it, `fux doctor` reported it verified with 74 documents, `fux ask "stop using debugger statements" --scope packages` ranked `rules/js/no-debugger.md` first.

## Definition of done

- ✅ The amended records name the 16 packages and cite this ruling.
- ✅ CI builds each index from a pinned fux and checks its digest is reproducible.
- ✅ `codelaw init` with no flag installs none of them (a test).
- 🔴 The packages are on both registries at the release version — Arpit's hand.

## Hazards

- **Sixteen more names to reserve and publish**, each with its own trusted
  publisher — the same work W-29/W-30 were.
- **A Fux release that changes the segment format** rebuilds every index package;
  the pin in CI is what keeps that a deliberate act.

**Records affected:** SR-LAW-6 (decisions 10 and 11), SR-RULE-PACKS, SR-PACKAGES,
SR-CLI-MCP (only if `init` gains a flag), `RELEASING.md`. None changed by this
file.
