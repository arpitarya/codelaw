# fixtures/types

The repo `crates/codelaw-cli/tests/types.rs` checks with the `ts/type-checked` preset (SR-TYPES):
each file under `src/` makes one type rule fire, `clean.ts` none. Its TypeScript is the one
`package.json` pins (L11 §3 D-45, tests and CI only): `npm ci` in this folder installs it, and the
test is skipped, with a note, when it or Node is missing (it fails instead under
`CODELAW_TYPES_REQUIRED=1`, as CI sets).
