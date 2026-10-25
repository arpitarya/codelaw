# react/no-context adoption fixture

A repo that uses React context in two files: `src/theme.js` (`createContext` and `useContext`)
and `src/Header.jsx` (`React.useContext`). `codelaw.toml` turns `react/no-context` on as an
error. `crates/codelaw-cli/tests/react_no_context.rs` copies it, with `packs/react` as
`node_modules/@codelaw/rules-react`, into a temporary folder and runs the adoption recipe on the
rule's page (W-41):

| step | command | exit |
|---|---|---|
| before the baseline | `codelaw check --as-of 2026-10-25` — three errors | 1 |
| grandfather today's uses | `codelaw baseline add --rule react/no-context --warn-from 2026-10-25 --on-touch keep --reason "legacy context, grandfathered"` | 0 |
| untouched old uses | `codelaw check --as-of 2026-10-25` — three warnings | 0 |
| a new file calls `useContext` | `codelaw check --as-of 2026-10-25` — that file errors, the old uses still warn | 1 |
| one more `useContext` in `Header.jsx` | `codelaw check --as-of 2026-10-25` — both uses in that file error | 1 |
| `codelaw explain react/no-context` | shows the recipe | 0 |
