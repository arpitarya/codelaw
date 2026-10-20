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

*Updated 2026-10-20 (the merge of W-04's pipeline).* The laws L0–L8 are accepted (L8's budgets unset; L6 host
floors Python ≥ 3.11, Node ≥ 22); every record 0101–0115 is accepted. W-01
rewrote the records for five rule packs; **W-02 built the engine**: a Cargo
workspace of 13 crates under `crates/` and the `codelaw` binary (`check`,
`fix`, `test`, `rules`, `explain`, `tree`, `new-rule`, `init`, `cache`), with
the semantic model, `native:` rules and pack resolution. Every crate is owned in
the register's Ownership table. W-03 steps 2–5 are done: five packs under `packs/` — js 132 rules, python 98,
html 71, css 70, data 30 — every rule `preview`, each pack's tests green and
checked against its upstream linter on third-party code. **W-04 built the
release pipeline**, merged into `main` 2026-10-20 (green on all eight
platforms, nothing published): `ci.yml` builds and checks all 27
packages on every push, `publish.yml` uploads exactly that set by OIDC. The
licence is MIT (Arpit, 2026-10-20; it superseded `MIT OR Apache-2.0`). **W-06**
added the public docs — `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`,
`SECURITY.md`, `CODE_OF_CONDUCT.md`. **The engine is complete since
2026-10-21:** the formatter (W-07), MCP (W-08) and LSP (W-09) have landed, and
`law-gate.py` lists nothing. Not built: graph, baseline, adapters, agent
surfaces, the rest of SR-FORMATTER (W-23) — open items W-10–W-23.
`cargo test --workspace` is green (204 tests; every pack's `codelaw test`
green). **Clippy is not clean:** CI's clippy step can't fail (no `pipefail`)
and hid three warnings — W-17, W-18. **A release is refused by
`scripts/law-gate.py`** until `format`, `mcp` and `lsp` exist (L7, L6
decision 3); the 2026-10-20 review's findings are W-17, W-18 and W-22. **Ground it before you edit it** — `git log`, the queue,
`cargo test`.

## 2 · In flight, and the immediate next step

**W-05** (🟢, `research`, Cowork): guidance **while** an agent writes — a construct-filtered `brief` before writing and a `hook --pre` check before bytes land; Fux only as an optional free-text layer. Next is step 1, the per-agent pre-write matrix. Design is a proposal until step 5 is ruled on.

**2026-10-20, cloud session (stopped by the decider to continue locally):**
W-08, W-17 and W-22 landed and are archived; W-09 landed except formatting
(waits on W-07); W-18 landed except the tag-only environment rule (decider,
inbox) and the staging-index install. Unfinished, unbuilt work sits on pushed
branches `wip/W-07-formatter`, `wip/W-08-mcp-W-10-agent-surfaces` (W-10, on
top of W-08), `wip/W-11-graph`, `wip/W-12-structured-rules`,
`wip/W-14-adapters`, `wip/W-16-commands-api`, `wip/W-20-conformance`,
`wip/W-21-pack-gaps` — each is one WIP commit (most with `--no-verify`) on a
base before the final merges: rebase or merge `main` in, build, finish,
and rewrite the WIP commit so the SR guard passes before it lands. W-15 and
W-19 were not started.

**2026-10-21, local session:** W-07 landed from `wip/W-07-formatter`, and
W-09 with it (LSP document formatting). `codelaw format`, the format step of
`fix`, and `format/unformatted` in `check`, the editor and MCP's snippets all
use six embedded engines behind one verifier. Parity against Prettier and
Black is recorded in SR-FORMATTER: JS/TS/JSON are 100 % identical, HTML only
62 %. What W-07's DoD left of SR-FORMATTER is W-23. **One reconciliation:**
`end_of_line` unset keeps each file's line ending (SR-ARCHITECTURE decision
9); the WIP had rewritten CRLF to LF, which matches Prettier but breaks that
record.

**Next for an agent:** W-10 (agent surfaces) from
`wip/W-08-mcp-W-10-agent-surfaces`, the W-10 commit only.

**Open questions for the decider (from 2026-10-20 agents):** a
`rustfmt.toml` and a one-time format of the tree (the code isn't
rustfmt-formatted, so `cargo fmt --all` rewrites ~94 files); whether the html
pack's `entities.json` (WHATWG table, CC BY 4.0, taken via html-validate's MIT
copy) passes L2; renaming `yaml/document-start` (shares yamllint's name;
NOTICE now says so); `codelaw check <file>` with no config takes the file's
own folder as the project root.

**W-04** (🔴): the pipeline is built, timed and merged; the first release waits on the
decider's one-time registry setup (`RELEASING.md` §One time, steps 1, 4, 5 —
the parts that publish nothing). W-07 and W-09 landed 2026-10-21, so the
engine L7 and L6 decision 3 require is complete; the ruling to publish is the
decider's.

**W-06** (🔴, filed as W-04): the decider places the five `.github/` templates. The
README's counts (401 rules, 3,060 checks, fix classes) and roadmap must move
with the packs — its registry row names the trigger.

**W-03** (🔴) step 6 is that same release; the packs also build locally with
`scripts/build-packs.py`. Rules that need engine
features not built wait in each pack README's "not ported" table: `target:`
rules (W-12), the graph (W-11), a native rule's file path and bytes (W-21),
a CSS grammar that parses modern CSS (W-21). For the decider to confirm or
correct: every pack rule ships `preview`; upstream's noisy defaults kept in
`html/require-sri`, `html/valid-id`, `html/button-type`; `json/package-engines`
`>=22` and `toml/pyproject-requires-python` `>=3.11`.
The decider (`git config user.name`) still installs `scripts/sr-guard.sh` as
the `commit-msg` hook.

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
