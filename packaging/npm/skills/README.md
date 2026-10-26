# codelaw skills

Every file [codelaw](https://github.com/arpitarya/codelaw) writes for a coding
agent — the skills, the steering and instruction files, the custom agent, the
hook and MCP registrations, the trigger evals — in one package, released with
codelaw at the same version.

| registry | install |
|---|---|
| npm | `npm i -D @codelaw/skills` |
| PyPI | `pip install codelaw-skills` or `uv add --dev codelaw-skills` |

It depends on `codelaw` at exactly its own version, so one install brings the
engine the skills were written for. The codelaw binary carries none of these
files: `codelaw init --agents` and `codelaw sync-agents` read them from this
package, and refuse (exit 2, with the install line) when it is missing or is
another version.

```bash
codelaw init --agents claude,copilot,kiro   # or add [agents] targets to codelaw.toml
codelaw sync-agents                         # rewrite the agent files from this package and your rules
```

codelaw looks for it in `node_modules/@codelaw/skills` from the project upwards,
then `codelaw_skills/` in the project's Python environment (`$VIRTUAL_ENV`, else
`.venv/`), then beside the codelaw binary itself (a `pipx` or `uvx` install).
`CODELAW_SKILLS=<folder>` names another place.

The files are `templates/agents/` of the codelaw repository, byte for byte, plus
`codelaw-skills.json`, which names the version. MIT licensed.
