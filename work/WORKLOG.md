---
type: Log
description: "Append-only per-session trail, date-grouped, newest first."
---

# WORKLOG — the append-only session log

Every session appends **one entry before it ends** — a chat-only session
counts. **Never edit a past entry; append only, newest on top.** A wrong old
entry is corrected by a new one. Distinct from `INTERVIEW.md`, which is the
*current* state of play. Rules: [SR-WORK-SESSION](../records/0054_WORK-session.md).

**Entry format:**

```
## YYYY-MM-DD — <one-line title>  ·  <surface> (<model>)

- **Asked:** *<the ask>*
- **Done:** <what landed, with commit shas where they exist>
- **Decided / left open:** <rulings taken in-session; forks put to the decider>
- **Mistakes:** <what went wrong and what was done about it, or "none">
- **Next:** <the single next step>
```

---

## 2026-10-20 — the licence ruled: MIT  ·  Cowork (Opus 5.5)

- **Asked:** *"Add mit licence for codelaw"*.
- **Done:**
  - `LICENSE` (MIT, © 2026 Arpit Arya, with `SPDX-License-Identifier: MIT`) at the root and copied into all five `packs/<name>/`.
  - SR-PACKAGES decision 10 records the ruling: MIT for the repo and every package.
  - `build-packs.py --release --verify` now builds all ten artefacts. Each `package.json` carries `"license": "MIT"`, and each wheel carries `License-Expression: MIT` and its `LICENSE`. Both installs gave byte-identical output.
  - The README licence section and badge, a CHANGELOG line and a CONTRIBUTING line are updated. The inbox row, `BLOCKED.json`, W-03 and W-04 are narrowed to the go to publish. Nothing committed.
- **Decided / left open:** the decider's licence ruling, MIT, taken as given. Still open: the go to publish (inbox). The crates carry no `license` field; B-038 adds it when it builds the engine packages.
- **Mistakes:** none.
- **Next:** B-038, the engine's packages; then the decider's go to publish.

---

## 2026-10-20 — W-05 filed: guidance while an agent writes  ·  Cowork (Opus 5.5)

- **Asked:** *codelaw should tell an agent how to write code while it writes, not only fix it after; research, propose in chat, file a work item; can Fux do it?*
- **Done:** W-05 (`work/open/W-05-write-time-guidance.md`), its `research` row, two glossary terms. Nothing committed.
- **Decided / left open:** proposed a construct-filtered `brief` and a `hook --pre` check (Claude Code `PreToolUse` can deny with a reason the agent reads); Fux as an optional free-text layer, not a dependency, since L5 needs codelaw complete alone and Fux SR-MCP refuses new tools. Rulings come at W-05 step 5.
- **Mistakes:** none.
- **Next:** W-05 step 1, the per-agent pre-write matrix.

---

