//! `codelaw init --ci github|gitlab --autofix daily|weekly [--unsafe]`
//! (SR-CLI-MCP decision 12, W-51): write the scheduled auto-fix workflow into
//! the consumer's repo; `codelaw init --ci github` alone (decision 12a, W-97):
//! the check workflow that runs codelaw's GitHub Action at this version. The
//! text is a template under `templates/ci/` (L10); this code fills the slots
//! (`cron`, `version`, `suggest_job`) and writes.

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

/// The check workflow (decision 12a): GitHub only, since the Action is GitHub's.
pub fn check_workflow(provider: Provider, version: &str) -> Result<(PathBuf, String), String> {
    match provider {
        Provider::Github => Ok((PathBuf::from(".github/workflows/codelaw.yml"), render("ci/github/codelaw.yml", &[("version", version)])?)),
        Provider::Gitlab => Err("--ci gitlab needs --autofix: codelaw's check workflow is a GitHub Action".into()),
    }
}

/// Write `text` at `rel`, unless a file there does not name `marker` (it is
/// not ours). `Ok(None)`: the same text is already there.
fn write_ours(root: &Path, rel: &Path, text: &str, marker: &str) -> Result<Option<String>, String> {
    let path = root.join(rel);
    let shown = rel.display().to_string();
    match std::fs::read_to_string(&path) {
        Ok(current) if current == text => return Ok(None),
        Ok(current) if !current.contains(marker) => {
            return Err(format!("{shown} exists and is not codelaw's; move it aside first"));
        }
        _ => {}
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(Some(shown))
}

/// Write the check workflow; never overwrites a different file already there.
pub fn install_check(root: &Path, provider: Provider) -> Result<Vec<String>, String> {
    let (rel, text) = check_workflow(provider, codelaw_core::VERSION)?;
    match write_ours(root, &rel, &text, "templates/ci/github/codelaw.yml")? {
        None => Ok(vec![format!("{} already up to date", rel.display())]),
        Some(shown) => Ok(vec![
            format!("wrote {shown}"),
            format!("it runs `uses: arpitarya/codelaw@v{}`; `sarif: true` also uploads to code scanning (needs `security-events: write`)", codelaw_core::VERSION),
        ]),
    }
}

/// Write the workflow; never overwrites a different file already there.
pub fn install(root: &Path, provider: Provider, schedule: Schedule, suggest: bool) -> Result<Vec<String>, String> {
    let (rel, text) = workflow(provider, schedule, suggest, codelaw_core::VERSION)?;
    let mut lines = Vec::new();
    match write_ours(root, &rel, &text, "codelaw-autofix.yml")? {
        None => return Ok(vec![format!("{} already up to date", rel.display())]),
        Some(shown) => lines.push(format!("wrote {shown}")),
    }
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
        let (path, t) = check_workflow(Provider::Github, "1.2.3").unwrap();
        assert_eq!(path, PathBuf::from(".github/workflows/codelaw.yml"));
        assert!(t.contains("uses: arpitarya/codelaw@v1.2.3") && !t.contains("{{"));
        assert!(check_workflow(Provider::Gitlab, "1.2.3").is_err());
    }
}
