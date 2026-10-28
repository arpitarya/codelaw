---
type: Handoff
name: W-77
description: "Arpit, 2026-10-27: move every agent-facing file codelaw ships — skills, steering, instructions, agent files, hook and MCP registration — out of the codelaw package into its own package, @codelaw/skills on npm and codelaw-skills on PyPI, which depends on codelaw at the exact same version."
item: W-77
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-77 — A separate package for everything agent-facing

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write the rulings into the records: the law on packages, the packages record, the agent-surfaces record, the dependency list | the split is the spec before any code moves | ✅ |
| 2 | Build `@codelaw/skills` (npm) and `codelaw-skills` (PyPI): the agent files, plus a hard dependency on the same codelaw version | one install brings the skills and the engine they were written for | ✅ |
| 3 | The codelaw binary stops carrying the agent files; `init --agents` and `sync-agents` read them from the installed skills package | codelaw is just the linter and formatter; the skills update as their own package | ✅ |
| 4 | Missing or mismatched skills package → a clear message and exit 2, never an old copy | nobody gets skills written for a different engine | ✅ |
| 5 | CI builds, checks and publishes both new packages beside the rest, at one version | they ship in every release, like the rule packs | ✅ (CI's first run proves it) |
| 6 | Arpit reserves the two names and publishes them the first time by hand | the registries accept CI's uploads after that | 🔄 npm ✅ — `@codelaw/skills@0.2.0` published and trusted (checked 2026-10-29); PyPI `codelaw-skills` → [W-88](W-88-pypi-day-3.md) (day 3) |

**Arpit, 2026-10-27:** *"I want a separate package to be created just for skills, and it will have a hard dependency on code law. Skills being agent facing scripts or files."*

## Rulings (Arpit, 2026-10-27, answering four questions)

| question | ruling |
|---|---|
| just the 12 skills, or the whole agent-facing set | **the whole set** — skills, steering, instruction and rules files, custom agent files, agent hook configs, MCP registration, trigger evals: everything under `templates/agents/` |
| which registries | **npm and PyPI** — `@codelaw/skills` and `codelaw-skills`; a Python-only repo keeps parity |
| what the codelaw package keeps | **nothing agent-facing**; `init --agents` / `sync-agents` need the skills package installed and say so when it isn't (exit 2) |
| how tight the dependency is | **the exact same version** — skills `X` requires codelaw `==X`, released together |

**Model:** Opus: it amends a law and three records, and touches the engine, both packagings, CI and release at once.

## Goal

`codelaw` is the engine only. Everything an agent reads or a hook config wires up lives in one package that a team can install, pin and upgrade on its own, and that can never run against an engine it wasn't written for.

## Where things are today (2026-10-27, read from the tree)

- The sources are `templates/agents/**` — 12 skills, `agents/codelaw-rule-author.md`, steering, Copilot instructions, Claude/Codex/Copilot/Kiro/Cowork hook and MCP configs, `evals/triggers.json`.
- They are **compiled into the binary**: `codelaw_core::template::TEMPLATES`, read by `crates/codelaw-agents/src/skills.rs` (`SKILLS`, `rule_author()`, `trigger_evals()`) and `sync.rs`; `codelaw-cli` calls `codelaw_agents` for `init --agents`, `sync-agents` and `hook`.
- SR-AGENT-SURFACES says *"Skills ship inside both packages (L6)"*; SR-PACKAGES says every published pack has **zero runtime dependencies** and `scripts/package.py check` enforces it.

## Design (proposals, not rulings)

- **Source stays in `templates/agents/`** (L10 unchanged: the files written into a consumer's repo still come from templates). The skills package is that folder, byte for byte, plus a manifest naming its version.
- **What stays in the engine:** the commands — `codelaw hook`, `codelaw mcp`, `init`, `sync-agents` — and the code that fills the templates' slots. Only the *files* move.
- **Finding the package** without a network (L5) or a Python/Node runtime call: npm, `node_modules/@codelaw/skills/` walking up from the project root; PyPI, `codelaw_skills/` beside the `codelaw` module in the same environment (the binary knows its own path). `codelaw.toml` may name a path to override.
- **Version check:** the skills manifest's version must equal the binary's; otherwise exit 2 naming both and the install command.
- **`sync-agents --check` in this repo's CI** reads `templates/agents/` directly, so the repo checks itself without a published package.
- **The dependency is one-way:** codelaw does not depend on the skills package; installing `@codelaw/skills` brings `codelaw`.

**Closed 2026-10-29 — merged into PyPI day 3 (Arpit, text):** *"Okay, W34 can be archived and it can be merged with PyPy day 2, day 3, day 4 items. W77, the npm package is published, the PyPy package merge it with PyPy day 2, day 3, day 4 or create a new one day 5. The package is already created, I guess."* `@codelaw/skills@0.2.0` is on npm and trusted; `codelaw-skills` is **not** on PyPI yet (pypi.org answered 404, 2026-10-29 01:21), and it is already in day 3 — [W-88](../../work/open/W-88-pypi-day-3.md) — so no day 5 is needed.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · records | L6 lists the two packages; SR-PACKAGES §1 gains them and names the one allowed runtime dependency (codelaw, exact); SR-AGENT-SURFACES moves "ship inside both packages" to the skills package; L11's list ratifies `codelaw` as the skills package's dependency (L11, Arpit 2026-10-27) — all in the same change as the code | ✅ L6 decisions 4, 10, 13 + enforcement; SR-PACKAGES §1, decisions 2, 3b, 7, 11; SR-AGENT-SURFACES decisions 1, 7, 8; SR-LAW-10 decisions 6–7 (the `agents/` exception); L11's list D-35 — same change as the code |
| 2 · packages | `packaging/npm/skills` → `@codelaw/skills` (`dependencies: { "codelaw": "X" }`); `codelaw-skills` pure-data wheel (`py3-none-any`, `codelaw==X`); both generated like every other manifest (SR-PACKAGES) | ✅ `scripts/package.py` `stage_skills` → `data_tarball` (`codelaw-skills-<v>.tgz`, `dependencies: {codelaw: v}`) and `data_wheel` (`codelaw_skills-<v>-py3-none-any.whl`, `Requires-Dist: codelaw==<pep440>`); README `packaging/npm/skills/README.md`; manifests generated from the one version; `stage-skills <dir>` for a local copy |
| 3 · engine | the binary no longer embeds `templates/agents/**`; `init --agents`, `sync-agents` and the hook installer read the installed package; output identical to today's for the same version (golden test) | ✅ `TEMPLATES` holds no `agents/` file (test refuses one); `crates/codelaw-agents/src/package.rs` finds the package (`CODELAW_SKILLS`, `node_modules/@codelaw/skills` upwards, site-packages, the binary's own env); golden: the assembled npm tarball and PyPI wheel each give the same 79 files, byte for byte, as the pre-W-77 binary on a 5-target consumer; `commands.rs` requires npm = PyPI |
| 4 · refusals | no skills package → exit 2 with `npm i -D @codelaw/skills` / `pip install codelaw-skills`; version mismatch → exit 2 naming both; tests for each, on both packagings | ✅ none → exit 2, both install lines, where it looked; other version → exit 2 naming both, on the npm and the PyPI layout; `init --agents --no-install` refuses before writing; `the_skills_package_is_found_or_refused`, `package::tests` |
| 5 · release | `scripts/package.py check` knows the set and the exception; version script bumps both; `ci.yml` builds and checks; `publish.yml` uploads; `fixtures/` gains a consumer that installs both | ✅ `package.py check`: names, the one exact dependency, npm = PyPI = `templates/agents/` + manifest; `version.py` docstring (generated, nothing to write); `ci.yml` (assemble/check already cover it); `publish.yml` publishes it after `codelaw`; `smoke.sh` installs both by name and diffs `sync-agents` output in `fixtures/skills-consumer`; `npm-trust.sh`/`withdraw.sh` read `names` |
| 6 · publish | names reserved and first-published by hand, trusted publishers added — filed as an inbox row when DoD 5 is green | 🔴 npm ✅ 2026-10-27 — `@codelaw/skills@0.2.0` published and trusted (W-85); PyPI → [W-88](W-88-pypi-day-3.md) (PyPI, Fri 2026-10-30), ruled 2026-10-27 |
| 7 · docs | README install lines; `RELEASING.md`; CHANGELOG (breaking: `sync-agents` now needs `@codelaw/skills` / `codelaw-skills`) with a migration note; `IMPLEMENTATION.md` | ✅ README, USAGE, `packaging/README.md`, `RELEASING.md`, `templates/README.md`, CHANGELOG (*Changed*, with *Before you upgrade*), `IMPLEMENTATION.md` |

## Blockers

None to build — built 2026-10-27. **DoD 6 is Arpit's**, filed as an inbox row in `OPEN-WORK.md`: reserve `@codelaw/skills` and `codelaw-skills`, publish the first version of each by hand (npm after `codelaw` at that version), then `scripts/npm-trust.sh` and a PyPI trusted publisher. Until npm trusts `publish.yml` for `@codelaw/skills`, the release preflight stops before anything is uploaded.

## Record affected

[SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) (the package list), [SR-PACKAGES](../../records/0107_packages.md) §1 and the zero-runtime-dependency rule, [SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) (where the files ship), [SR-LAW-11](../../records/0013_LAW-11-dependencies.md) via L11's list.

## Hazards

- **A breaking change for every consumer that runs `sync-agents` or `init --agents`.** Ship it in a minor bump with the migration note; the refusal message is the migration path.
- **Two copies drift.** If the binary keeps any agent text "as a fallback", the split is fake — DoD 3 removes it entirely.
- **Python discovery.** A `pipx`/`uvx` codelaw and a project-venv `codelaw-skills` live in different environments; the lookup order and the override must cover it, and the error must say where it looked.
- **L6 parity.** npm and PyPI must produce byte-identical agent files for the same version — the golden test runs on both.
- **L9 for the building session:** check `target/` against the 5 GB budget ([SR-LAW-9](../../records/0011_LAW-9-machine-budget.md)) before building.

## Reference

- `templates/agents/` · `crates/codelaw-agents/src/{skills,sync,hook}.rs` · `crates/codelaw-cli/src/{init,main}.rs`
- [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) · [SR-PACKAGES](../../records/0107_packages.md) · [SR-AGENT-SURFACES](../../records/0102_agent-surfaces.md) · [SR-LAW-10](../../records/0012_LAW-10-consumer-templates.md) · [SR-LAW-11](../../records/0013_LAW-11-dependencies.md)
