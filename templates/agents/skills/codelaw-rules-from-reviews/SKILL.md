---
name: codelaw-rules-from-reviews
description: Turn the points reviewers keep repeating on merged pull requests into codelaw rules or written guidance, on GitHub, GitLab, Bitbucket or Azure DevOps. Use when the user says "turn our review comments into rules", "what do reviewers keep telling us", "mine our PR reviews for conventions", "make rules from code review feedback", or a reviewer wrote "codelaw: rule" on a comment. Reads comments through the platform's own CLI (gh, glab, curl, az), clusters repeats, proposes each with evidence, and builds approved rules as preview rules with a baseline. The user approves every step.
---

# codelaw-rules-from-reviews — what reviewers repeat becomes a rule, with approval

Reviewers who type the same comment for the third time are describing a
rule nobody wrote down. This skill finds those comments, shows the user the
evidence, and — **only for what the user approves** — writes the rule
through `codelaw-author-rule`.

**You fetch; codelaw never does.** No `codelaw` command reads the network.
Every request below is a platform CLI (`gh`, `glab`, `az`) or `curl` that
**you** run. **Stop and ask before each numbered step's output is acted on**;
nothing turns on by itself.

## 1. Settings

Run `codelaw config --print` and read the `reviews.*` lines (they live in
`[reviews]` of the root `codelaw.toml`, `[tool.codelaw.reviews]` or
`"codelaw": {"reviews": …}`; codelaw has already validated them):

| key | default | meaning |
|---|---|---|
| `platform` | from the remote | `github`, `gitlab`, `bitbucket`, `bitbucket-datacenter`, `azure` |
| `url` | none | base URL of a self-hosted instance |
| `min_repeats` | `3` | merged PRs that must carry the same point |
| `min_reviewers` | `1` | different reviewers who made it |
| `window` | `"6m"` | how far back: `<n>d`, `<n>w`, `<n>m` (months), `<n>y` |

The user may override any of them for one run ("look at the last year");
say which values you used in every report.

## 2. Which platform

`git remote get-url origin` (or the remote the user names), then by host:

- `github.com`, or `[reviews] platform = "github"` → **GitHub** (`gh`; for
  Enterprise Server set `GH_HOST=<host of url>`).
- `gitlab.com`, or `gitlab` → **GitLab** (`glab`; self-managed:
  `GITLAB_HOST=<url>`).
- `bitbucket.org` → **Bitbucket Cloud**; `bitbucket-datacenter` → **Bitbucket
  Data Center** at `url` (both `curl`).
- `dev.azure.com`, `*.visualstudio.com`, or `azure` → **Azure DevOps** (`az`
  with the `azure-devops` extension; Server: `--organization <url>`).
- Anything else with no `[reviews] platform`: ask the user which platform it
  is, and suggest setting `[reviews] platform` and `url`.

**Credentials.** Check the login first (`gh auth status`, `glab auth status`,
`az account show`); for `curl`, a token in the environment
(`BITBUCKET_TOKEN`, and `BITBUCKET_USER` for Cloud app passwords). If none
works, tell the user how to log in and stop. Refer to a token only as
`$NAME` in the command; never print it, never write it to a file.

## 3. Collect — one shape for every platform

Work in a temporary folder **outside the repository**
(`dir=$(mktemp -d)`); delete it when the run ends. Compute the window's start
date (`since`, ISO `YYYY-MM-DD`) from `window`. Write one JSON object per
comment to `$dir/comments.jsonl`:

```json
{"comment": "…", "file": "src/a.ts", "line": 12, "hunk": "@@ … @@\n…",
 "reviewer": "login", "pr": "https://…/pull/41", "date": "2026-07-02",
 "author": "pr author login"}
```

`file`, `line` and `hunk` are `null` for a comment on the whole PR. Page
through every list; a flag a CLI refuses → check its `--help`; the shape is
what matters, not the exact call.

- **GitHub:**
  `gh pr list --state merged --search "merged:>=$since" --limit 1000 --json number,url,author`;
  per PR, `gh api --paginate repos/{owner}/{repo}/pulls/<n>/comments`
  (`body`, `path`, `line` or `original_line`, `diff_hunk`, `user.login`,
  `html_url`, `created_at`) and `gh api --paginate repos/{owner}/{repo}/pulls/<n>/reviews`
  (review bodies, no file).
- **GitLab:**
  `glab api --paginate "projects/:id/merge_requests?state=merged&updated_after=$since&per_page=100"`,
  keeping those with `merged_at >= since`; per MR,
  `glab api --paginate "projects/:id/merge_requests/<iid>/discussions"` —
  each note's `body`, `position.new_path`, `position.new_line`,
  `author.username`, `created_at`; skip `system` notes. No hunk is returned:
  build it locally (below) from `position.head_sha`.
