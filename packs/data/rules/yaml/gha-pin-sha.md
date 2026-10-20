---
id: yaml/gha-pin-sha
title: "GitHub Actions are pinned to a commit SHA"
languages: [yaml]
files: ["**/.github/workflows/*.yml", "**/.github/workflows/*.yaml", "**/action.yml", "**/action.yaml"]
fix: manual
fix_reason: "finding the commit a tag points to needs the network, and the engine never uses it (L5)"
stability: preview
since: 0.1.0
---

A `uses:` in a GitHub Actions workflow or composite action names a full 40-character commit SHA, not a tag or branch. Local actions (`./…`) and `docker://` images are not checked.

**Why:** A tag or branch can be moved to different code at any time, so a compromised or careless upstream changes what your workflow runs; a commit SHA can't change.

```codelaw
match:
  kind: flow_node
  regex: "@"
  inside:
    kind: block_mapping_pair
    has: { field: key, regex: '^uses$' }
  all:
    - not: { regex: '@[0-9a-fA-F]{40}' }
    - not: { regex: '^[''"]?(\./|docker://)' }
message: "Pin `$$MATCH` to a full commit SHA; a tag or branch can be moved to other code"
```

## How to fix

1. Look up the commit the tag points to, for example with `git ls-remote https://github.com/<owner>/<repo> <tag>`.
2. Replace the tag with the 40-character SHA and keep the tag as a comment: `uses: actions/checkout@<sha> # v4.2.2`.
3. Let Dependabot or Renovate update the pinned SHAs.

## Examples

### YAML

#### Don't

```yaml path=.github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
```

#### Do

```yaml path=.github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
```
