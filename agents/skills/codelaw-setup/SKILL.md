---
name: codelaw-setup
description: Set codelaw up in a repository — install the engine and the rule packs for the languages present, write the config, wire pre-commit and CI, and add agent files only if asked. Use when the user says "set up codelaw", "add codelaw to this repo", "add a linter that agents follow", "install codelaw", or asks to lint and format a repo with codelaw for the first time. Not for a repo that already has a codelaw config (use codelaw-upgrade or edit the config instead).
---

# codelaw-setup — one repo, one config, the right packs

`codelaw init` does the detection and writes the files; your job is to run it
with the right flags, finish what it prints, and verify the result.

## 1. Look before you run

- Is there already a config? `codelaw.toml`, `[tool.codelaw]` in
  `pyproject.toml`, or a `"codelaw"` key in `package.json`. If so, stop:
  `init` refuses, and this is not a setup job.
- Which package manager does the repo use? A `package.json` with a lockfile
  (`pnpm-lock.yaml`, `yarn.lock`, `bun.lock`, `package-lock.json`) means npm
  packages; a `pyproject.toml` with `uv.lock`, `poetry.lock` or a
  `requirements*.txt` means PyPI. A repo with both takes the Python pack from
  PyPI and the rest from npm, unless the user picks one registry.
- Does the user want agent files, and for which agents? Do not add them
  unless asked.

## 2. Install the engine

- npm repo: add `codelaw` as a dev dependency with the repo's own manager
  (`pnpm add -D codelaw`, `npm i -D codelaw`, `yarn add -D codelaw`,
  `bun add -d codelaw`).
- Python repo: `uv add --dev codelaw` (or `poetry add --group dev codelaw`,
  or `pip install codelaw` into the project's virtualenv).

Both packages are the same engine with identical output.

## 3. Run init

`codelaw init` — detects the languages, writes `codelaw.toml` with `extends`
set to the matching presets, creates `.codelaw/rules/` with a README, adds
`.codelaw/cache/` to `.gitignore`, and installs the rule packs
(`@codelaw/rules-*` on npm, `codelaw-rules-*` on PyPI) with the repo's
package manager.

Flags:

- `--registry npm` or `--registry pypi` — take every pack from one registry;
- `--no-install` — print the install commands instead of running them (then
  run them yourself, or show them to the user);
- `--agents claude,cowork,copilot,codex,kiro` — only the agents the user
  named; it records them in `[agents] targets` and runs `codelaw sync-agents`.

Read everything `init` prints; any command it says to run is part of setup.

## 4. Verify

1. `codelaw rules` lists rules from the packs (not only `codelaw/no-rules`).
2. `codelaw check` runs. On an existing codebase it will find things; report
   the counts by rule (`--format json`, `summary` and `diagnostics[].rule`).
   Do **not** fix the whole repo as part of setup unless the user asks — offer
   `codelaw fix` (auto fixes only) as the next step.
3. With agents: `codelaw sync-agents --check` exits 0.

## 5. Wire pre-commit and CI (if the user wants it)

- Pre-commit: run `codelaw check` on staged files (for example a
  `pre-commit` hook calling `codelaw check $(git diff --cached --name-only)`,
  or the repo's existing hook tool).
- CI: a job that installs dependencies and runs `codelaw check`; with agent
  files, also `codelaw sync-agents --check`. Exit codes: `0` clean, `1`
  errors remain, `2` config, rule or usage error.

Follow the repo's existing CI layout; do not add a second CI system.

## Done when

Config written, packs installed, `codelaw check` runs, and the user has a
summary: languages detected, presets enabled, finding counts, files written.

## Never

- **Never switch rules off to make the first run look clean.**
- **Never write agent files the user did not ask for**, and never edit files
  in the user's home directory (codelaw prints user-level snippets instead).
