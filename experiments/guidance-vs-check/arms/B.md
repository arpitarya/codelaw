
## Rule docs and the codelaw tools

- Every rule this repo runs has a page in `docs/codelaw-rules/` (what it checks,
  why, and how to fix it); the project's own rules are the files in
  `.codelaw/rules/acme/`.
- The codelaw MCP server is connected: `check_snippet` and `check_paths` check
  code, `explain_rule` explains a rule and `rules_for_path` lists the rules that
  apply to a file.
