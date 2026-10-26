//! `codelaw init --ci github|gitlab --autofix daily|weekly [--unsafe]`
//! (SR-CLI-MCP decision 12, W-51): write the scheduled auto-fix workflow into
//! the consumer's repo. The text is a template under `templates/ci/` (L10);
//! this code fills the slots (`cron`, `version`, `suggest_job`) and writes.

use std::path::{Path, PathBuf};

use codelaw_core::template::render;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    Github,
    Gitlab,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Daily,
    Weekly,
}

pub fn parse_provider(s: &str) -> Result<Provider, String> {
    match s {
        "github" => Ok(Provider::Github),
        "gitlab" => Ok(Provider::Gitlab),
        other => Err(format!("--ci is `github` or `gitlab`, not `{other}`")),
    }
}

pub fn parse_schedule(s: &str) -> Result<Schedule, String> {
    match s {
        "daily" => Ok(Schedule::Daily),
        "weekly" => Ok(Schedule::Weekly),
        other => Err(format!("--autofix is `daily` or `weekly`, not `{other}`")),
    }
}

impl Schedule {
    fn cron(self) -> &'static str {
        match self {
            Schedule::Daily => "17 4 * * *",
            Schedule::Weekly => "17 4 * * 1",
        }
    }
}

/// The workflow text and the path it lands at, relative to the repo root.
pub fn workflow(provider: Provider, schedule: Schedule, suggest: bool, version: &str) -> Result<(PathBuf, String), String> {
    let (main, extra, path) = match provider {
        Provider::Github => ("ci/github/codelaw-autofix.yml", "ci/github/autofix-suggest-job.yml", ".github/workflows/codelaw-autofix.yml"),
        Provider::Gitlab => ("ci/gitlab/codelaw-autofix.yml", "ci/gitlab/autofix-suggest-job.yml", ".gitlab/codelaw-autofix.yml"),
    };
    let suggest_job = if suggest { render(extra, &[("version", version)])? } else { String::new() };
    let text = render(main, &[("cron", schedule.cron()), ("version", version), ("suggest_job", &suggest_job)])?;
    Ok((PathBuf::from(path), text))
}

/// Write the workflow; never overwrites a different file already there.
pub fn install(root: &Path, provider: Provider, schedule: Schedule, suggest: bool) -> Result<Vec<String>, String> {
    let (rel, text) = workflow(provider, schedule, suggest, codelaw_core::VERSION)?;
    let path = root.join(&rel);
    let shown = rel.display().to_string();
    let mut lines = Vec::new();
    match std::fs::read_to_string(&path) {
        Ok(current) if current == text => return Ok(vec![format!("{shown} already up to date")]),
        Ok(current) if !current.contains("codelaw-autofix.yml") => {
            return Err(format!("{shown} exists and is not codelaw's; move it aside first"));
        }
        _ => {}
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, &text).map_err(|e| e.to_string())?;
    lines.push(format!("wrote {shown}"));
    match provider {
        Provider::Github => {
            lines.push("the pull request's own CI needs a token that can trigger workflows: add the secret CODELAW_AUTOFIX_TOKEN (a GitHub App token or a fine-grained PAT), or start CI by hand".into());
            lines.push("exclude the branch `codelaw/autofix` from branch protection's force-push rule".into());
        }
        Provider::Gitlab => {
            lines.push(format!("add `include: \".gitlab/codelaw-autofix.yml\"` to .gitlab-ci.yml, create a pipeline schedule with the cron `{}`, and set the masked variable CODELAW_AUTOFIX_TOKEN (api and write_repository scopes)", schedule.cron()));
        }
    }
    if suggest {
        lines.push("--unsafe: the `suggest` fixes go to the branch `codelaw/autofix-suggest`, in their own PR".into());
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_workflow_leaves_no_unfilled_slot() {
        for p in [Provider::Github, Provider::Gitlab] {
            for s in [Schedule::Daily, Schedule::Weekly] {
                for u in [false, true] {
                    let (_, t) = workflow(p, s, u, "1.2.3").unwrap();
                    assert!(!t.contains("{{") || t.contains("${{"), "{p:?}");
                    assert!(t.contains("codelaw@1.2.3"));
                    assert_eq!(t.contains("autofix-suggest"), u);
                }
            }
        }
    }
}
