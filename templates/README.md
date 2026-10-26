# templates/ — every file codelaw writes into a consumer's repo

The registry of [SR-LAW-10](../records/0012_LAW-10-consumer-templates.md) (L10):
**one row per template**, laid out as `templates/<area>/<path as it lands>`. Code computes
values and fills a template's `{{name}}` slots; no consumer text lives in code. To change
what consumers receive, edit the template here — the diff is the change.

- **A template is used byte for byte.** `{{name}}` is a slot, `\{{` writes a literal `{{`,
  and there is nothing else: no conditions, no loops (an optional section is its own
  template, rendered into a slot). A file whose last line ends in a slot has no final newline.
- **Embedded at build time** (`crates/codelaw-core/src/template.rs`), so both packages are
  complete and identical and nothing is read from disk at run time.
- **Adding a template** is three edits: the file, one `t!("<path>")` line in `TEMPLATES`, one
  row below. `cargo test -p codelaw-core` fails until all three agree, and until its fixture
  fills exactly the slots it has.
- **Not here:** data codelaw serialises from state (baseline, deprecations, the deps snapshot,
  the cache), fixed source, reports, and a pack rule copied by `rules eject`
  (SR-LAW-10 decision 2).

| template | lands at | written by | record |
|---|---|---|---|
| `agents/agents/codelaw-rule-author.md` | .claude/agents/codelaw-rule-author.md, .github/agents/codelaw-rule-author.agent.md | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/always-on.md` | the body of the `CLAUDE.md` and `AGENTS.md` blocks and of `.kiro/steering/codelaw.md` | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/block.md` | the marker block in `CLAUDE.md` and `AGENTS.md` (the merge stays in code) | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/claude/hook-group.json` | the `PostToolUse` entry of `.claude/settings.json` and the Cowork `hooks.json` | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/codex/config.toml` | printed, not written: what to add to `~/.codex/config.toml` | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/copilot/hook.json` | .github/hooks/codelaw.json | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/copilot/instructions.md` | .github/instructions/codelaw-<language>.instructions.md | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/copilot/mcp-server.json` | the `codelaw` entry of `.vscode/mcp.json` | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/cowork/hooks.json` | .claude/plugins/codelaw/hooks/hooks.json | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/cowork/mcp.json` | .claude/plugins/codelaw/.mcp.json | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/cowork/plugin.json` | .claude/plugins/codelaw/.claude-plugin/plugin.json | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/evals/triggers.json` | not written: the trigger evals of every skill (decision 5) | tests only | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/kiro/hook.json` | .kiro/hooks/codelaw.json | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/kiro/steering-language.md` | .kiro/steering/codelaw-<language>.md | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/kiro/steering.md` | .kiro/steering/codelaw.md | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/mcp-server.json` | the `codelaw` entry of `.mcp.json`, `.kiro/settings/mcp.json` and the Cowork `.mcp.json` | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/note-block.md` | the generated note inside a marker block, a steering or an instructions file | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/note-skill.md` | the generated note after a skill's or the agent's frontmatter | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/path-scoped-rule.md` | one rule line in a per-language instructions file | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/path-scoped.md` | the body of a per-language instructions file | `sync-agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-author-rule/SKILL.md` | `<agent>/skills/codelaw-author-rule/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-ci-triage/SKILL.md` | `<agent>/skills/codelaw-ci-triage/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-debug-rule/SKILL.md` | `<agent>/skills/codelaw-debug-rule/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-explain/SKILL.md` | `<agent>/skills/codelaw-explain/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-fix/SKILL.md` | `<agent>/skills/codelaw-fix/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-migrate/SKILL.md` | `<agent>/skills/codelaw-migrate/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-pack/SKILL.md` | `<agent>/skills/codelaw-pack/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-review/SKILL.md` | `<agent>/skills/codelaw-review/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-setup/SKILL.md` | `<agent>/skills/codelaw-setup/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-suppress/SKILL.md` | `<agent>/skills/codelaw-suppress/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-upgrade/SKILL.md` | `<agent>/skills/codelaw-upgrade/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `agents/skills/codelaw-write/SKILL.md` | `<agent>/skills/codelaw-write/SKILL.md` — .claude/skills, .github/skills, .agents/skills, .kiro/skills, the Cowork plugin | `sync-agents`, `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| `ci/github/autofix-suggest-job.yml` | rendered into the slot `suggest_job` of `ci/github/codelaw-autofix.yml` with `--unsafe` | `init --ci <provider> --autofix` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 12 |
| `ci/github/codelaw-autofix.yml` | .github/workflows/codelaw-autofix.yml | `init --ci <provider> --autofix` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 12 |
| `ci/gitlab/autofix-suggest-job.yml` | rendered into the slot `suggest_job` of `ci/gitlab/codelaw-autofix.yml` with `--unsafe` | `init --ci <provider> --autofix` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 12 |
| `ci/gitlab/codelaw-autofix.yml` | .gitlab/codelaw-autofix.yml | `init --ci <provider> --autofix` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 12 |
| `git-hooks/post-commit` | the git hooks folder's `post-commit` | `init --git-hooks` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `git-hooks/pre-commit` | the git hooks folder's `pre-commit` | `init --git-hooks` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `init/.codelaw/rules/README.md` | .codelaw/rules/README.md | `init` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `init/codelaw-agents.toml` | the `[agents]` section of `codelaw.toml` | `init --agents` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `init/codelaw.toml` | codelaw.toml | `init` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `init/gitignore` | the line `init` adds to `.gitignore` (the merge stays in code) | `init` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/example.md` | .codelaw/rules/<namespace>/<name>.md — one language's Don't/Do pair | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/fix-line.md` | .codelaw/rules/<namespace>/<name>.md — the `fix:` line of the rule block | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/fix-reason.md` | .codelaw/rules/<namespace>/<name>.md — the `fix_reason:` line | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/how-to-fix.md` | .codelaw/rules/<namespace>/<name>.md — the `## How to fix` section of a `manual` rule | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/matcher-file.md` | .codelaw/rules/<namespace>/<name>.md — the `match:` block of a file rule | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/matcher-project.md` | .codelaw/rules/<namespace>/<name>.md — the `graph:` block of a project rule | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/rule.md` | .codelaw/rules/<namespace>/<name>.md — the rule file | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
| `new-rule/scope-project.md` | .codelaw/rules/<namespace>/<name>.md — the `scope: project` line | `new-rule` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) |
