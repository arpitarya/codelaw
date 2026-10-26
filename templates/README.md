---
type: Index
description: "The registry of consumer-file templates (SR-LAW-10): one row per template, its consumer path, the command that writes it, the record that owns that command."
---

# `templates/` — what codelaw writes into a consumer's repo

Text and `{{slot}}` only; `\{{` writes a literal `{{`. Rules: [SR-LAW-10](../records/0012_LAW-10-consumer-templates.md).

| template | lands at | written by | record |
|---|---|---|---|
| `ci/github/codelaw-autofix.yml` | `.github/workflows/codelaw-autofix.yml` | `init --ci github --autofix` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 12 |
| `ci/github/autofix-suggest-job.yml` | rendered into the slot `suggest_job` of the file above with `--unsafe` | same | same |
| `ci/gitlab/codelaw-autofix.yml` | `.gitlab/codelaw-autofix.yml` | `init --ci gitlab --autofix` | same |
| `ci/gitlab/autofix-suggest-job.yml` | rendered into the slot `suggest_job` of the file above with `--unsafe` | same | same |
