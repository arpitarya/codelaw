---
type: Proposal
title: codelaw beside ESLint + Prettier, Biome, Ruff and oxc — what to claim, what to build, who it is for
description: "W-36's research outcome: a sourced map of the incumbents at the four moments, codelaw's adoption gaps as candidate items, the claims it can and cannot make, and the README changes that follow."
status: graduated
timestamp: 2026-10-27T08:45:00+05:30
---

# codelaw beside ESLint + Prettier, Biome, Ruff and oxc — W-36's proposal

**For the decider to pick** (§7). Researched 2026-10-27 (Cowork, Opus 5.5) from each tool's own docs, changelogs and source (§8); codelaw's side from this repo the same day. A competitor cell goes stale fast: every one carries its source and date.

## 1 · In one line

**No incumbent checks an edit before it is written, none lints JS and Python and CSS and HTML and TOML from one engine, and none ships a rule catalogue an agent can query — those three are codelaw's to own. But it trails on day-one adoption (no migrate command, no published editor extension, no GitHub Action), it cannot yet claim speed, and the fastest-moving rival for agents is oxc, not ESLint.**

## 2 · The four moments, incumbents vs codelaw (step 1)

| moment | ESLint 10.13 (+ts-eslint 8.72) | Prettier 3.9 | Biome 2.6.3 | Ruff 0.17.3 | oxlint 1.89 / oxfmt 0.74 | **codelaw 0.2.0** |
|---|---|---|---|---|---|---|
| **Before** — an agent asks what applies | `@eslint/mcp` 0.3.15: **one tool, `lint-files`**, files on disk; no list/explain [1][2] | none [17] | none; `concise` reporter "saves tokens" [33] | none; Claude Code plugin with skills [46] | none; `llms.txt`, a "Coding agents" page [54][55] | ✅ MCP `rules_for_path`, `check_snippet`, `explain_rule`, `format_snippet`; per-rule pages; skills for 5 agents |
| **While** — refuse an edit before disk | ✗ | ✗ | ✗ | ✗ | ✗ (its documented hook is PostToolUse) [54] | 🧪 `hook --pre` — W-05 proposal (every major agent now has a pre-write hook) |
| **After** — fix by default? | needs `--fix` [5] | needs `--write` [19] | needs `--write` [26] | needs `--fix` [37] | oxlint `--fix`; oxfmt writes by default [50][60] | `codelaw hook` fixes and reports; `fix` vs `fix --unsafe` |
| **After** — agent output | — | — | `concise` [33] | — | `--format=agent`, **auto-selected when an agent is detected** (1.66, 2026-05-18) [51][53] | `--format agent` — chosen by flag only |
| **Without AI** | the reference | the reference | one tool for lint+format | the Python reference | fastest JS/TS | CLI, LSP, git hooks, CI formats, SARIF |

**Reading it:** the *before* and *while* columns are empty for every incumbent; ESLint's MCP is the only official one and is a lint-on-disk call. oxc is moving fastest toward agents (agent detection, agent format, a coding-agents page) — the rival to watch.

## 3 · Day-one adoption — what a team expects (step 2)

| table stake | ESLint | Biome | Ruff | oxc | **codelaw** | gap → item |
|---|---|---|---|---|---|---|
| one-line install | npm | npm | pip | npm | ✅ npm and PyPI, same engine | — |
| `init` | ✅ [8] | ✅ [26] | — | ✅ [50][62] | ✅ `codelaw init` (detects languages, packs, agents, git hooks) | — |
| **migrate a config** | eslintrc→flat [9] | ✅ `migrate eslint/prettier/stylelint` [26] | "drop-in parity" [36] | ✅ `@oxlint/migrate`, `oxfmt --migrate prettier/biome` [49][61] | ✗ a `codelaw-migrate` **skill** only (needs an agent) | **G1** |
| first-party editor extension | ✗ (third-party) [10] | VS Code, Zed, IntelliJ [29] | VS Code, Zed [41] | VS Code, Zed, JetBrains [56] | built, **not published** | **G2** |
| pre-commit.com hook | ✗ [10] | ✅ [30] | ✅ [38] | ✅ mirror [57] | `init --git-hooks`, husky/lefthook recipes; the pre-commit mirror is W-64 | W-64 |
| GitHub Action | ✗ | ✅ `setup-biome` [31] | ✅ `ruff-action` [38] | ✗ (auto-annotations) [57] | `--format github`; `init --ci github` writes a workflow; no Action | **G3** |
| SARIF | ✗ [10] | ✅ 2.4 [27] | in source [39] | ✅ 1.63 [51] | ✅ | — |
| baseline / bulk suppressions | ✅ `eslint-suppressions.json` [7] | inline only [28] | inline only [37] | hidden flags [52] | ✅ baseline + exceptions with **owner, expiry and on-touch revocation** | — (an edge) |
| inline ignore | ✅ | ✅ reason required | ✅ | ✅ (+ `eslint-disable`) [58] | ✅ `codelaw-ignore … expires= by=: reason` | consider accepting `eslint-disable`/`noqa` → **G1** |
| monorepo config | ✅ [11] | ✅ `extends: "//"` [32] | ✅ [40] | ✅ [51] | ✅ nested `codelaw.toml` | — |
| type-aware TS rules | typescript-eslint | ~75 % of one rule set [32] | ty (beta) [63] | **stable, 59/61 rules via tsgo** [59] | ✗ — parked proposal `type-aware-rules` | **G6** |
| `llms.txt` | not found | ✅ [18] | not found | ✅ [45] | ✗ (per-rule pages exist, `docs/rules/`) | **G7** |

