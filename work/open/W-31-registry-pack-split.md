---
type: Handoff
name: W-31
description: "Arpit, 2026-10-24: publish on each registry only the packs its users install — rules-js split into js, ts, react, node and stencil; eight packs on npm, six on PyPI; 0.2.0 is the first release in that layout."
item: W-31
filed: 2026-10-24
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-24
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: arpit
---

# W-31 — Each registry publishes only the packs its users install

## At a glance

| pack | npm | PyPI | rules |
|---|---|---|---|
| `js` | ✅ | ✅ | 72 — `js/*` and `graph/no-unused-exports` |
| `ts` | ✅ | — | 21 |
| `react` | ✅ | ✅ | 21 |
| `node` | ✅ | — | 10 |
| `stencil` | ✅ | — | 16 |
| `html` · `css` · `data` | ✅ | ✅ | 72 · 71 · 37 |
| `python` | — | ✅ | 99 |
| **total** | **8 packs** | **6 packs** | **419** (unchanged) |

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Split `packs/js/` into `js`, `ts`, `react`, `node`, `stencil`; each pack its own manifest, presets, data, README, NOTICE, LICENSE | one folder per published pack | ✅ 2026-10-24 |
| 2 | `FIRST_PARTY_PACKS` (pack.rs) names each pack's npm and PyPI name; resolution, the missing-pack error and `codelaw init` follow it | the engine finds a pack only where it is published | ✅ 2026-10-24 |
| 3 | `package.py` reads that table; `assemble`, `check`, `names`, `build-packs.py`, the bench workload and the docs follow | 17 npm tarballs + 14 wheels, one list | ✅ 2026-10-24 |
| 4 | L6, SR-RULE-PACKS, SR-PACKAGES and the records that describe the touched files amended | the records say what the tree does | ✅ 2026-10-24 |
| 5 | Arpit unpublishes `@codelaw/rules-python` from npm **before 2026-10-27 03:18 IST** (the 72-hour window) | npm carries no Python pack | ✅ 2026-10-24 13:43 IST: deleted on npmjs.com after Arpit's 2FA; the registry answers 404 |
| 6 | Arpit commits and pushes; CI goes green on `main` (W-24) | a `dist` in the new layout | ⏳ Arpit |
| 7 | 0.2.0: `version.py --set 0.2.0`, CI green, then the first hand publish of `@codelaw/rules-ts`, `-react`, `-node`, `-stencil` from that `dist`, `npm-trust.sh`, and the PyPI pending publishers incl. `codelaw-rules-react` (W-30) | the first release in this layout, on both registries | ⏳ Arpit |

**Arpit, 2026-10-24:** *"All the packages aren't needed to be published in npm or PyPI repo. The ones that are going to be used in npm and the ones that are going to be used in PyPI should only be published."* Then: *"in js have js, node, stencil as separate packs; in pypi have js, react as separate packs and remove ts, node and stencil from pypi"*, html kept on npm, and *"implement everything"*.

**Model:** Opus for steps 1–4 (landed); NONE for 5–7 — Arpit's hand at npm and PyPI.

## Goal

Each registry carries only the packs its users install: npm the eight JS-ecosystem and shared packs, PyPI the six a Python repo needs, one version, one release.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · tree | `packs/` holds nine folders; `cargo test -p codelaw-rules` loads each as its first-party pack and pins the 8 / 6 split | ✅ 2026-10-24 |
| 2 · packages | `package.py names` lists 17 npm and 7 PyPI names; `build-packs.py --release --verify` builds 14 pack artefacts, equal output from the packs on both, each layout loading all its own | ✅ 2026-10-24 (`build-packs.py`; `package.py check` runs in CI on the next push) |
| 3 · init | `codelaw init` installs only packs the chosen registry publishes, writes only their presets, and notes the rest (`commands.rs`, `consumer_layout.rs`) | ✅ 2026-10-24 |
| 4 · npm | `npm view @codelaw/rules-python` is a 404 | ✅ 2026-10-24 |
| 5 · released | 0.2.0 on npm (17 names) and PyPI (7 projects), trusted publishing bound for every name | ⏳ step 7 |

## Blockers

Steps 5–7 are Arpit's hand. Step 7 also waits on W-24 (CI green on `main`).

## Record affected

L6 decisions 9–11 and its law line (CLAUDE.md, SR-LAWS, the register); SR-RULE-PACKS §1 and decisions 2–6; SR-PACKAGES §1 and decisions 7, 9–11; and, for the files they own or describe, SR-RULE-FORMAT decision 1, SR-CONSUMER-LAYOUT decision 8, SR-RULE-AUTHORING level 4, SR-LINTER decision 9, SR-CLI-MCP decision 10 and SR-LAW-8's reference workload.

## Hazards

- **`codelaw@0.1.0` on npm still points `init` at `@codelaw/rules-python`.** After step 5, `init` in a 0.1.0 install with Python files asks npm for a package that is gone. Nobody is known to use 0.1.0; 0.2.0 fixes it.
- **`ts/recommended` no longer turns on `js/no-var`, `js/prefer-const`, `js/prefer-rest-params` or `js/prefer-spread`** — a preset may only name its own pack's rules. They are `codelaw:js/typescript` now, and `init` writes it for a TypeScript repo. A config written by hand with only `ts/recommended` loses those four (CHANGELOG).
- **`js/conventions` split** into `js/`, `ts/`, `react/` and `stencil/conventions`, for the same reason.
- **The reference workload's digest moved** (`8a5682d3ee6bf132` → `bf9c5bea0d8b31ee`) because its config gained `codelaw:js/typescript`; the same rules run. No L8 budget is set, so nothing is invalidated, but the `perf` numbers before this commit are on the old digest (W-19).
- **What each registry leaves out:** an npm-only repo gets no Python rules; a PyPI-only repo gets no `ts/*`, `node/*` or `stencil/*` rules. The decider's accepted cost (L6 decision 11).
- **A published version is permanent:** the 0.1.0 `@codelaw/rules-js` on npm keeps its ts, react, node and stencil rules.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- [W-29](W-29-first-npm-publish.md) (the 0.1.0 npm publish this follows) · [W-30](W-30-first-pypi-publish.md) (PyPI, now at 0.2.0) · [W-32](W-32-redux-and-query-packs.md) (two more npm packs).
- `crates/codelaw-rules/src/pack.rs` (`FIRST_PARTY_PACKS`) · `crates/codelaw-cli/src/init.rs` · `scripts/package.py` · `scripts/build-packs.py`.
