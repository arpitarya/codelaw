# skills-consumer

A consumer that installs codelaw **and the skills package** (W-77): npm's
`@codelaw/skills` or PyPI's `codelaw-skills`, at codelaw's exact version. Its
`codelaw.toml` names every agent and a fixed launcher, so the agent files are
the same whichever registry they came from.

`scripts/smoke.sh` copies it beside its npm install and its PyPI install of the
staged release, runs `codelaw sync-agents` in each, and requires the two trees
to be byte-identical (L6) and `sync-agents --check` to pass. Locally, without
a release:

```bash
python3 scripts/package.py stage-skills /tmp/skills    # the package's folder
CODELAW_SKILLS=/tmp/skills/skills codelaw sync-agents     # run in a copy of this folder
```

`package.json` and `pyproject.toml` are what a team writes; nothing here is
installed in the repo.