## 2026-10-20 — W-03 step 6: the pack packaging  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement w-03"*.
- **Done:**
  - Triage: one 🔴 inbox row (W-04's `.github/` templates, 0 d); W-03 was the only 🟢 item.
  - `scripts/build-packs.py` (stdlib, Python 3.9): the five packs → `@codelaw/rules-<name>` tarballs and `codelaw-rules-<name>` `py3-none-any` wheels in `dist/packs/`, version from `Cargo.toml`. Byte-reproducible; byte parity with `packs/<name>/` checked per artefact; `--verify` installs both layouts over `fixtures/l3` (10 `recommended` presets, 20 diagnostics, identical JSON); `--release` refuses without a `LICENSE`.
  - Checked by hand with real `npm i` (Node 24) and `pip install` / `pip uninstall` (Python 3.14 venv); the engine resolves the packs from `node_modules/` and through `VIRTUAL_ENV`.
  - SR-PACKAGES decision 9 names the script and owns it (register row); B-038 and B-012 narrowed; `/dist/` ignored; CONTRIBUTING gains the build command.
- **Decided / left open:** filed to the decider: the project licence (one SPDX id) and the go to publish; `BLOCKED.json` set to `ASK`. The engine's packages (B-038: launcher, eight binaries, engine wheels, release job) are not built: they are not W-03's own scope and want their own `W-nn`.
- **Mistakes:** none. The uncommitted W-04 changes from the Cowork session were left as they were, and this session's changes were added on top of them in the shared files.
- **Next:** the decider rules on the licence; meanwhile B-038 can be promoted to a `W-nn` and built.

## 2026-10-20 — W-04: the consumer README and the public-repo docs  ·  Cowork (Opus 5.5)

- **Asked:** *"create the readme file and other necessary files. research and create one which is helpful for consumer and could be used for marketing as well"*.
- **Done:**
  - Researched dev-tool README structure (Ruff, Biome, a README guide). Built 0.1.0 in a Linux workspace and ran every README command on a sample mixed repo, so the output blocks are real.
  - `README.md`: pitch, plain-words section, real `check`/`fix` output, quickstart from source (packs linked into `node_modules/` until published), the five packs with counts and fix classes, a local rule with `codelaw test`, config, a Claude Code `PostToolUse` hook recipe, commands, who it is for, roadmap matching B-028–B-041, FAQ.
  - `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`. The new docs pass `md/recommended` and `data/recommended` (dogfooded).
  - Registry rows for the five docs (SR-WORK-REGISTRY §3); W-04 filed.
- **Decided / left open:** `.github/` is protected from remote writes, so the five issue and PR templates were sent to the decider to place (W-04, 🔴). The licence is still open (W-03 step 6); the README says it will be chosen before the first release. `codelaw`, `@codelaw/rules-js` and `codelaw-rules-js` returned 404 on npm and PyPI today, so the names look free.
- **Mistakes:** a probe file `.github/x` was written while testing whether `.github/` is writable; removed with the decider's delete permission.
- **Found while testing** (not fixed, engine untouched): `codelaw rules … | head` panics on a broken pipe; `init` writes a `codelaw.toml` comment pointing at `codelaw config --print`, which is not built (B-037); `fix` leaves a blank first line after removing a file's only import; with `preview` off, `check` says "no rule pack is configured" when packs are configured but all-preview.
- **Next:** the decider places the `.github/` templates; then W-03 step 6.

---

## 2026-10-20 — W-03 steps 2–5: all five rule packs merged  ·  Claude Code (Opus 5.5)

- **Asked:** *"continue all run everything in parallel"*.
- **Done:**
  - `packs/js` finished: `node`, `stencil` and `js/conventions` presets — 132 rules (b9cabc6).
  - Four pack agents in worktrees; `packs/data` (30 rules, f10648f) and `packs/css` (70 rules, edc1eaf) reviewed, re-tested and merged. The native registry test now checks unique ids, not sorted order, so parallel branches merge by union.
  - My own checks after each merge: workspace tests, clippy, every pack's `codelaw test` and `rules lint`; the data pack over this repo's records and docs; the css pack over 60 third-party stylesheets — every sampled finding matched upstream's behaviour.
  - SR-RULE-PACKS decision 7 named MDN browser-compat-data (CC0-1.0) as an allowed data source, against L2 (CC0 is not OSI-approved); corrected to name the MIT and Apache-2.0 sources actually used. W-03's css row said the same and is corrected.
  - Backlog B-040 (native rules see no file path) and B-041 (tree-sitter-css can't parse modern CSS).
- **Decided / left open:** `md/no-duplicate-heading` defaults to siblings only; `json/package-engines` defaults to `>=22` and `toml/pyproject-requires-python` to `>=3.11`, the L6 host floors — for the decider to correct. Still open: whether every pack rule stays `preview`, and the project licence (blocks publishing).
- **Mistakes:** the agents shared one scratchpad corpus folder and one emptied it under another mid-run; each now keeps its own. A whole-disk search for CSS files ran past its timeout and was stopped.
  - `packs/html` (71 rules, 6045676) merged after the same checks plus a 51-file rustdoc and Homebrew-docs run: findings sampled were spec violations html-validate also reports.
  - `packs/python` (98 rules) held back: compared with Ruff on pip's source it gave 87 extra unused-import findings and false `undefined-local` and `groupby` findings; returned to its agent with reproductions.
  - `packs/python` re-checked after the fixes and merged (f746d06): the agent traced the false positives to the semantic model reading `List[int]` as a PEP 695 type-parameter list, fixed it there with tests; on pip the only remaining extras are lines under one `# noqa`.
- **Next:** W-03 step 6 — the packaging (B-038), then the decider's ruling to publish and on the project licence.

---

## 2026-10-19 — W-03 in part: js, ts and react presets; Stencil and Node natives; subagents on python and css  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement open work items"*, then *"continue"*, then *"commit everything and push"*.
- **Done:**
  - `packs/js`: `js/recommended` (all 64 of ESLint 10.0.1), `ts/recommended` (typescript-eslint 8.71.2's 20 plus four core rules), `react/recommended` (17 of eslint-plugin-react 7.37.5, plus `rules-of-hooks` and `exhaustive-deps`) — 107 rules, `codelaw test packs/js` green.
  - Native rules for those, plus 16 Stencil rules and `node/no-deprecated-api` with a data table; their rule files are not written yet.
  - Engine: five predicates, data tables for natives, kinds missing from a grammar match nothing, duplicate YAML keys refused, empty `$$$` captures.
  - Corpus runs over npm's source, zod/ajv/TanStack sources and react-query: false-positive patterns found and fixed with tests; real defects found in `@npmcli/agent` and `node-gyp`.
- **Decided / left open:** every pack rule is `preview` (SR-RULE-AUTHORING decision 6), so presets need `preview = true` until a corpus job promotes them — for the decider to confirm. The project's own licence is undecided, so packs have no `LICENSE`; it blocks only publishing (W-03 step 6).
- **Mistakes:** I repeatedly wrote duplicate YAML keys in rule specs; the loader now refuses them. Both pack subagents stopped on a usage limit; the Python one is restarted, the CSS one resumed — both still running, in worktrees, uncommitted.
- **Next:** the Stencil and Node rule files and presets; then review and merge the python and css branches.

---

## 2026-10-19 — W-02: the engine — 13 crates, the `codelaw` binary, the law checks  ·  Claude Code (Opus 5.5)

- **Asked:** *"implement open work items"* — the queue's 🟢 pick was W-02 (W-03 waited on it).
- **Done:**
  - **The engine** in `crates/`: core, cache, config, lang, rules, semantic, match, native, fix, engine, report, test, cli. `cargo test --workspace` green; clippy clean.
  - **Languages:** all nine parse; TOML registered by codelaw; Markdown's inline grammar runs on `inline` nodes; HTML and Markdown regions map back to host positions.
  - **Rules:** the Markdown rule parser with every template and L4 check at load; `where:` predicates over the semantic model; eight fix primitives; the apply loop with the verifier; `native:` rules (`js/no-unreachable` first); pack resolution from `node_modules` and the Python environment.
  - **Checks:** layers (B-020), L2 licences over every Cargo dependency (B-008, Cargo part), the L3 fixture matrix and isolation diff (`fixtures/l3/`), L4 fix fields and loader, L5 determinism, CLI/library parity and the no-model fence, cold/warm/partial cache.
  - **Records:** SR-RULE-FORMAT 3a–3d and 5, SR-ARCHITECTURE 7, 15a, 18 and 28, SR-LINTER 3 and 6, SR-RULE-PACKS 3, 4, 6, 9 and 11, SR-CONFIG 2 and 3, SR-CACHE 5, SR-CLI-MCP 4; every crate owned in the register; `built` column defined.
  - **Work files:** W-02 closed and archived; W-03 🟢; backlog B-028–B-039 added, B-010 and B-021 closed, five rows narrowed; two lessons, two MACHINE rows, five glossary terms.
  - **`CLAUDE.md` changed:** the layout block names `crates/`, `fixtures/` and `Cargo.toml` (a fact; SR-WORK-DOCS decision 7).
- **Decided / left open:** this session's calls, for the decider to correct:
  - the `where:` syntax, with `$$MATCH` for the matched node and `not`/`all`/`any`;
  - **root-only config keys** (`extends`, `rules_dir`, `preview`, the `[files]`-style sections): a nested config may set only languages, include/exclude, `[rules]` and overrides;
  - never entered: `.git`, `node_modules`, `.venv`, `.codelaw/cache`; the global gitignore isn't read;
  - `init` in a repo with both manifests: `rules-python` from PyPI, the rest from npm;
  - preview gating applies to pack rules only; `npm:`/`pypi:`/`path:` packs enable all their rules;
  - SR-ARCHITECTURE decision 28 now lists `init`'s package manager, which SR-RULE-PACKS decision 6 already required;
  - **the law records SR-LAW-2 and SR-LAW-3 were touched in `owns:` frontmatter only**, to own their checks; no law text changed.
- **Mistakes:** several test expectations were wrong on first run (line numbers, module-level Python names, a vacuous assertion) and were corrected against the code's behaviour, which was right. Two real bugs were caught by tests: YAML 1.1 booleans, and a missing `fix:` cascading into three misleading errors.
- **Next:** W-03 — the five rule packs, starting with `rules-js`.

---

## 2026-10-19 — W-01: the records rewritten for five separately published rule packs; SR-RULE-PACKS accepted  ·  Cowork (Opus 5.5)

- **Asked:** *"go with the recommendations for all of them"*: run W-01, then accept SR-RULE-PACKS.
- **Done:**
  - **SR-RULE-PACKS** rewritten: five first-party packs (`js`, `html`, `css`, `python`, `data`) on npm `@codelaw/rules-*` and PyPI `codelaw-rules-*`, with layout, resolution, `init` install, ported-rule policy and data tables. The embedded-in-binary choice is moved to Alternatives. Accepted.
  - **SR-LAW-6:** law text and decisions 3, 4, 9 and 10 amended; decision 11 added.
  - **SR-PACKAGES:** 20 crates (`codelaw-rules-builtin` out; `codelaw-semantic` and `codelaw-native` in); decision 9 covers the packs.
  - **SR-ARCHITECTURE:** decisions 15a–15c (semantic model, native rules, pack data).
  - **Smaller amendments:** SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT (`ported_from:`, `native:`), SR-LINTER, SR-AGENT-SURFACES, SR-BASELINE, SR-CLI-MCP (`init --no-install --registry`), L3 decision 2, L4.
  - **Routing:** `CLAUDE.md` L6 line, SR-LAWS, the register, DESIGN.md and the glossary.
  - **Queue:** W-01 closed and archived; W-02 🟢.
- **Decided / left open:** this session's calls, for the decider to correct:
  - the `rules-data` presets are namespaced `data/…`;
  - an opt-in `conventions` preset per pack;
  - `native:` only in first-party packs;
  - the npm copy of a pack is found before the Python copy;
  - SR-ARCHITECTURE's new decisions are lettered (15a–15c) so that nothing is renumbered.
- **Mistakes:** SR-RULE-PACKS's decisions were renumbered by the rewrite; four citations in other records were found by grep and repointed in the same change.
- **Next:** W-02 (the engine).

---

## 2026-10-19 — the decider rules on the whole inbox: L5 and fourteen records accepted; L6 floors 3.11 / 22  ·  Cowork (Opus 5.5)

- **Asked:** *"go with the recommendations for all of them."* (the blocker list with a recommendation per row)
- **Done:** accepted SR-LAW-5 (banner replaced by the ruling line) and SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE, SR-FORMATTER, SR-LINTER, SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP (frontmatter + register). SR-LAW-6 decision 8: host floors Python ≥ 3.11, Node ≥ 22. `CLAUDE.md` L5 line and `docs/DESIGN.md` law table no longer say *(proposed)*. Inbox emptied; W-02 and W-03 🔴 → 🟡; W-01 gains DoD 10 (accept SR-RULE-PACKS after the rewrite). Committed with everything filed since `9baa117`.
- **Decided / left open:** SR-RULE-PACKS stays proposed until W-01 rewrites it for separately published packs (the ruling: accept after the rewrite). Installing `scripts/sr-guard.sh` as the `commit-msg` hook stays Arpit's step.
- **Mistakes:** none.
- **Next:** W-01.

---

## 2026-10-19 — preset research; W-01–W-03 filed to split rules into five packs  ·  Cowork (Opus 5.5)

- **Asked:** *"research the rules preset in ruff, eslint, tslint, prettier, and other and create as separate packages, like 1 for js/ts/node/jsx/tsx, 1 for html, 1 for css, 1 for python and engine will be separate create work item"*
- **Done:** researched ESLint (`@eslint/js` recommended, 64 rules), typescript-eslint configs, TSLint (deprecated 2019), React hooks, Node, Stencil, oxlint and Biome; Ruff (old default 59 rules, v0.16.0 default 413); stylelint (recommended 41, standard); html-validate presets, HTMLHint, markuplint; markdownlint, SchemaStore, yamllint, taplo; Prettier option defaults. Filed **W-01** (record update, 🟢), **W-02** (engine, 🔴), **W-03** (the five packs + the research, 🔴); the nine-record and SR-LINTER inbox rows now block W-02 and W-03. Glossary: semantic model, native rule. Nothing committed.
- **Decided / left open:** the decider ruled: **separately published**; mirror every pack on both registries (`@codelaw/rules-*` / `codelaw-rules-*`); a fifth pack `rules-data` for JSON/YAML/TOML/Markdown; `codelaw init` installs detected packs and the engine ships no rules. This session's calls inside the items: one version for engine and packs; reimplement, never vendor; drop layout rules (L7) and type-aware rules (L3); Pylint and yamllint contribute ideas only (GPL). Found a gap no record covered: porting real presets needs a per-file semantic model and `native:` rules — now W-02 DoD 3–4.
- **Mistakes:** none.
- **Next:** W-01.

---

## 2026-10-19 — SR-ARCHITECTURE rewritten as the design of record; cross-cutting gaps closed  ·  Cowork (Fable 5.1)

- **Asked:** *"design the architecture fill/resolve the gaps if any."*
- **Done:** audited the 21 records against each other and rewrote `records/0108_architecture.md` (proposed) to 30 decisions in eight groups. Gaps no record decided, now decided there: file encoding and BOM (8), line endings (9), generated and oversized files (10), glob base (11), safe writes and the LSP never writing (12), position encodings incl. UTF-16 at the LSP boundary (13), git via the `git` binary only (14), lazy embedded regions and the `languages` switch (16), `root = true` as a separate project (17), the external-fixer stdin/stdout contract (18), the adapter manifest and "never in hook/LSP" (19), warm-socket discovery with version equality (21), daemon lifecycle (22), tree-cache memory bound (24), rule-panic isolation (25), `skipped[]` honesty (26), no telemetry (27), what may execute (28), fix-conflict detection (5). Small pointer patches in SR-CONFIG, SR-LINTER, SR-RULE-FORMAT, SR-BASELINE, SR-CLI-MCP, SR-EDITOR, SR-CACHE; `docs/DESIGN.md` §3 now defers to the record. B-026, B-027; two glossary terms. Nothing committed.
- **Decided / left open:** this session's calls, for the decider to correct: shell out to `git` rather than link a git library; adapters excluded from hook and LSP mode; symlinks not followed; 1 MiB file-size default; daemon stays opt-in. Still undecided by design: a JS rule API, type-aware rules, SCSS/Vue/Svelte. SR-ARCHITECTURE stays in the existing nine-record inbox row.
- **Mistakes:** the rewrite renumbered decisions, which broke three cross-references in other records; found by grep and fixed in the same change.
- **Next:** the decider rules on the inbox.

---

## 2026-10-18 — laws L7, L8 and nine records filed; L3, L4, L6 amended  ·  Cowork (Opus)

- **Asked:** *SRs for packages, presets + local rules, VS Code extension + LSP, MCP + CLI, rule authoring (manual or agent skill; debuggable, testable), the rule template, caching, architecture, the config file, the consumer directory; laws for the eleven file types, linter + formatter, fixer-or-skill, one `codelaw` package per registry with `@codelaw/*` sub-packages, and performance budgets that are never broken.*
- **Done:** SR-LAW-7 (linter + formatter), SR-LAW-8 (performance budgets, values unset); SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP (proposed). Amended in place by the decider's ruling: SR-LAW-3 (JSX and TSX named explicitly), SR-LAW-4 (a skill carries every manual fix; `fix_skill:`), SR-LAW-6 (naming: only `codelaw` unscoped on npm, `@codelaw/*` for the rest; only `codelaw` on PyPI). SR-RULE-PACKS retitled to cover local rules (no new record: it already owned presets, packs and local rules). Routed in SR-LAWS, the register and `CLAUDE.md` (L4/L6 lines changed, L7/L8 lines added). B-017–B-025; glossary. Nothing committed.
- **Decided / left open:** the language law, the fixer law and the packages law already existed (L3, L4, L6), so they were amended rather than duplicated. L8 fixes the mechanism now and leaves every number `unset` until measured (B-019). The nine records are one inbox row.
- **Mistakes:** none.
- **Next:** the decider rules on the inbox.

---

## 2026-10-18 — SR-LINTER filed; L3 amended to nine languages  ·  Cowork (Opus)

- **Asked:** *"create an SR for linter as well. The linter and formatter are going to work for JavaScript, TypeScript, JSON, Python, CSS, HTML."*
- **Done:** SR-LINTER (`records/0106_linter.md`, proposed). SR-LAW-3 amended to nine languages (CSS, HTML added). CSS/HTML carried into SR-RULE-PACKS (presets + 8 starter rules), SR-FORMATTER (Biome CSS, `markup_fmt` for HTML), SR-PROJECT-GRAPH (`graph/no-unused-css-classes`, HTML link edges), SR-AGENT-SURFACES (skill trigger), `docs/DESIGN.md`, SR-LAWS, the register and `CLAUDE.md` (L3 pointer line changed). B-016; glossary "embedded region". Nothing committed.
- **Decided / left open:** the ask named six languages without YAML/TOML/Markdown; the decider ruled **add CSS and HTML, keep all seven** (nine). SCSS/Sass/Less and Vue/Svelte/Angular templates stay out of scope. Ruling on SR-LINTER filed in the inbox.
- **Mistakes:** none.
- **Next:** the decider rules on the eight inbox rows.

---

## 2026-10-18 — SR-FORMATTER filed: codelaw is a linter and a formatter  ·  Cowork (Opus)

- **Asked:** *"Create an SR that linter code law will be a linter as well as a prettier."*
- **Done:** SR-FORMATTER (`records/0105_formatter.md`, proposed); register row, inbox row, B-015, glossary term "layout rule", `docs/DESIGN.md` link and config sample. Nothing committed.
- **Decided / left open:** engines embedded, not shelled out (L3/L6): Biome for JS/TS/JSON (Prettier-compatible), Ruff for Python (Black-compatible), pretty_yaml, taplo, dprint-plugin-markdown — all permissive (L2). Biome and Ruff are git-pinned because their crates.io copies are stale or internal. Layout-only lint rules are refused. Ruling filed in the inbox.
- **Mistakes:** none.
- **Next:** the decider rules on the seven inbox rows.

---

## 2026-10-18 — laws L3–L6 and four design records filed  ·  Cowork (Opus)

- **Asked:** *create laws (languages; every rule has a fix; Python and Node packages), decide how predefined rules ship, an SR for the skills (the whole agent-facing set), then the mechanisms for unused exports across a monorepo and deprecations with a grandfathering file.*
- **Done:** SR-LAW-3 (languages), SR-LAW-4 (fix), SR-LAW-5 (no model in the engine, proposed), SR-LAW-6 (PyPI + npm); SR-RULE-PACKS, SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE (all proposed); routed in SR-LAWS, the register and `CLAUDE.md` §The laws (four pointer lines added — `CLAUDE.md` changed); `docs/DESIGN.md` links the laws; B-009–B-014; glossary terms. Nothing committed.
- **Decided / left open:** predefined rules embedded in the binary as presets (the decider left the choice open); yamllint dropped as an adapter (GPL-3.0, refused by L2); host floors Python ≥ 3.9 / Node ≥ 18 are this session's reading. Six rulings filed in the inbox.
- **Mistakes:** this session first wrote its laws as `0001`–`0004` / L1–L4 without seeing the work system set up minutes earlier, and overwrote the uncommitted-in-its-turn `records/README.md`. Caught by listing the folder before the next write; the register was restored from `HEAD`, the laws renumbered to `0005`–`0008` / L3–L6, and the four misnumbered files removed.
- **Next:** the decider rules on the six inbox rows.

---

## 2026-10-18 — law L2 filed  ·  Claude Code (Opus 5.5)

- **Asked:** *"add a new law to not use paid and open source dependencies."*
- **Done:** SR-LAW-2 (`records/0004_LAW-2-foss-dependencies.md`); routed in SR-LAWS, `CLAUDE.md` §The laws and the register; backlog row B-008 for the licence check nobody runs yet. `CLAUDE.md` changed: one pointer line for L2.
- **Decided / left open:** the ask read two ways, so it went to the decider, who ruled: **FOSS only, nothing paid**, and **no copyleft anywhere**, dev tooling included. Weak copyleft (MPL, EPL) and paid hosted services are refused too — that reading is this session's, stated in SR-LAW-2 decisions 2 and 5 for the decider to correct.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; B-008 becomes a `W-nn` when the first manifest lands.

---

## 2026-10-18 — the work system set up from WORK-SYSTEM.md  ·  Claude Code

- **Asked:** *set up the work system from WORK-SYSTEM.md.*
- **Done:** records 0001–0003 and 0051–0060, `work/`, `archive/`, `scripts/`, `.claude/` hooks, `CLAUDE.md`; WORK-SYSTEM.md moved to `archive/bootstrap/`.
- **Decided / left open:** tests deferred — filed as backlog rows B-001–B-005.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; the first `W-nn` is filed.