## 4 · Claims — what survives a reader checking (step 3)

| claim | verdict | why |
|---|---|---|
| "the only one that checks an edit before it is written" | **hold until W-05 ships**, then make it the lead | true of all five incumbents today [2][54]; W-35's rule: say what ships |
| "one engine, one config: JS, TS, Python, JSON, YAML, TOML, Markdown, CSS/SCSS, HTML" | **make it** | no incumbent spans JS and Python; nothing else lints TOML [25][36][49] |
| "every rule carries a fix" (L4) | **make it, precisely** | incumbents leave suggestion/unsafe tiers unapplied [6][26][37][50]; codelaw's `manual` fixes are written steps, not applied — say "a fix, auto or written", not "auto-fixes everything" |
| "a rule catalogue agents can query" | **make it** | ESLint's MCP has no list/explain [2] |
| "exceptions with an owner and an expiry" | **make it** | ESLint's suppressions file has neither [7] |
| "faster than ESLint / Biome / oxlint" | **drop until measured** | `codelaw bench` measures codelaw alone; a side-by-side needs the rivals installed in CI (L11) on the three pinned repos — **G5** |
| README: "**441** rules reimplemented" (line 97) and "**652** rules reimplemented" (line 205) | **both wrong** | 652 rules in all; **492** carry `ported_from:`, 160 are codelaw's own (counted 2026-10-27) |
| README hero (line 36): "The agent gets the rules before it writes, **is checked while it writes**, and is fixed after" | **overclaims** | the four-moments table below it says *While* is 🧪; the hero must too (W-35) |
| "with Fux and good docs an agent never breaks a rule" (step 3a) | **unproven either way** — run §5 | the README's "rules it can't skip" line stands until it is |

## 5 · Step 3a — the experiment, ready to run

- **Tasks:** 20 small tasks in `fixtures/` (add an endpoint, a React component, a CLI flag, a data migration…) in a repo with `js/recommended`, `py/recommended`, three custom rules and two configured `use-instead` rules.
- **Arms:** (A) agent with only AGENTS.md; (B) + codelaw MCP and the rule pages; (C) + Fux index and the rule pages. Same model, same prompt, 3 runs each — 180 runs.
- **Measure:** `codelaw check` on the diff (new findings only, by rule), with `hook` and `hook --pre` **off**; record tokens and turns.
- **Decision rule:** if arm C gives zero new `error`s in all 60 runs, the README's "you need a check" line goes; otherwise it stays, with the measured rate ("in N of 60 runs an agent with the best guidance still broke a rule").
- **Who runs it:** a Claude Code session on the Mac (it needs a real agent); filed as its own item when picked.

## 6 · Who picks it, and their objections (step 4)

