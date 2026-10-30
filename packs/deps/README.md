# @codelaw/rules-deps

codelaw's dependency rules — one of the first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 15), on npm and
PyPI (`codelaw-rules-deps`). It tells you, and your coding agent, which npm
and Python dependencies are vulnerable and the exact version to move to, and
which are behind their latest release.

`codelaw init` adds it when the repo has a `package.json` or `pyproject.toml`.

```toml
# codelaw.toml
extends = ["codelaw:deps/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

```sh
npm i -D @codelaw/rules-deps      # or
pip install codelaw-rules-deps
codelaw deps refresh              # the one command that reads the network
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | what it holds |
|---|---|---|
| `deps/recommended` | 4 | Vulnerable and outdated dependencies of package.json and pyproject.toml, from the committed snapshot `codelaw deps refresh` writes |

<!-- /docs-sync:presets -->

## Rules



| rule | fix | what it reports |
|---|---|---|
| `deps/vulnerable` | suggest (manual with no clean version) | a version in use, direct or transitive, that an advisory affects, with the lowest known-clean version to move to |
| `deps/incomplete-scan` | manual | an enabled advisory source the last refresh could not read |
| `deps/outdated` | suggest | a dependency behind its latest version, or a deprecated or yanked one (was `data/outdated-dependency`) |
| `deps/stale-snapshot` | manual | a snapshot older than `[deps] max_snapshot_age`, or one that cannot be read (was `data/stale-deps-snapshot`) |



## How it works

The rules read `.codelaw/deps-snapshot.json`, which `codelaw deps refresh`
writes and the project commits, and the lockfiles — never the network, so
`codelaw check` gives the same answer offline and in CI.

- **Advisory sources** are `[deps] advisory_sources`, each on or off:
  `registry` (the npm registry and Python index you already install from)
  first, then `osv` (`https://api.osv.dev`), or a mirror of either. Every
  enabled source is read and merged — the safer answer wins. A source that
  cannot be read is recorded, and `deps/incomplete-scan` says so.
- **The version to move to** is the lowest version every advisory leaves
  clean and some source actually covered, inside the major in use first.
- **Agents** ask `check_dependency` (MCP) or `codelaw deps check` before
  adding a package; `[deps] agent` sets, per severity, whether the hooks
  only inform or also block.

The pack holds no advisory data: the snapshot keeps ids, ranges, versions, a
severity, a URL and the sources' names — never an advisory's text.
