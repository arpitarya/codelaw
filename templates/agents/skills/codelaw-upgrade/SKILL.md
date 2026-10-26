---
name: codelaw-upgrade
description: Upgrade codelaw and its rule packs to a new version deliberately. Use when the user says "upgrade codelaw", "bump codelaw", "a new codelaw version is out", or a dependency bot opened a codelaw upgrade. Reads the changelog, upgrades the engine and every pack together, runs the new rules and compares findings, then fixes or adopts each change on purpose and re-syncs agent files.
---

# codelaw-upgrade — engine and packs together, changes on purpose

codelaw and its five first-party packs share **one version** and must be
upgraded together. A minor release may add findings (a stable rule that finds
more, a changed fix); an upgrade is done when every new finding has been
fixed or consciously adopted, not when the version number changed.

## 1. Before

1. `codelaw --version` and the installed pack versions (lockfile).
2. A baseline of today's findings:
   `codelaw check --format json > /tmp/codelaw-before.json` (anywhere outside
   the repo).
3. Read the changelog entries between the two versions: new rules, rules
   that find more, changed fixes, deprecations, config changes.

## 2. Upgrade everything at once

With the repo's package manager, set `codelaw` and every `@codelaw/rules-*`
(npm) or `codelaw-rules-*` (PyPI) dependency to the **same** new version, and
install. A third-party pack keeps its own version but check its
`min_codelaw`.

`codelaw rules` must load without errors. A config key that the new version
refuses is reported with the file, the line and a fix; apply it.

## 3. Compare

`codelaw check --format json > /tmp/codelaw-after.json` and compare
`diagnostics[]` by `rule` and `file`:

- **new findings** — group by rule; for each rule, `explain_rule` and the
  changelog say why;
- **gone findings** — usually a fix or a removed false positive; note them.

To try new preview rules before adopting them, run with `preview = true` set
temporarily and compare again.

## 4. Fix or adopt, rule by rule

For each rule with new findings, with the user:

- **fix**: `codelaw fix` for `auto`, then the `codelaw-fix` loop for the rest;
- **adopt later**: if the repo uses a baseline, add the new findings to it;
  otherwise agree a follow-up;
- **decline**: setting the rule off is a config decision the user makes
  explicitly, with a comment saying why.

## 5. Agent files and CI

If the repo has `[agents] targets`, run `codelaw sync-agents` and commit the
regenerated files (new skills or rule one-liners); `codelaw sync-agents
--check` must pass. Run the CI command locally once.

## Done when

One version everywhere, `codelaw check` result understood rule by rule, agent
files synced, and a summary for the user: version from → to, new rules, new
findings and what was done with each.

## Never

- **Never upgrade the engine and the packs to different versions.**
- **Never switch new rules off wholesale** to keep the diff small.