| who | why codelaw | their objection | the answer, or the gap |
|---|---|---|---|
| **a team where agents write much of the code** | rules before and during the write; a deterministic verdict; one catalogue for 5 agents | "my agent already runs ESLint after editing" | the after-check costs a turn per mistake; a pre-write refusal costs none — **once W-05 ships** |
| **a full-stack team, JS + Python** | one binary, one config, one CI step, one baseline, both registries | "Ruff and Biome are each best-in-class" | parity is the bar: ported rule sets, Prettier-style output — and **G6** for TS |
| **a platform team** | owners and expiries on exceptions, SARIF, a rule as a Markdown file with no plugin | "every rule is `preview`" | promote rules by corpus runs (the FAQ's own answer) — **G8** |
| **anyone switching** | — | "how long will moving take?" | **G1** (`codelaw migrate`), **G2** (an extension they can install) |

## 7 · Rulings asked of the decider

| # | question | recommendation |
|---|---|---|
| R1 | README now: fix the hero's *while* overclaim and the 441/652 counts; add the "one engine for both stacks" and "exceptions with owner and expiry" lines; **no speed claim** | **Yes** — a docs change, W-35's rule |
| R2 | File the gaps as items, in this order: **G1** `codelaw migrate eslint\|prettier\|biome\|ruff\|stylelint` (deterministic config translation; accept `eslint-disable`/`# noqa` as aliases) · **G2** publish the VS Code extension (Marketplace + Open VSX) · **G4** auto-select `--format agent` when an agent is detected (CLAUDECODE, CODEX_*, CURSOR_AGENT, GEMINI_CLI) · **G3** a `codelaw-action` for GitHub · **G7** `llms.txt` from `docs/rules/` · **G5** a side-by-side bench (ratify the rivals as CI-only bench tools under L11) · **G8** rule promotion from `preview` by corpus | **Yes, all eight; G1, G2, G4 first** |
| R3 | Reopen the parked `type-aware-rules` proposal (**G6**) — oxlint made type-aware TS stable on 2026-07-22 | **Yes, as a research item** |
| R4 | Run step 3a (§5) as its own item | **Yes** — it decides a README line with data |

## 8 · Sources (each as read 2026-10-27)

[1] github.com/eslint/rewrite/blob/main/packages/mcp/CHANGELOG.md — @eslint/mcp 0.1.0 (2025-05-13) … 0.3.15 (2026-10-16) · [2] …/packages/mcp/src/mcp-server.js — one tool, `lint-files` · [5] eslint.org/docs/latest/use/command-line-interface · [6] eslint.org/docs/latest/extend/custom-rules · [7] eslint.org/docs/head/use/suppressions (bulk suppressions, 2025-04-04) · [8] eslint.org/docs/latest/use/getting-started · [9] eslint.org/docs/latest/use/configure/migration-guide · [10] eslint.org/docs/latest/use/integrations · [11] eslint.org/blog/2026/02/eslint-v10.0.0-released/ · [17] prettier.io/blog · [18] prettier.io/llms.txt · [19] prettier.io/docs/cli · [25] biomejs.dev · [26] biomejs.dev/reference/cli/ · [27] Biome CHANGELOG (SARIF 2.4.0; 2.6.3) · [28] biomejs.dev/analyzer/suppressions/ · [29] biomejs.dev/guides/editors/first-party-extensions/ · [30] biomejs.dev/recipes/git-hooks/ · [31] biomejs.dev/recipes/continuous-integration/ · [32] biomejs.dev/blog/biome-v2/ · [33] biomejs.dev/blog/biome-v2-5/ · [36] docs.astral.sh/ruff/ · [37] docs.astral.sh/ruff/linter/ · [38] docs.astral.sh/ruff/integrations/ · [39] ruff `crates/ruff_linter/src/settings/types.rs` (`OutputFormat::Sarif`) · [40] docs.astral.sh/ruff/settings/ · [41] docs.astral.sh/ruff/editors/setup/ · [45] docs.astral.sh/ruff/llms.txt · [46] github.com/astral-sh/claude-code-plugins · [49] oxc.rs/docs/guide/usage/linter.html · [50] oxc.rs/docs/guide/usage/linter/cli.html · [51] oxlint CHANGELOG (1.63.0 2026-05-05 agent format + SARIF; 1.66.0 2026-05-18 agent detection; 1.89.0 2026-10-19) · [52] oxc `apps/oxlint/src/command/lint.rs` (hidden suppress flags) · [53] oxc `apps/oxlint/src/agent_detection.rs` · [54] oxc.rs/docs/guide/usage/coding-agents.html · [55] oxc.rs/llms.txt · [56] oxc.rs/docs/guide/usage/linter/editors.html · [57] oxc.rs/docs/guide/usage/linter/ci.html · [58] oxc.rs/docs/guide/usage/linter/ignore-comments · [59] oxc.rs/blog/2026-07-22-type-aware-linting-stable · [60] oxc.rs/docs/guide/usage/formatter.html · [61] oxc.rs/blog/2026-02-24-oxfmt-beta · [62] oxfmt CHANGELOG (0.74.0) · [63] github.com/astral-sh/ty.

**Plain words:** the big tools all check code *after* it is written, and each covers only part of a JS-plus-Python codebase. codelaw can win on "before and during" and on "one tool for everything" — but to be picked up it still needs the easy stuff the others have (a migrate command, an editor extension you can install, a CI action), and it must not claim speed until it has measured it.
