# L3 fixture matrix

One folder of files per L3 language, and one set of fixture rules covering all
nine. `crates/codelaw-cli/tests/laws.rs` builds a repo per language (the rules
plus that language's files) and one combined repo (the rules plus every
language's files), runs `check`, `fix` and `test` in each, and requires every
file's diagnostics and fixed bytes to be identical alone and together
([SR-LAW-3](../../records/0005_LAW-3-languages.md) decisions 3–6).
