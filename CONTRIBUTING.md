# Contributing to codelaw

Thanks for helping. **The most useful contribution is a false positive with a five-line reproduction.** Every rule gets better from real code.

---

## Ways to help

| you have | do this |
|---|---|
| A finding that's wrong | [Open a false-positive issue](https://github.com/arpitarya/codelaw/issues/new?template=false_positive.yml). Include the smallest snippet that triggers it and the rule id. |
| A crash or a wrong fix | [Open a bug report](https://github.com/arpitarya/codelaw/issues/new?template=bug_report.yml). Include `codelaw --version`, the command and the file. |
| A rule you want | [Open a rule request](https://github.com/arpitarya/codelaw/issues/new?template=rule_request.yml). If it exists in ESLint, Ruff, Pylint, stylelint, html-validate, markdownlint or any other tool, link the original; for a port, title it `port: <tool>/<rule>`. The `codelaw-port-rule` skill can write the rule into your project first and open this form pre-filled — only if you say yes. |
| A fix of your own | Open a pull request (see below). |
| A security issue | Don't open a public issue. See [SECURITY.md](SECURITY.md). |

---

## Build and test

You need a Rust toolchain, 1.85 or newer.

```bash
git clone https://github.com/arpitarya/codelaw.git
cd codelaw
cargo build --release              # the binary is target/release/codelaw
cargo test --workspace             # engine, law checks, layer and licence checks
cargo clippy --workspace           # must be clean
```

Test a rule pack against its own examples, and lint its rule files:

```bash
target/release/codelaw test packs/js
target/release/codelaw rules lint packs/js
python3 scripts/build-packs.py --verify target/release/codelaw   # the pack tarballs and wheels, in dist/packs/
```

---

## Changing a rule

- **A rule is one file**: `packs/<pack>/rules/<namespace>/<name>.md`. The format is [SR-RULE-FORMAT](records/0112_rule-format.md), and the authoring guide is [SR-RULE-AUTHORING](records/0113_rule-authoring.md).
- **Fix a false positive in two steps.** First, add the reproduction as a `Do` example (or as a case in `<name>.cases.md`) and watch `codelaw test` fail. Then fix the rule.
- **Run it over real code.** Examples alone hide false positives, so check a few real repos before you open the PR.
- **Ports say where they come from.** A rule ported from another linter sets `ported_from:` and lists any behaviour gap under `## Differences from the source`.
- **Every rule has a fix**: `auto` where it's safe, otherwise `suggest` or `manual` ([L4](records/0006_LAW-4-every-rule-has-a-fix.md)).

---

## Which ports people want

`scripts/ported-rule-demand.py` counts, for each source rule, the public repositories whose
`.codelaw/rules/` carry a port of it (`ported_from:`, through `gh search code`) and the `port: <tool>/<rule>`
or `ported-rule` issues on this repository; the most wanted are candidates for a first-party pack. A
maintainer runs it with `gh` logged in; it is never in CI and no part of the engine.

```bash
python3 scripts/ported-rule-demand.py                      # one row per source rule: repos, issues, first seen
python3 scripts/ported-rule-demand.py --format json        # the same, as data
python3 scripts/ported-rule-demand.py --dates --save raw.json   # first commit per file; keep what gh returned
python3 scripts/ported-rule-demand.py --from-json raw.json # offline, from a saved run
python3 scripts/ported-rule-demand.py --self-test          # offline check of the parsing
```

---

## How the project is run

codelaw is governed by **standing records**: one Markdown file per decision, in [`records/`](records/README.md). Read these before a larger change.

- **The records are the spec.** When code and a record disagree, the record wins, or both change together ([L0](records/0002_LAW-0-authority.md)).
- **A change under `crates/` updates the record that owns that crate.** The owner is in the register's Ownership table. Install the guard once:

  ```bash
  ln -sf ../../scripts/sr-guard.sh .git/hooks/commit-msg
  ```

- **Dependencies must be free, open source and permissively licensed.** No copyleft and nothing paid, dev tooling included ([L2](records/0004_LAW-2-foss-dependencies.md)). `cargo test` checks this.
- **No AI model in the engine** ([L5](records/0007_LAW-5-no-model-in-the-engine.md)). Output must be deterministic.
- **What's being worked on** is in [`work/OPEN-WORK.md`](work/OPEN-WORK.md); what's named but not started is in [`work/BACKLOG.md`](work/BACKLOG.md).

---

## Pull requests

1. **Keep it to one change.** A rule fix, a new rule, or one engine change.
2. **Add tests.** For a rule, that's examples or cases; for the engine, a Rust test.
3. **Run** `cargo test --workspace`, `cargo clippy --workspace` and `codelaw test` on any pack you touched.
4. **If behaviour changes, update the owning record and add a line to** [CHANGELOG.md](CHANGELOG.md) under *Unreleased*.

By contributing, you agree that your contribution is released under the project's [MIT licence](LICENSE), and to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
