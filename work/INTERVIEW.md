---
type: Handoff
description: "Cold-start state of play for a successor session: in-flight work, constraints, lessons."
---

# Model handoff — the agent-succession record

*The departing model answers for the record, so whatever succeeds it inherits
the judgment and not just the files.*

**Standing instruction to every future model reading this: you are now the
maintainer of this document. When the project's direction, decisions or state
change, update it in the same change. Leave it better.**

## How to use this file

A state-of-play doc, not an end-of-session report. Keep it current **during**
the session. Write it for a different model arriving with zero other context,
mid-task. The four sections below are maintained; anything after them is
history. **A stale INTERVIEW at handoff is as serious as a missing worklog
entry.**

---

## 1 · State of play

**2026-10-26: L11 ratified** — no dependencies unless the decider ratifies each by name in `DEPENDENCIES.md` (ruled 2026-10-26: 29 ratified, `strsim`/`anyhow`/`postcard` replaced by W-67; W-65 built the CI check: `crates/codelaw-cli/tests/dependencies.rs`).

**2026-10-26 docket:** the inbox holds only W-38 (PyPI by hand). Ruled: watch polls (no `notify`), hook passes no profile, pre-commit `language: system` now + W-64 mirror later, CSS/SCSS rules move to the tree-sitter-css fork + tree-sitter-scss (B-047 → W-62), W-45's PR is Claude Code's.

**2026-10-26 (Cowork): L10 ratified** — every file codelaw writes into a consumer's repo comes from a template in `templates/` ([SR-LAW-10](../records/0012_LAW-10-consumer-templates.md)); not built yet (W-60). W-59 renames `[[overrides]]` → `[scope.<name>]` and `[profiles.*]` → `[profile.*]`. W-58 adds max-lines / max-lines-per-function / max-params.

**2026-10-25 (Cowork): new feature work filed, not built.** CI-vs-local severity (profiles, agents on `ci`), a per-glob severity `remap`, dated exceptions (`expires`, `warn_days`, `require_expiry`, git-checked `added_by`) and `react/no-context` — W-39, W-40, W-41, rulings in [the compare doc](compare/ci-profiles-and-dated-exceptions.compare.md). Each build writes its record, schema and loader in one change: SR-CONFIG §1 is parsed by `crates/codelaw-config/tests/schema.rs`.

**Built 2026-10-26 (Claude Code, lockmaster): W-54 `security-js`** — 66 rules, 55 in `sec-js/recommended`, 11 more in `strict`; archived. Two catalogue fix classes could not be met with the edit primitives (`no-bidi-characters`, `no-cleartext-http`); W-55 builds on it.

**Built 2026-10-25 (Claude Code, six parallel worktrees, merged):** W-47 baseline history, W-52 `deps refresh` (L5 amended: the one networked command), W-53 security-pack groundwork (L6: fifteen packs) — closed; W-44 watch, W-45 CI formats, W-50 git hooks — built, each waiting on a ruling or a PR from the decider. W-54–W-57 are now open to an agent.

