---
name: codelaw-pack
description: Share codelaw rules across repositories as a rule pack. Use when the user says "share our rules across repos", "publish a rule pack", "make our .codelaw/rules reusable", or "create a team codelaw pack for npm or PyPI". Scaffolds the pack folder with its manifest, presets, rules, tests, licence and versioning, wires the npm and/or PyPI publish config, and shows how consumers extend it.
---

# codelaw-pack — one folder, both registries

A rule pack is a folder of data: a manifest, presets, rule files and data
tables. It contains **no code and no dependencies**. The same folder is the
npm package, and the package data of a pure-Python wheel.

## 1. Decide with the user

- the pack name: npm `@<scope>/codelaw-rules[-<topic>]`, PyPI
  `<org>-codelaw-rules[-<topic>]`;
- the registries: npm, PyPI or both (both if the team has Node and Python
  repos);
- the namespace the rules use (the team's, never a reserved first-party one:
  `js`, `ts`, `react`, `node`, `stencil`, `py`, `json`, `yaml`, `toml`, `md`,
  `data`, `css`, `html`, `graph`, `format`, `codelaw`);
- which rules move in (usually the team's `.codelaw/rules/`), and the
  licence.

## 2. Scaffold the folder

```text
<pack>/
├─ codelaw-pack.toml              name, version, min_codelaw, languages, presets, description
├─ presets/<namespace>/<preset>.toml   [rules] the preset turns on, severity or { severity, options }
├─ rules/<namespace>/<name>.md    the rule files, + <name>.cases.md
├─ data/                          optional JSON tables rules read
├─ README.md                      the presets and their rules
└─ LICENSE
```

`codelaw-pack.toml`:

```toml
name = "@acme/codelaw-rules"
version = "1.0.0"
min_codelaw = "<the codelaw version you test with>"
languages = ["typescript", "tsx"]
presets = ["acme/recommended"]
description = "Acme's codelaw rules"
```

Every rule a preset names must be in the pack. Move rule files with their
`.cases.md` sidecars, unchanged.

## 3. Test the pack

`codelaw test <pack folder>` runs every rule's examples and cases;
`codelaw rules lint <pack folder>/rules` checks the template. Both must pass.
Then, in a scratch repo, `extends = ["path:<pack folder>"]` and run
`codelaw rules` and `codelaw check` to see it load as consumers will.

A pack whose rules fail to load is refused as a whole, so never publish a
failing pack.

## 4. Publish config

- **npm:** a `package.json` with `name`, `version` (equal to the manifest's),
  `license`, `files` listing the pack's folders, and no `dependencies`.
- **PyPI:** a `pyproject.toml` building a pure `py3-none-any` wheel whose
  `<org>_codelaw_rules` package includes the pack folder as package data, and
  no dependencies.
- Keep the version in one place and check both copies agree in CI.

Do not publish yourself unless the user asks; publishing is theirs.

## 5. Consumers

```toml
extends = ["npm:@acme/codelaw-rules"]      # or "pypi:acme-codelaw-rules", or "path:../shared/rules"
```

An `npm:`/`pypi:`/`path:` entry enables every rule in the pack at its own
severity; local rules with the same id shadow pack rules. A pack's external
fixers stay disabled unless a consumer lists the pack in `allow_pack_fixers`.

## Never

- **Never put code or dependencies in a pack.**
- **Never publish with failing `codelaw test`.**
