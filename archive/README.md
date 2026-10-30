# `archive/` — the one archive, and where each thing went

**There is exactly ONE archive directory, and it is this one, at the repo
root.** Anything archived is moved here, into a directory mirroring where it
came from. This README is the **map**: every archived doc gets a row naming its
**live successor**.

## Archive is not evidence

A doc in here may be *named*. It may never be *cited as backing a live claim*
— nothing guarantees an archived file was not overwritten after it was
retired. Entries are frozen; **relative links inside them are not repaired**.
Rules: [SR-WORK-ARCHIVE](../records/0057_WORK-archive.md).

## Layout — the archive mirrors the live tree

| directory | holds |
|---|---|
| `open/` | closed work items' detail files, byte for byte |
| `records/` | retired records (L1) |
| `compare/` | decided forks whose triggers were ported into a record |
| `proposals/` | implemented or rejected proposals |
| `worklog/` | worklog slices cut at each major release |
| `bootstrap/` | the WORK-SYSTEM.md this repo's work system was set up from |
| `DEPENDENCIES.md` | a retired root file (W-90); see *Root files* |

## `open/`

| file | closed | where the live answer lives now |
|---|---|---|
| [`W-137-editor-findings-look.md`](open/W-137-editor-findings-look.md) | 2026-10-30 | done — [SR-EDITOR](../records/0114_editor.md) decision 8 (one `data` shape, the hover card, inlay hints, the manifest CodeLens and `codelaw.applyFixes`; the extension's three views, badges, chips, sidebar, status bar and tab colours); the design of record is [`docs/design/editor/`](../docs/design/editor/README.md) |
| [`W-140-port-rule-on-demand.md`](open/W-140-port-rule-on-demand.md) | 2026-10-30 | done — the skill `codelaw-port-rule` and the custom agent `codelaw-rule-porter` in the skills package, `ported_from` on the `rules` JSON card, the opt-in share issue (`rule_request.yml`), `scripts/ported-rule-demand.py`, three ports in `fixtures/port-rule/`: [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) decision 11, [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 2, [SR-RULE-FORMAT](../records/0112_rule-format.md) decision 2, [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 3 |
| [`W-130-feature-flag-expiry.md`](open/W-130-feature-flag-expiry.md) | 2026-10-30 | done — [SR-PROJECT-GRAPH](../records/0103_project-graph.md) decision 11 (accessors, six SDK presets, wrappers through the graph, `.codelaw/flags.toml` and `codelaw-flag` markers, the as-of date), [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 17 (`js`/`py` `flag-expiry` and `flag-marker`, `data/flag-registry`), `[flags]` (SR-CONFIG), `codelaw flags accessors` (SR-CLI-MCP) |
| [`W-133-rules-from-review-comments.md`](open/W-133-rules-from-review-comments.md) | 2026-10-30 | done — the skill `codelaw-rules-from-reviews` in the skills package, `[reviews]` in `codelaw.toml` (validated, read by no engine code): [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) decision 10, [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 2, [SR-CONFIG](../records/0110_config.md) |
| [`W-131-todo-and-skipped-test-markers.md`](open/W-131-todo-and-skipped-test-markers.md) | 2026-10-30 | done — `[todo]` and `[tests.skip]` (SR-CONFIG decision 11a); `*/todo-marker` in eight languages, `js/no-focused-tests`, `js/skipped-test`, `py/skipped-test`; opt-in `<ns>/markers` presets; dates on the as-of date |
| [`W-136-l3-findings-on-plain-text-config.md`](open/W-136-l3-findings-on-plain-text-config.md) | 2026-10-30 | done — [SR-LAW-3](../records/0005_LAW-3-languages.md) decision 10 (seven plain-text config files carry findings, never lint or format), [SR-ARCHITECTURE](../records/0108_architecture.md) decision 15d (`NativeRule::elsewhere`, the copy), [SR-EDITOR](../records/0114_editor.md) decision 2a; W-128, W-129 and W-132 report their second place through it |
| [`W-127-release-0-4-0.md`](open/W-127-release-0-4-0.md) | 2026-10-30 | done — v0.4.0 released from d2a87c8: npm, PyPI and the Visual Studio Marketplace (all eight platforms); Open VSX waits on W-117 |
| [`W-125-revert-pypi-new.md`](open/W-125-revert-pypi-new.md) | 2026-10-29 | done — `only=pypi-new` out of `publish.yml`, `main` off the `pypi` environment, L6 decision 9, SR-PACKAGES, RELEASING.md back to *a release uploads only existing projects* |
| [`W-124-session-cleanup.md`](open/W-124-session-cleanup.md) | 2026-10-29 | done — dropped on the decider's ruling (*"execute the cleanup of all the working trees"*); the one unlanded ruling on `lm/W-119-loop` carried to [W-125](open/W-125-revert-pypi-new.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-123-session-cleanup.md`](open/W-123-session-cleanup.md) | 2026-10-29 | done — dropped on the decider's ruling (*"execute the cleanup of all the working trees"*); the one unlanded ruling on `lm/W-119-loop` carried to [W-125](open/W-125-revert-pypi-new.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-122-session-cleanup.md`](open/W-122-session-cleanup.md) | 2026-10-29 | done — dropped on the decider's ruling (*"execute the cleanup of all the working trees"*); the one unlanded ruling on `lm/W-119-loop` carried to [W-125](open/W-125-revert-pypi-new.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-121-session-cleanup.md`](open/W-121-session-cleanup.md) | 2026-10-29 | done — dropped on the decider's ruling (*"execute the cleanup of all the working trees"*); the one unlanded ruling on `lm/W-119-loop` carried to [W-125](open/W-125-revert-pypi-new.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-120-session-cleanup.md`](open/W-120-session-cleanup.md) | 2026-10-29 | done — dropped on the decider's ruling (*"execute the cleanup of all the working trees"*); the one unlanded ruling on `lm/W-119-loop` carried to [W-125](open/W-125-revert-pypi-new.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-119-pypi-through-pending-publishers.md`](open/W-119-pypi-through-pending-publishers.md) | 2026-10-29 | done — every codelaw name on PyPI (24) and npm (45), trusted; `publish.yml`'s `only=pypi-new` was ruled temporary — its revert is [W-125](open/W-125-revert-pypi-new.md) |
| [`W-115-spell-pack-first-publish.md`](open/W-115-spell-pack-first-publish.md) | 2026-10-29 | merged into [W-119](../work/open/W-119-pypi-through-pending-publishers.md) — PyPI projects created by CI through pending publishers |
| [`W-89-pypi-day-4.md`](open/W-89-pypi-day-4.md) | 2026-10-29 | merged into [W-119](../work/open/W-119-pypi-through-pending-publishers.md) — PyPI projects created by CI through pending publishers |
| [`W-88-pypi-day-3.md`](open/W-88-pypi-day-3.md) | 2026-10-29 | merged into [W-119](../work/open/W-119-pypi-through-pending-publishers.md) — PyPI projects created by CI through pending publishers |
| [`W-116-release-0-3-0.md`](open/W-116-release-0-3-0.md) | 2026-10-29 | done — v0.3.0 on npm and PyPI; the extension's listing → [W-95](../work/open/W-95-publish-vscode-extension.md) |
| [`W-87-pypi-day-2.md`](open/W-87-pypi-day-2.md) | 2026-10-29 | done — `codelaw` and three Fux index projects on PyPI, trusted |
| [`W-86-pypi-day-1.md`](open/W-86-pypi-day-1.md) | 2026-10-29 | done — six packs and `codelaw` on PyPI, trusted; the user check is [W-38](../work/open/W-38-pypi-0-2-0-by-hand.md) |
| [`W-38-pypi-0-2-0-by-hand.md`](open/W-38-pypi-0-2-0-by-hand.md) | 2026-10-29 | done — PyPI `codelaw` and six packs live, trusted, checked as a user; the eight-platform registry smoke stays a CI job |
| [`W-77-skills-package.md`](open/W-77-skills-package.md) | 2026-10-29 | built (SR-PACKAGES, SR-AGENT-SURFACES); npm live; PyPI `codelaw-skills` → [W-88](../work/open/W-88-pypi-day-3.md) |
| [`W-34-fux-index-packages.md`](open/W-34-fux-index-packages.md) | 2026-10-29 | built (L6 decision 12, SR-PACKAGES decision 14); npm live; PyPI uploads → [W-87](../work/open/W-87-pypi-day-2.md), [W-88](../work/open/W-88-pypi-day-3.md), [W-89](../work/open/W-89-pypi-day-4.md); `publish.yml` for them → B-063 |
| [`W-114-session-cleanup.md`](open/W-114-session-cleanup.md) | 2026-10-29 | done — the five worktrees of session 2df06f6d dropped on the decider's ruling; [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) |
| [`W-111-two-unmerged-worktrees.md`](open/W-111-two-unmerged-worktrees.md) | 2026-10-29 | merged into [W-114](../work/open/W-114-session-cleanup.md) — the keep-or-drop ruling for all five worktrees of session 2df06f6d |
| [`W-110-session-2df06f6d-cleanup.md`](open/W-110-session-2df06f6d-cleanup.md) | 2026-10-28 | [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md); `.claude/hooks/worktree-cleanup.py` |
| [`W-109-bitbucket-pipelines.md`](open/W-109-bitbucket-pipelines.md) | 2026-10-28 | `--format bitbucket` in [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 4 and 12b; no pipe, no generated step in [SR-PACKAGES](../records/0107_packages.md) decision 15a; the step in USAGE §Bitbucket Pipelines; the live proof BACKLOG B-061 |
| [`W-62-scss-support.md`](open/W-62-scss-support.md) | 2026-10-28 | L3 and L7 (SCSS as a CSS dialect); [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 3 and 15b (both grammar forks; the CSS rules on the trees); `crates/codelaw-native/src/css_tree.rs`; [`packs/scss/`](../packs/scss/README.md); BACKLOG B-056–B-060 |
| [`W-113-project-icon.md`](open/W-113-project-icon.md) | 2026-10-28 | the icon files and their README in `docs/brand/` |
| [`W-112-spell-words-file-and-learn.md`](open/W-112-spell-words-file-and-learn.md) | 2026-10-28 | [SR-SPELL](../records/0116_spell.md) decisions 1 and 7; [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 1 (`init --no-spell`, `spell learn`); [SR-CONFIG](../records/0110_config.md) (`[spell] words_file`) |
| [`W-108-oxlint-live-capture.md`](open/W-108-oxlint-live-capture.md) | 2026-10-28 | the capture in `crates/codelaw-adapters/tests/fixtures/oxlint-type-aware/stdout.json`; which label is the span in [SR-LINTER](../records/0106_linter.md) decision 2 |
| [`W-104-type-aware-records.md`](open/W-104-type-aware-records.md) | 2026-10-28 | [SR-TYPES](../records/0117_types.md); L3 decision 9, L5, L6 decision 3, L8 decisions 1 and 8; D-45 in [SR-LAW-11](../records/0013_LAW-11-dependencies.md) §3; the build W-105 |
| [`W-93-hook-pre-and-brief.md`](open/W-93-hook-pre-and-brief.md) | 2026-10-28 | `hook --pre` and `brief --for` in [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) decision 9 and [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 14; `hook_pre` (unset) in [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) and `perf/budgets.toml` |
| [`W-100-rule-promotion.md`](open/W-100-rule-promotion.md) | 2026-10-28 | the promotion criterion in [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 6a; `scripts/corpus.py promote`; `scripts/corpus.json` `promotion.promoted`; the README FAQ |
| [`W-99-side-by-side-bench.md`](open/W-99-side-by-side-bench.md) | 2026-10-28 | context rows in [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) §Measurements; the harness in [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 10; README FAQ *Is it faster than ESLint and Prettier?* |
| [`W-107-oxlint-type-aware-gap.md`](open/W-107-oxlint-type-aware-gap.md) | 2026-10-27 | oxlint type-aware in [SR-LINTER](../records/0106_linter.md) decision 2; not caching it in [SR-ARCHITECTURE](../records/0108_architecture.md) decision 19; USAGE *Type rules today* |
| [`W-94-codelaw-migrate.md`](open/W-94-codelaw-migrate.md) | 2026-10-27 | `codelaw migrate` in [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 15; `[lint] ignore_aliases` in [SR-LINTER](../records/0106_linter.md) decision 6; USAGE *Coming from another tool* |
| [`W-101-type-aware-ts-research.md`](open/W-101-type-aware-ts-research.md) | 2026-10-27 | [proposal](../work/proposals/type-aware-rules.md); the build W-104–W-107 |
| [`W-35-before-while-after-positioning.md`](open/W-35-before-while-after-positioning.md) | 2026-10-27 | the README's *One set of rules, four moments*; `hook --pre` and `brief` in [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) decision 9 and [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 14 |
| [`W-91-fux-auto-bump.md`](open/W-91-fux-auto-bump.md) | 2026-10-27 | [`.github/workflows/fux-bump.yml`](../.github/workflows/fux-bump.yml), `scripts/fux-index.py latest`/`bump`; [SR-PACKAGES](../records/0107_packages.md) decision 14 |
| [`W-98-llms-txt.md`](open/W-98-llms-txt.md) | 2026-10-27 | [`llms.txt`](../llms.txt), [`docs/rules/llms-full.txt`](../docs/rules/llms-full.txt), [`docs/rules/rules.json`](../docs/rules/rules.json), generated by [`scripts/rule-docs.py`](../scripts/rule-docs.py); [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) §3 |
| [`W-90-l11-holds-the-dependency-list.md`](open/W-90-l11-holds-the-dependency-list.md) | 2026-10-27 | [SR-LAW-11](../records/0013_LAW-11-dependencies.md) — L11 "dependencies", its §3 the list and the ledger; `crates/codelaw-cli/tests/dependencies.rs` |
| [`W-85-npm-first-publish.md`](open/W-85-npm-first-publish.md) | 2026-10-27 | [`RELEASING.md`](../RELEASING.md) §One time, by hand and §Every release step 0; [`scripts/npm-trust.sh`](../scripts/npm-trust.sh) |
| [`W-96-agent-format-autodetect.md`](open/W-96-agent-format-autodetect.md) | 2026-10-27 | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 4a |
| [`W-36-position-against-eslint-and-biome.md`](open/W-36-position-against-eslint-and-biome.md) | 2026-10-27 | [proposal](../work/proposals/linter-for-ai-positioning.md); gaps W-94–W-100, W-101, W-102; the README |
| [`W-05-write-time-guidance.md`](open/W-05-write-time-guidance.md) | 2026-10-27 | [proposal](../work/proposals/write-time-guidance.md); the build [W-93](../work/open/W-93-hook-pre-and-brief.md) |
| [`W-44-watch-mode.md`](open/W-44-watch-mode.md) | 2026-10-27 | `codelaw check --watch` / `fix --watch`: [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 22 and 22a, [SR-CLI-MCP](../records/0115_cli-and-mcp.md), `crates/codelaw-engine/src/watch.rs`, the smoke's watch step |
| [`W-71-tests-for-new-code.md`](open/W-71-tests-for-new-code.md) | 2026-10-27 | `graph/untested-function`, `graph/function-coverage`, `.codelawignore` and `codelaw:tests` — their rule pages in [`packs/`](../packs) and USAGE "tests for new code"; `codelaw debug changed-functions` |
| [`W-70-dev-server-example-apps.md`](open/W-70-dev-server-example-apps.md) | 2026-10-27 | [`examples/dev-servers/`](../examples/dev-servers) and CI's `dev-servers` job; [SR-PACKAGES](../records/0107_packages.md) decision 3a |
| [`W-84-ci-green-on-main.md`](open/W-84-ci-green-on-main.md) | 2026-10-27 | CI's `ci.yml` on `main` (run 40437239647, all green); the guard's baseline in [`records/RULE-SINCE`](../records/RULE-SINCE) |
| [`W-51-scheduled-autofix-pr.md`](open/W-51-scheduled-autofix-pr.md) | 2026-10-27 (parked) | [`work/BACKLOG.md`](../work/BACKLOG.md) B-053; the workflow `codelaw init --ci github --autofix` writes, [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| [`W-45-ci-report-formats.md`](open/W-45-ci-report-formats.md) | 2026-10-27 | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 4 (`--format github`, `gitlab`, `junit`, `checkstyle`); USAGE's CI snippets |
| [`W-81-commit-main-checkout.md`](open/W-81-commit-main-checkout.md) | 2026-10-27 | the commit itself; the pages are [`docs/rules/`](../docs/rules/README.md), made by `scripts/rule-docs.py` |
| [`W-58-size-limits.md`](open/W-58-size-limits.md) | 2026-10-27 | the six rule pages in [`packs/js/rules/js/`](../packs/js/rules/js) and [`packs/python/rules/py/`](../packs/python/rules/py) (`max-lines`, `max-lines-per-function`, `max-params`), `crates/codelaw-native/src/size.rs`, [SR-RULE-PACKS](../records/0101_rule-packs.md) (the `size` presets) |
| [`W-73-configurable-approved-calls.md`](open/W-73-configurable-approved-calls.md) | 2026-10-27 | [SR-RULE-FORMAT](../records/0112_rule-format.md) decisions 2, 3a (`calls:`, `function_calls:`) and 3b; the rule pages of the `sanitizers` and `validators` rules, `js/use-instead`, `py/use-instead` |
| [`W-80-session-cleanup.md`](open/W-80-session-cleanup.md) | 2026-10-27 | the W-77 worktree is W-77's live change ([`work/open/`](../work/open/W-77-skills-package.md)) |
| [`W-79-session-cleanup.md`](open/W-79-session-cleanup.md) | 2026-10-27 | branch `keep/W-73-fux-index-2026-10-27` (`e5af708`; was `lm/W-73`, renamed 2026-10-27 so `lockmaster start W-73` could run), the regenerated index kept |
| [`W-74-editor-logs.md`](open/W-74-editor-logs.md) | 2026-10-27 | [SR-EDITOR](../records/0114_editor.md) decision 7; `packaging/vscode/README.md` *Troubleshooting* |
| [`W-78-commit-main-checkout.md`](open/W-78-commit-main-checkout.md) | 2026-10-27 | the commit itself; W-77 lives in [`work/open/`](../work/open/W-77-skills-package.md) |
| [`W-76-commit-main-and-clean-worktrees.md`](open/W-76-commit-main-and-clean-worktrees.md) | 2026-10-27 | the commit itself and the worklog entry |
| [`W-75-commit-main-checkout.md`](open/W-75-commit-main-checkout.md) | 2026-10-27 | the commit itself; the Fux setup is `fux.toml`, `.fux/` and [`AGENTS.md`](../AGENTS.md) |
| [`W-49-dev-server-plugins.md`](open/W-49-dev-server-plugins.md) | 2026-10-26 | [`packaging/npm/`](../packaging/npm) (`dev-core`, `vite-plugin`, `webpack-plugin`, `next`), [SR-PACKAGES](../records/0107_packages.md) decision 3a, [`plugins.test.js`](../packaging/npm/dev-core/test/plugins.test.js); the real-host example apps are [W-70](open/W-70-dev-server-example-apps.md) (closed 2026-10-27) |
| [`W-61-max-lines-html-css.md`](open/W-61-max-lines-html-css.md) | 2026-10-26 | [`size_lines.rs`](../crates/codelaw-native/src/size_lines.rs), [SR-RULE-PACKS](../records/0101_rule-packs.md) (`size` presets) and [`tests/size_limits.rs`](../crates/codelaw-cli/tests/size_limits.rs) |
| [`W-63-css-modules.md`](open/W-63-css-modules.md) | 2026-10-26 | [`packs/css-modules/`](../packs/css-modules/README.md), [SR-PROJECT-GRAPH](../records/0103_project-graph.md) decision 8 and [`tests/css_modules.rs`](../crates/codelaw-cli/tests/css_modules.rs) |
| [`W-60-consumer-templates.md`](open/W-60-consumer-templates.md) | 2026-10-26 | [`templates/`](../templates/README.md), [`template.rs`](../crates/codelaw-core/src/template.rs) and [SR-LAW-10](../records/0012_LAW-10-consumer-templates.md) |
| [`W-67-replace-three-dependencies.md`](open/W-67-replace-three-dependencies.md) | 2026-10-26 | [`DEPENDENCIES.md`](../DEPENDENCIES.md), `crates/codelaw-core/src/similar.rs` and `crates/codelaw-cache/src/codec.rs` ([SR-CACHE](../records/0109_cache.md) decision 4) |
| [`W-65-dependency-check.md`](open/W-65-dependency-check.md) | 2026-10-26 | [`crates/codelaw-cli/tests/dependencies.rs`](../crates/codelaw-cli/tests/dependencies.rs), the ledger in [`DEPENDENCIES.md`](../DEPENDENCIES.md) and [SR-LAW-11](../records/0013_LAW-11-no-dependencies.md) decision 7 |
| [`W-66-ratify-dependencies.md`](open/W-66-ratify-dependencies.md) | 2026-10-26 | [`DEPENDENCIES.md`](../DEPENDENCIES.md) and [SR-LAW-11](../records/0013_LAW-11-no-dependencies.md); the three removals are [W-67](../work/open/W-67-replace-three-dependencies.md) |
| [`W-03-rule-packs.md`](open/W-03-rule-packs.md) | 2026-10-25 | the packs in `packs/` and [SR-RULE-PACKS](../records/0101_rule-packs.md); publishing is [W-37](../work/open/W-37-npm-0-2-0-through-the-pipeline.md) and [W-38](../work/open/W-38-pypi-0-2-0-by-hand.md); the *later* presets are B-048 in [`work/BACKLOG.md`](../work/BACKLOG.md) |
| [`W-04-release-pipeline.md`](open/W-04-release-pipeline.md) | 2026-10-25 | `.github/workflows/`, [`RELEASING.md`](../RELEASING.md) and [SR-PACKAGES](../records/0107_packages.md) decision 11; the first release is W-37 (npm) and W-38 (PyPI) |
| [`W-29-first-npm-publish.md`](open/W-29-first-npm-publish.md) | 2026-10-25 | [`RELEASING.md`](../RELEASING.md) §One time; npm 0.2.0 and the 8-platform install check are [W-37](../work/open/W-37-npm-0-2-0-through-the-pipeline.md) |
| [`W-30-first-pypi-publish.md`](open/W-30-first-pypi-publish.md) | 2026-10-25 | [W-38](../work/open/W-38-pypi-0-2-0-by-hand.md), which absorbed all of it |
| [`W-31-registry-pack-split.md`](open/W-31-registry-pack-split.md) | 2026-10-25 | [SR-LAW-6](../records/0008_LAW-6-python-and-node-packages.md) decision 11 and [SR-RULE-PACKS](../records/0101_rule-packs.md); the 0.2.0 release is [W-37](../work/open/W-37-npm-0-2-0-through-the-pipeline.md) and [W-38](../work/open/W-38-pypi-0-2-0-by-hand.md) |
| [`W-32-redux-and-query-packs.md`](open/W-32-redux-and-query-packs.md) | 2026-10-25 | `packs/tanstack-query/`, `packs/redux-toolkit/` and [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 4; both on npm at 0.2.0 |
| [`W-01-rule-pack-records.md`](open/W-01-rule-pack-records.md) | 2026-10-19 | [SR-RULE-PACKS](../records/0101_rule-packs.md) and [SR-LAW-6](../records/0008_LAW-6-python-and-node-packages.md), the records it rewrote |
| [`W-02-engine-p0.md`](open/W-02-engine-p0.md) | 2026-10-19 | the engine in `crates/`, owned per the register's Ownership table; [SR-ARCHITECTURE](../records/0108_architecture.md) is its design of record; [`work/IMPLEMENTATION.md`](../work/IMPLEMENTATION.md) has what landed |
| [`W-06-consumer-readme.md`](open/W-06-consumer-readme.md) | 2026-10-22 | `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` and `.github/ISSUE_TEMPLATE/`, `.github/pull_request_template.md`; [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) §3 |
| [`W-07-formatter.md`](open/W-07-formatter.md) | 2026-10-21 | `crates/codelaw-format`; [SR-FORMATTER](../records/0105_formatter.md) §What is built and its parity table; [SR-LAW-7](../records/0009_LAW-7-linter-and-formatter.md) §How L7 is enforced; the rest is W-23 |
| [`W-08-mcp-server.md`](open/W-08-mcp-server.md) | 2026-10-20 | `crates/codelaw-mcp`; [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 5–6 |
| [`W-09-lsp-and-extension.md`](open/W-09-lsp-and-extension.md) | 2026-10-21 | `crates/codelaw-lsp`, `packaging/vscode`; [SR-EDITOR](../records/0114_editor.md); what it did not build is W-23, W-15, W-04 and B-043–B-044 |
| [`W-17-engine-defects.md`](open/W-17-engine-defects.md) | 2026-10-20 | the fixes and their regression tests in `crates/`; deviations in [SR-ARCHITECTURE](../records/0108_architecture.md) 15a and [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7 |
| [`W-11-project-graph.md`](open/W-11-project-graph.md) | 2026-10-22 | `crates/codelaw-graph`; [SR-PROJECT-GRAPH](../records/0103_project-graph.md), its Consequences holding the measured false-positive rate |
| [`W-10-agent-surfaces.md`](open/W-10-agent-surfaces.md) | 2026-10-22 | `crates/codelaw-agents` and `agents/`; [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| [`W-12-structured-file-rules.md`](open/W-12-structured-file-rules.md) | 2026-10-22 | `crates/codelaw-match` (`data`, `target`, `schema`) and `crates/codelaw-fix/src/structured.rs`; [SR-RULE-FORMAT](../records/0112_rule-format.md) decision 3e |
| [`W-13-baseline-and-time.md`](open/W-13-baseline-and-time.md) | 2026-10-22 | `crates/codelaw-baseline`, `crates/codelaw-engine/src/{baseline,git}.rs`, `crates/codelaw-graph/src/deprecated.rs`; [SR-BASELINE](../records/0104_baseline-and-deprecations.md) |
| [`W-14-adapters.md`](open/W-14-adapters.md) | 2026-10-22 | `crates/codelaw-adapters` and `crates/codelaw-engine/src/adapters.rs`; [SR-LINTER](../records/0106_linter.md) decision 2 and [SR-ARCHITECTURE](../records/0108_architecture.md) decision 19 |
| [`W-15-warm-processes.md`](open/W-15-warm-processes.md) | 2026-10-22 | `crates/codelaw-cli/src/warm.rs`, `crates/codelaw-engine/src/incremental.rs`, the LSP's tree cache; [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 21–24 |
| [`W-16-commands-and-api.md`](open/W-16-commands-and-api.md) | 2026-10-22 | `crates/codelaw-cli`, `codelaw.schema.json` and the npm/Python API; [SR-CLI-MCP](../records/0115_cli-and-mcp.md), [SR-CONFIG](../records/0110_config.md) and [SR-PACKAGES](../records/0107_packages.md) |
| [`W-23-formatter-rest.md`](open/W-23-formatter-rest.md) | 2026-10-22 | `crates/codelaw-format`, `crates/codelaw-test`'s layout refusal, `crates/codelaw-lsp` range formatting; [SR-FORMATTER](../records/0105_formatter.md) §What is built and decision 3; B-045 |
| [`W-22-record-drift.md`](open/W-22-record-drift.md) | 2026-10-20 | [SR-RULE-PACKS](../records/0101_rule-packs.md), [SR-PACKAGES](../records/0107_packages.md) decision 9, [SR-ARCHITECTURE](../records/0108_architecture.md) decision 1 |
| [`W-21-pack-engine-gaps.md`](open/W-21-pack-engine-gaps.md) | 2026-10-24 | the engine in `crates/codelaw-lang` (regions, the tree-sitter-css fork) and `crates/codelaw-engine` (file context); [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 3 and 15b; the upstream PR is [B-046](../work/BACKLOG.md), the CSS-tree port [B-047](../work/BACKLOG.md) |
| [`W-28-layout-check-by-tokens.md`](open/W-28-layout-check-by-tokens.md) | 2026-10-24 | `codelaw_lang::token_stream` and `codelaw-test`'s layout check; [SR-FORMATTER](../records/0105_formatter.md) decision 3, [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 4 |
| [`W-26-public-docs-from-the-tree.md`](open/W-26-public-docs-from-the-tree.md) | 2026-10-24 | `scripts/docs-sync.py` and the README/CHANGELOG regions it keeps; [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) Consequences |
| [`W-25-governance-drift.md`](open/W-25-governance-drift.md) | 2026-10-24 | the work system's checks: `scripts/check-work-system.py` (rules 40, 64; SR-WORK-OWNERSHIP decisions 2, 9); [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md), [SR-WORK-OWNERSHIP](../records/0058_WORK-ownership.md) |
| [`W-27-engine-and-packaging-hygiene.md`](open/W-27-engine-and-packaging-hygiene.md) | 2026-10-25 | MSRV 1.97 and the `msrv` job, `codelaw_core::process::kill_group`, Windows one-shot asserted by `smoke.sh`, `version.py`, one pack builder; [SR-ARCHITECTURE](../records/0108_architecture.md) decision 21, [SR-PACKAGES](../records/0107_packages.md) |
| [`W-18-release-hardening.md`](open/W-18-release-hardening.md) | 2026-10-25 | `ci.yml`/`publish.yml` hardening, `SHA256SUMS` and attestation, the eight-platform `smoke` job from a staging index, tag-only environments; [SR-PACKAGES](../records/0107_packages.md), `RELEASING.md` |
| [`W-39-ci-and-local-profiles.md`](open/W-39-ci-and-local-profiles.md) | 2026-10-25 | `ci`/`local` profiles picked per surface, per-glob `remap`, `js/no-console`; [SR-CONFIG](../records/0110_config.md) decisions 10–11 |
| [`W-40-dated-exceptions.md`](open/W-40-dated-exceptions.md) | 2026-10-25 | `expires`, `warn_days`, `require_expiry`, `added_by` checked against git, on baselines, deprecations and inline ignores; [SR-BASELINE](../records/0104_baseline-and-deprecations.md) decision 9 |
| [`W-41-react-no-context.md`](open/W-41-react-no-context.md) | 2026-10-25 | `react/no-context` and its baseline recipe (old uses warn, new ones fail); `fixtures/react-no-context`, `tests/react_no_context.rs`; [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 8 |
| [`W-43-react-hooks-rules.md`](open/W-43-react-hooks-rules.md) | 2026-10-25 | ten React hooks rules from eslint-plugin-react-hooks 7 and @eslint-react in `react/recommended` (`react_hooks.rs`); [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7, [SR-ARCHITECTURE](../records/0108_architecture.md) 15b |
| [`W-42-tanstack-query-hooks-rules.md`](open/W-42-tanstack-query-hooks-rules.md) | 2026-10-25 | six `tanstack-query/*` rules from TanStack Query v5's guides (`tanstack_query.rs`); the pack's corpus; [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7, [SR-ARCHITECTURE](../records/0108_architecture.md) 15b |
| [`W-37-npm-0-2-0-through-the-pipeline.md`](open/W-37-npm-0-2-0-through-the-pipeline.md) | 2026-10-25 | `publish.yml`'s npm-only run by hand on a `v*` tag; `v0.2.0` on `08b7158`; npm 0.2.0 from CI (run 40207195373); [L6](../records/0008_LAW-6-python-and-node-packages.md) decision 9, [SR-PACKAGES](../records/0107_packages.md) decision 11 |
| [`W-19-performance-budgets.md`](open/W-19-performance-budgets.md) | 2026-10-25 | `codelaw bench` (and `--repo`), the reference workload, the `perf` gate, seven L8 budgets set by the ruled method, `fix_one_file` unset by ruling; [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md), [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 10 |
| [`W-33-stencil-preset-gaps.md`](open/W-33-stencil-preset-gaps.md) | 2026-10-25 | `stencil/ban-side-effects` and `stencil/enforce-slot-jsdoc` in `crates/codelaw-native/src/stencil.rs` and `stencil/recommended`; the pack README's count; [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7, [SR-ARCHITECTURE](../records/0108_architecture.md) 15b |
| [`W-24-ci-green-on-main.md`](open/W-24-ci-green-on-main.md) | 2026-10-25 | the Windows `bench` build, `RULE-SINCE` at `5e489f4`, verdict-cache keys over every test input, the law gate for real, `perf` only on engine changes, the UTF-8 inbox check, the macOS staging-index stall; [SR-WORK-OWNERSHIP](../records/0058_WORK-ownership.md) decision 7, [SR-PACKAGES](../records/0107_packages.md) decision 11, [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) decision 7 |
| [`W-20-conformance-tests.md`](open/W-20-conformance-tests.md) | 2026-10-25 | the L5 fence, the L2 licence check, the consumer-layout test, `codelaw test --corpus` with the pinned corpus and its `corpus` job, `check-work-system.py`; [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 4 |
| [`W-48-codelaw-migrate.md`](open/W-48-codelaw-migrate.md) | 2026-10-25 | not built — moved to the backlog by the decider as [B-050](../work/BACKLOG.md); the spec is [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) §P2 `codelaw-migrate` |
| [`W-50-git-hooks.md`](open/W-50-git-hooks.md) | 2026-10-26 | `codelaw init --git-hooks` (`crates/codelaw-cli/src/git_hooks.rs`, `hooks/*.sh`, `tests/git_hooks.rs`), `.pre-commit-hooks.yaml`; [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 11 |
| [`W-46-exceptions-report.md`](open/W-46-exceptions-report.md) | 2026-10-26 | `codelaw exceptions` and MCP `list_exceptions` (`crates/codelaw-engine/src/exceptions.rs`, `crates/codelaw-report/src/exceptions.rs`, `tests/exceptions.rs`); [SR-BASELINE](../records/0104_baseline-and-deprecations.md) decision 11, [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 1 and 5 |
| [`W-47-baseline-burn-down.md`](open/W-47-baseline-burn-down.md) | 2026-10-25 | `codelaw baseline history` (`crates/codelaw-baseline/src/history.rs`, `tests/baseline_history.rs`); [SR-BASELINE](../records/0104_baseline-and-deprecations.md) decision 10 |
| [`W-52-dependency-latest-versions.md`](open/W-52-dependency-latest-versions.md) | 2026-10-25 | `codelaw deps refresh`, `data/outdated-dependency`, `data/stale-deps-snapshot`; [L5](../records/0007_LAW-5-no-model-in-the-engine.md) decision 3, [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 15; refresh in the auto-fix PR is W-51's; the agent hint is B-052, `requirements*.txt` B-051 |
| [`W-53-security-packs-groundwork.md`](open/W-53-security-packs-groundwork.md) | 2026-10-25 | the four security-pack scaffolds, `security:` metadata, `from_source:`, preset `extends`; [L6](../records/0008_LAW-6-python-and-node-packages.md), [SR-RULE-FORMAT](../records/0112_rule-format.md) decisions 2a and 3a; the catalogues are W-54–W-57 |
| [`W-56-security-html-pack.md`](open/W-56-security-html-pack.md) | 2026-10-26 | the 19 `sec-html/*` rules and two presets: [`packs/security-html/`](../packs/security-html/README.md), `crates/codelaw-native/src/html_security.rs`; [SR-RULE-PACKS](../records/0101_rule-packs.md) |
| [`W-54-security-js-pack.md`](open/W-54-security-js-pack.md) | 2026-10-26 | `packs/security-js`: 66 rules (55 `recommended`, 11 `strict`), `data/sources/`, the README's not-ported table, corpus counts; [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 3; differences from the catalogue are at the foot of the item |
| [`W-55-security-ts-pack.md`](open/W-55-security-ts-pack.md) | 2026-10-26 | `packs/security-ts`: 16 rules (10 `recommended`, 6 `strict`), syntax-only, extending `security-js`; [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 3 |
| [`W-57-security-python-pack.md`](open/W-57-security-python-pack.md) | 2026-10-26 | `security-python`: 61 `match:` rules (Bandit and Ruff `S` ported, Django, Flask, FastAPI, ML loaders, LLM sinks) with examples, cases, `security:` metadata and a fix each, `data/sources/`, both presets, the README's Bandit table, the pack's corpus; [SR-RULE-PACKS](../records/0101_rule-packs.md), [SR-RULE-FORMAT](../records/0112_rule-format.md) decisions 2a and 3a |

## `backlog/`

A backlog row has no file, so nothing of it is archived but its id: each
deleted `B-nnn` that a document still names gets a row here
([SR-WORK-ARCHIVE](../records/0057_WORK-archive.md) decision 3). B-006, B-007,
B-043, B-044 and B-045 are live in [`work/BACKLOG.md`](../work/BACKLOG.md).

| id | deleted | what absorbed it |
|---|---|---|
| B-001–B-005 | 2026-10-20 | W-20: `scripts/check-work-system.py` — the queue's row rules, `owns:` ↔ the Ownership table, the registry, lost `W-nn` ids, one archive |
| B-008 | 2026-10-20 | W-02 (Cargo) and W-20 (npm, PyPI tools): `crates/codelaw-cli/tests/licences.rs` |
| B-009 | 2026-10-20 | W-02: `crates/codelaw-cli/tests/laws.rs` `l3_*`; the runtime fence and install matrix are W-18's (`scripts/smoke.sh`) |
| B-010 | 2026-10-19 | W-02: `crates/codelaw-cli/tests/laws.rs` `l4_*` |
| B-011 | 2026-10-20 | W-02 (determinism, CLI/library parity: `laws.rs` `l5_*`) and W-20 (`tests/network_fence.rs`) |
| B-012 | 2026-10-20 | W-04 (`scripts/version.py --check`) and W-18 (the install matrix, `scripts/smoke.sh`) |
| B-013 | 2026-10-20 | W-11: the corpus run in SR-PROJECT-GRAPH's Consequences |
| B-014 | 2026-10-20 | W-13: the veto check in SR-BASELINE, `crates/codelaw-baseline/src/tests.rs` |
| B-015 | 2026-10-20 | W-07: `scripts/format-parity.py`, the parity table in SR-FORMATTER |
| B-016 | 2026-10-20 | W-19 (`codelaw bench`) and the region tests named in SR-LINTER's veto |
| B-017 | 2026-10-20 | W-07: `crates/codelaw-cli/tests/laws.rs` `l7_*` |
| B-018 | 2026-10-20 | W-19: `codelaw bench`, `perf/budgets.toml`, `ci.yml`'s `perf` job |
| B-019 | 2026-10-20 | W-19: the proposed budgets; setting them is the decider's (the OPEN-WORK inbox) |
| B-020 | 2026-10-20 | W-02 (`crates/codelaw-cli/tests/layers.rs`) and W-04 (the names check in `scripts/package.py check`) |
| B-021 | 2026-10-19 | W-02: `crates/codelaw-engine/tests/engine.rs` `cold_warm_and_partial_cache_agree` |
| B-022 | 2026-10-20 | W-16: the schema-vs-records test |
| B-023 | 2026-10-20 | W-20: `crates/codelaw-cli/tests/consumer_layout.rs` |
| B-024 | 2026-10-20 | W-20: `codelaw rules lint`, `crates/codelaw-cli/tests/promotion.rs`; the corpus job is still W-20's |
| B-025 | 2026-10-20 | W-08: `crates/codelaw-cli/tests/mcp_parity.rs`; the LSP side is W-09's |
| B-026 | 2026-10-20 | W-17 (the panic fixture) and W-15 (stale-socket cleanup) |
| B-027 | 2026-10-20 | W-19: the cold targets measured by `codelaw bench` |
| B-028 | 2026-10-20 | W-07 and W-23: `crates/codelaw-format` |
| B-029 | 2026-10-20 | W-11: `crates/codelaw-graph` |
| B-030 | 2026-10-20 | W-12: `target:` and `assert:` rules, the structured fix primitives |
| B-031 | 2026-10-20 | W-13: `crates/codelaw-baseline` |
| B-032 | 2026-10-20 | W-14: `crates/codelaw-adapters` |
| B-033 | 2026-10-20 | W-08: `crates/codelaw-mcp` |
| B-034 | 2026-10-20 | W-09: `crates/codelaw-lsp`, `packaging/vscode` |
| B-035 | 2026-10-20 | W-10: `crates/codelaw-agents`, `agents/` |
| B-036 | 2026-10-20 | W-15: the warm processes |
| B-037 | 2026-10-20 | W-16 (the commands and flags), W-13 (`--changed-since`, `--staged`), W-19 (`bench`) |
| B-038 | 2026-10-20 | W-04: the packaging and the release pipeline |
| B-039 | 2026-10-20 | W-21: tagged templates as regions |
| B-040 | 2026-10-20 | W-21: native rules' file context |
| B-041 | 2026-10-20 | W-21: the in-tree tree-sitter-css fork |
| B-042 | 2026-10-20 | W-16: `rulesFor`/`rules_for`, `format` and `codelaw.schema.json` in both APIs |

## `records/`

| file | retired | ruling | successor |
|---|---|---|---|

## `compare/`

| file | decided | the trigger lives in |
|---|---|---|

## `proposals/`

| file | outcome | successor |
|---|---|---|

## Root files

| file | archived | ruling | successor |
|---|---|---|---|
| [`DEPENDENCIES.md`](DEPENDENCIES.md) | 2026-10-27 | the decider, W-90 (voice docket): *"Move it to archive."* | [SR-LAW-11](../records/0013_LAW-11-dependencies.md) §3 — the list, its statuses and the ledger |

## `bootstrap/`

| file | archived | successor |
|---|---|---|
| `WORK-SYSTEM.md` | 2026-10-18 | the records `0001`–`0003` and `0051`–`0060`, which are now the only home of every rule it carried |