**Built 2026-10-26 (Claude Code):** W-57, the `security-python` catalogue — 61 `match:` rules (Bandit and Ruff `S` ported; Django, Flask, FastAPI, ML loaders and LLM rules), `sec-py/recommended` (57) and `strict` (+4), the README's Bandit accounting. Still `preview`: no rule is `stable` until its corpus count is reviewed, and the PyPI upload of `codelaw-rules-security-python` is the decider's, by hand (W-38's pattern). Three rules the catalogue called `suggest` are `manual`: `no-xpath-injection`, `no-xmlrpc`, `no-cleartext-http`.

**Also filed 2026-10-25 (Cowork):** W-44 watch mode · W-45 CI formats · W-46 exceptions report (waits on W-40) · W-47 baseline history · W-48 `migrate` (ESLint, Ruff, stylelint, Biome; no oxlint — moved to the backlog as B-050 the same day) · W-49 dev-server plugins (waits on W-44) · W-50 git hooks. Then W-51 (scheduled auto-fix PR) and W-52 (dependency latest versions; the decider ruled `deps refresh` the one network command). Type-aware via typescript-go: parked as B-049 with [a proposal](proposals/type-aware-rules.md). Security: W-53 (groundwork, amends L6 to fifteen packs) then W-54–W-57 (`security-js`, `-ts`, `-html`, `-python`).

*Updated 2026-10-24 (the parallel session: W-20, W-21, W-24–W-28 merged to `main` `82df231`, unpushed).*
**Where things stand:** the laws L0–L8 are accepted (L8's budgets still
`unset`); every record 0101–0115 is accepted. **The engine is complete**
(L7, L6 decision 3; `law-gate.py` lists nothing): 20 crates under `crates/`,
the `codelaw` binary with `check`, `fix`, `format`, `test`, `rules`,
`explain`, `tree`, `new-rule`, `init`, `cache`, `graph`, `config`, `match`,
`hook`, `sync-agents`, `baseline`, `deprecate`, `daemon`, `bench`, `mcp`,
`lsp`. **Nine packs since W-31 (2026-10-24)** — `js`, `ts`, `react`,
`node`, `stencil`, `html`, `css`, `python`, `data`; eight on npm, six on PyPI,
the split in `FIRST_PARTY_PACKS` (pack.rs) — 419 rules, 3,949 example checks,
every pack green; every rule `preview`. W-01–W-02, W-06–W-17, W-22–W-23 are closed and archived.
Verified on `main` `f75c848` in a cloud clone (rustc 1.97 — the pinned 1.99.1
cannot be downloaded there): clippy clean, `cargo test --workspace` green,
every pack 0 failed. **0.1.0 is on npm** (2026-10-24, W-29: the five-pack
layout, 14 names, trusted publishing bound), published by hand from a
failed CI run's `dist`; **PyPI has nothing** and starts at 0.2.0 (W-30).
**W-31 is committed** (`da9b207`) and `@codelaw/rules-python` is off npm;
0.2.0 first-publishes `rules-ts`, `-react`, `-node`, `-stencil` by hand.
**The review's fixes are merged, not pushed** (2026-10-24): W-24's seven CI
fixes, W-27's hygiene (MSRV is **1.97**, the pinned `ruff_*` crates' floor),
W-28's token-stream layout check, W-25's register and inbox-age rules, W-26's
`docs-sync.py` and `*/project` presets, W-20's pinned corpus, W-21's patch
checked against upstream. On `main` `82df231`, rustc from `rust-toolchain.toml`:
clippy `-D warnings` clean, `cargo test --workspace` 449 passed, `codelaw test`
on all nine packs 0 failed, 0 advisory warnings.

