# yaml/gha-pin-sha — edge cases

### case: local action and docker image

```yaml
steps:
  - uses: ./.github/actions/setup
  - uses: docker://alpine:3.20
```

```expect
(none)
```

### case: reusable workflow on a branch

```yaml
jobs:
  call:
    uses: octo/repo/.github/workflows/ci.yml@main
```

```expect
3:11 yaml/gha-pin-sha
```
