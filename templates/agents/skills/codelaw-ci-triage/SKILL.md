---
name: codelaw-ci-triage
description: Triage a failed codelaw CI job. Use when a CI codelaw step failed, when the user pastes a codelaw log, a SARIF file or a GitHub code-scanning alert from codelaw, or asks "why is CI red on lint", "what broke the codelaw check". Reads the report, reproduces it locally with the same flags and version, then fixes it or routes it to codelaw-fix, codelaw-debug-rule or sync-agents.
---

# codelaw-ci-triage — reproduce, classify, route

CI and your machine must agree before anything is fixed: codelaw is
deterministic (the same files, config, rules and version give the same
result), so a difference means one of those four differs.

## 1. Read what CI said

From the log or SARIF, collect:

- the **command and flags** the job ran (for example
  `codelaw check --format sarif`, `codelaw sync-agents --check`);
- the **exit code**: `1` errors remain; `2` a config, rule or usage error, or
  a rule failed (`codelaw/internal-error`);
- the **codelaw version** and the rule-pack versions, if logged;
- each finding: file, line, rule id (SARIF `ruleId`, `locations`).

## 2. Reproduce locally

1. Same version: compare `codelaw --version` with CI's; compare the installed
   pack versions with the lockfile. If they differ, install what the
   lockfile says before anything else.
2. Same files: check out the commit CI ran on.
3. Same command and flags, with `--format json` added for detail.

If the local result differs from CI, report which of the four inputs differs;
do not fix code against a result you cannot reproduce.

## 3. Classify and route

| what failed | do |
|---|---|
| findings in changed code (exit 1) | the `codelaw-fix` skill on those files |
| `sync-agents --check` drift | run `codelaw sync-agents`, review the diff, commit it; never hand-edit the generated files |
| config or rule load error (exit 2) | read the error: it names the file, the line and usually the fix ("did you mean …?"); fix the config or local rule |
| `codelaw/internal-error` | a rule crashed on a file: report it with the rule id, file and codelaw version; it is a bug for the rule's owner, not something to work around |
| a finding that looks wrong | the `codelaw-debug-rule` skill |
| only on CI, not locally, same inputs | report it with both outputs; that is a determinism bug in codelaw |

## 4. Confirm

Re-run the exact CI command locally; it must exit `0` (or show only what the
user decided to keep). Summarise: cause, what you changed, the command that
now passes.

## Never

- **Never make CI green by changing the job** (removing the step, adding
  `|| true`, raising `--max-warnings`, pinning an older codelaw) unless the
  user decides that.
- **Never disable a rule or add suppressions to clear a CI failure.**