**What the 2026-10-22 review found (filed 2026-10-24 as W-24–W-28):** CI
cannot pass on `main` as it stands — `bench/mod.rs`'s `not(unix)` `wait`
does not compile (both Windows builds red, so no `dist`, no attestation, no
release), the SR-guard replay fails nine as-is WIP merges, the verdict cache
keys miss inputs the tests read, the law gate still runs `--report` (W-24).
The records and work files drifted from the tree — `codelaw-graph` and
`perf/` have no owner, the register's `built` column is stale for seven
records, dead `B-nnn` ids are cited, item files name landed blockers (W-25).
The README's roadmap, command table and the CHANGELOG lag the binary (W-26).
`rust-version = "1.85"` is false (let-chains need 1.88), one process-kill
is duplicated, Windows has no `daemon` and nothing says so (W-27). The
layout-rule check is defined by what Biome does, not by codelaw (W-28 — the
decider: *"biome is not the right thing to reference; build how we design
it"*).

**The decider's rulings of 2026-10-22** (merged 2026-10-24 as `f75c848`):
npm's Artistic-2.0 passes L2; `js/no-irregular-whitespace` narrowed and
`css/media-query-no-invalid`'s example rewritten (W-23 closed; B-045 holds the
formatter's meaning change); `tree-sitter-css` forked in-tree with the
upstream patch in `crates/codelaw-lang/grammars/tree-sitter-css/upstream/`
(W-21 steps 3–4); the `.github/` templates placed (W-06 closed). **Still the
decider's (inbox):** the one-time registry setup (W-03, W-04), the
`npm`/`pypi` environments with the `v*` tag rule (W-18) — both answered
"later, keep the row" on 2026-10-22 — and L8's budgets after the `perf`
job's first run on `main` (W-19). **The push of `main`** is the decider's,
after W-24 lands (ruled 2026-10-22: "push after the review's fixes land").

**Ground it before you edit it** — `git log`, the queue, `cargo test`.
Concurrent sessions are the norm here: the 2026-10-22 rulings sat on a branch
for two days because another session's edits were uncommitted in the main
checkout.

## 2 · In flight, and the immediate next step

**Release state, 2026-10-25 (Cowork):** all 19 npm names exist and trust `publish.yml`, but the engine and eight packs are still 0.1.0 there. **W-37 (Claude Code, now):** commit the npm-only `publish.yml` run (L6 decision 9 amended on Arpit's ruling), Arpit pushes, tag `v0.2.0`, run it, check the 19 names. **W-38 (🔴 Arpit, 2–3 days):** the four remaining PyPI packs by hand (four new projects a day), six trusted publishers, then `gh release create v0.2.0` on the same tag brings `codelaw` to PyPI. W-03, W-04, W-29–W-32 are closed and archived into those two; the *later* presets are B-048.

**Next:** the decider pushes `main` — CI's first run proves W-24 (green),
W-27 (the `msrv` job, the win32 smoke) and gates L8's budgets. **Next for an
agent:** W-32's two packs (steps 2–4), W-19's `perf` pin and the three repos in `bench`; W-20's snapshots, W-25 and W-26 are done (2026-10-24). W-05 is Cowork research only (rule 37a). **Run one cargo build
at a time on this machine** (LESSONS 2026-10-24).

**Docket 2026-10-24:** W-21 closed (fork kept, the PR is B-046, DoD 3 restated
engine-only, a CSS-tree port is B-047); W-32 named (`rules-redux-toolkit`,
`rules-tanstack-query` — renamed from `react-query` 2026-10-25 — Redux Style Guide); W-28's token rule confirmed; W-20's
build-and-test leg dropped; W-19's machine (`ubuntu-24.04`) and three repos
named; step 5 of the registry setup done. Left in the inbox: PyPI step 4, L8's budgets.

**2026-10-25:** `main` pushed and **CI green on every job for the first time** (run
40124005285) — W-18, W-20, W-27 closed; W-24 waits only on a docs-only push skipping
`perf`. The L9 / SR-WORK-NOW / SR-WORK-CLEANUP hooks exist and are wired (written
from the records; `target/` cleaned on the decider's ruling). `bench --repo` times the
three pinned repos (W-19). On `ubuntu-24.04` three p95s sit above their Mac
proposals (`check_cold_10k` 8701 ms, `format_warm_10k` 1353 ms, `mcp_rules_for_path`
40.8 ms), so the proposals must be re-applied from CI's runs before the decider rules.

**Docket 2026-10-25:** `main` pushed; L8's budgets ruled by method (min(target, 1.5 × worst CI p95), a capped one back to the decider; W-19 applies it with `gh` on the Mac); PyPI step 4 — Arpit logs in, Claude adds the seven; 0.2.0 bumped in the tree (uncommitted), command list in W-31; W-29's 8-platform check moved to 0.2.0; W-33 filed (two Stencil rules).

**2026-10-25 (Cowork, W-05):** the Fux layer is ruled — each pack's Fux index is its own optional package, built in codelaw's CI, on npm and PyPI (W-34: built 2026-10-26 — L6 decision 12, `scripts/fux-index.py`, `init` only prints the install lines when `.fux/` exists; left: Arpit reserves and publishes the 16 names, and ratifies `fux-engine` as D-33). W-35 puts "before, while and after the AI writes" on the README, the while beat labelled until W-05's `hook --pre` ships. W-05's next step is its step 1 payload capture (Codex, Copilot CLI, Kiro).

**2026-10-25 (Cowork, README):** the README is the pitch (story, four moments, the ESLint/Prettier/Biome comparison) and `USAGE.md` the how-to; `docs-sync.py` checks both. W-36 (research) checks the claims against the incumbents and finds the adoption gaps.

**Open questions for the decider, carried from earlier sessions:** W-11's two default
gaps (self-aliased private packages; CSS only for linked stylesheets); W-13's
`on_touch` default for non-deprecation baselines and `codelaw/stale-baseline`
as `manual`; W-15's per-project tree-cache bound; a `rustfmt.toml` and a
one-time format of the tree; whether the html pack's `entities.json` (WHATWG,
CC BY 4.0 via html-validate's MIT copy) passes L2; renaming
`yaml/document-start`; `codelaw check <file>` with no config taking the
file's folder as the project root; every pack rule shipping `preview`;
upstream's noisy defaults kept in `html/require-sri`, `html/valid-id`,
`html/button-type`; `json/package-engines` `>=22` and
`toml/pyproject-requires-python` `>=3.11`.

**W-29 / W-30** (🟡, `arpit`): the first publish, by hand from Arpit's machine
(his ask, 2026-10-24), npm then PyPI, from one CI `dist`; the commands are in
the item files. The npm org `codelaw` exists (checked 2026-10-24).
**W-04 / W-03** (🔴): built, timed, merged; the first release waits on the
registry setup (the go was given with npm 0.1.0; `BLOCKED.json` is `PROCEED`). **W-18** (🟢): the environments
and their tag rule set 2026-10-24; CI's first run proves the rest. **W-19** (🟢): pin `perf` to `ubuntu-24.04` and bench the three repos; the budgets after CI's first `perf` run.

## 3 · Standing constraints

- The laws: L0 (records are the only source; links, never restatements), L1
  (retired records are archived), L2 (every dependency is free, open source and
  permissive — no paid, no copyleft, dev tooling included), L3 (nine
  languages, alone or combined), L4 (every rule has a fix), L5 (no model in the
  engine), L6 (`codelaw` on PyPI + npm, identical; rules as five packs on both;
  npm sub-packages only `@codelaw/*`), L7 (linter and formatter, always both),
  L8 (performance budgets: never exceeded, never loosened). See
  `records/0001_LAWS.md`.
- Adapters must pass L2 too: yamllint (GPL-3.0) is out.
- The queue is the list; a blocker is a file; nothing closed is deleted.
- A change under `crates/` stages its owning record (the guard enforces it once
  installed); `crates/codelaw-cli/tests/` holds the layer, licence and law
  checks.
- Rule files are YAML 1.2 (SR-RULE-FORMAT decision 3d); `where:` and the fix
  primitives are SR-RULE-FORMAT decisions 3a–3c.
- **Releases come only from `publish.yml`, and it compiles nothing** — it
  uploads the `dist` a green `ci.yml` run on `main` assembled (SR-PACKAGES
  decisions 11–12). The version is set only with `scripts/version.py --set`.
  `ci.yml` and `publish.yml` keep their names: the registries' trusted
  publishers are bound to them. The Rust toolchain is pinned in
  `rust-toolchain.toml` (SR-PACKAGES decision 13); bump it in its own commit.

## 4 · Lessons learned

- **Skip the compile, not the checks** — a push that changes no engine source
  restores its eight wheels from cache; the binary's tests still run
  (`work/LESSONS.md`, 2026-10-20).
- **Run a new rule over real code** — its own examples passed while eight
  false-positive patterns hid; a corpus run found them (`work/LESSONS.md`).
- **Write a benchmark's output outside the folder it checks** — otherwise the
  second run checks the first run's output (`work/LESSONS.md`, 2026-10-19).
- **YAML 1.1 booleans** turned `message: n` into `false`; rule YAML is read as
  1.2 now (`work/LESSONS.md`, 2026-10-19).
