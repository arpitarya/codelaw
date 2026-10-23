## What this changes

<!-- One change per pull request: a rule fix, a new rule, or one engine change. -->

## Why

<!-- Link the issue, for example "Fixes #123". For a false positive, paste the snippet. -->

## Checklist

- [ ] Tests added: examples or cases for a rule, a Rust test for the engine.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `codelaw test packs/<pack>` passes for every pack touched.
- [ ] If behaviour changed: the owning record under `records/` is updated, and `CHANGELOG.md` has a line under _Unreleased_.
- [ ] No new dependency, or it is free, open source and permissively licensed (L2).

By contributing, I agree that my contribution is released under the project's [MIT licence](https://github.com/arpitarya/codelaw/blob/main/LICENSE) and to follow the [Code of Conduct](https://github.com/arpitarya/codelaw/blob/main/CODE_OF_CONDUCT.md).