- **Bitbucket Cloud:**
  `curl -sf -u "$BITBUCKET_USER:$BITBUCKET_TOKEN" "https://api.bitbucket.org/2.0/repositories/<workspace>/<repo>/pullrequests?state=MERGED&pagelen=50"`
  (follow `next`; stop past the window by `updated_on`), then
  `…/pullrequests/<id>/comments?pagelen=100` — `content.raw`, `inline.path`,
  `inline.to`, `user.nickname`, `links.html.href`, `created_on`; skip
  `deleted`.
- **Bitbucket Data Center:**
  `curl -sf -H "Authorization: Bearer $BITBUCKET_TOKEN" "<url>/rest/api/1.0/projects/<KEY>/repos/<slug>/pull-requests?state=MERGED&limit=100&start=<n>"`
  (follow `nextPageStart`), then `…/pull-requests/<id>/activities?limit=100` —
  the `COMMENTED` ones: `comment.text`, `commentAnchor.path`,
  `commentAnchor.line`, `diff` (the hunk), `user.slug`, `createdDate`
  (milliseconds).
- **Azure DevOps:**
  `az repos pr list --status completed --top 1000 -o json` (keep
  `closedDate >= since`); per PR,
  `az devops invoke --area git --resource pullRequestThreads --route-parameters project=<project> repositoryId=<repo> pullRequestId=<id> --api-version 7.1 -o json`
  — each thread's `threadContext.filePath`, `threadContext.rightFileStart.line`,
  and its comments' `content`, `author.uniqueName`, `publishedDate`; skip
  `commentType: "system"`. Build the hunk locally.

**A hunk built locally:** `git show <commit>:<file>` lines `line-3` to
`line+3` (fetch the commit with `git fetch origin <commit>` if it is
missing; if it cannot be had, leave `hunk` null).

**Drop:** comments by bots (`[bot]` logins, service accounts), comments by
the PR's own author, empty ones and pure acknowledgements ("LGTM",
"thanks", "done", "+1").

## 4. Cluster

Read the comments and group those that make **the same point** ("use
`const`", "don't import from `internal/`", "every handler logs the request
id"), however worded. For each group count:

- **repeats** — the distinct merged PRs carrying it (several comments in one
  PR count once);
- **reviewers** — the distinct reviewers who made it.

Keep a group when repeats ≥ `min_repeats` **and** reviewers ≥
`min_reviewers`. A comment carrying `codelaw: rule` on a line of its own is
**fast-tracked**: its point is kept whatever the counts, marked as such.

Before proposing, check each point against what codelaw already has:
`codelaw rules` and `explain_rule` (MCP) — a rule that exists but is off
becomes "switch on `<id>`", not a new rule.

## 5. Propose — stop for approval

Show one entry per kept group, strongest first:

> **1. Don't import from another package's `internal/`** — 5 PRs, 2
> reviewers, can be a rule
> - evidence: links to 3 of the PRs, and one or two sample hunks
> - proposal: a new rule `team/no-internal-import`, fix `manual` | switch on
>   `<id>` | guidance only

Grade each one:

- **can be a rule** — codelaw can check it exactly: a syntax-tree matcher,
  a project rule, a config value, or an existing rule switched on;
- **guidance only** — it needs judgement ("name things clearly"); propose
  the line for the team's own part of `CLAUDE.md` / `AGENTS.md`, **outside**
  the `<!-- codelaw:begin -->` block, never inside a generated file.

Then **ask the user** to approve, edit or reject each entry. Nothing below
happens for an entry they did not approve.

## 6. Author each approved rule

Follow `codelaw-author-rule` (or hand it to the `codelaw-rule-author`
subagent) with the evidence as input: the point, in one sentence, is the
convention; the commented hunks are the first Don't examples; the code as
merged after the comment, where it changed, the first Do. Pick the fix class
honestly (`auto` only where provably safe). The `**Why:**` may link the PRs;
it **names no reviewer**. Leave `stability: preview`.

Run `codelaw test <id>` until green, then `codelaw check --rule <id>` on the
repo, and show the user the rule, the test output and the count of existing
violations. **Ask before landing.**

## 7. Land with a baseline

On approval: `codelaw baseline add --rule <id> --reason "from review comments: <point>"`
freezes today's violations, so new code obeys from day one and the old ones
burn down. Approved guidance lines go into the files the user agreed. Then
hand back: the rule and cases files, the baseline entries, the guidance
lines, and the run's settings (platform, window, thresholds) — for the user
to commit in a pull request of their own.

## Never

- **Never expect a `codelaw` command to fetch**, and never add one that
  does; the platform CLIs and `curl` are yours to run.
- **Never write a token** into a file, the config, a script, a commit, a log
  or the chat; refer to it as `$NAME`.
- **Never turn a rule on, write a rule or baseline it without the user's
  approval of that step.**
- **Never post, resolve, approve or edit anything on the platform**; this
  skill only reads.
- **Never commit the collected comments** or leave them in the repository;
  delete the temporary folder.
- **Never name a reviewer in a rule file**, and never present a point below
  the thresholds as meeting them.
- **Never mark a rule `stable` or claim it works** without a green
  `codelaw test`.
