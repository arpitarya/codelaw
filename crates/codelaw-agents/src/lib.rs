//! The agent surfaces (SR-AGENT-SURFACES): `codelaw hook`, which checks and
//! fixes the files an agent just edited, `codelaw hook --pre`, which judges
//! a write before it lands (the Gate), `codelaw brief`, what is specific to
//! the repo for the files an agent is about to write, and `codelaw sync-agents`, which
//! writes every agent's files from one source — the installed skills
//! package (`@codelaw/skills`, `codelaw-skills`: `templates/agents/` of the
//! same release, W-77) and the project's rules. Like every surface it only translates; the
//! verdicts come from `codelaw-engine` (SR-ARCHITECTURE decision 2).

pub mod brief;
pub mod hook;
pub mod package;
pub mod pre;
pub mod skills;
pub mod sync;

pub use brief::{BriefArgs, brief};
pub use hook::{HookArgs, hook};
pub use sync::{SyncArgs, sync_agents, write_for_init};

use std::path::Path;

/// The agents of the decision-2 matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Claude,
    Cowork,
    Copilot,
    Codex,
    Kiro,
}

impl Target {
    pub const ALL: [Target; 5] = [Target::Claude, Target::Cowork, Target::Copilot, Target::Codex, Target::Kiro];

    pub fn name(self) -> &'static str {
        match self {
            Target::Claude => "claude",
            Target::Cowork => "cowork",
            Target::Copilot => "copilot",
            Target::Codex => "codex",
            Target::Kiro => "kiro",
        }
    }

    pub fn parse(name: &str) -> Result<Target, String> {
        Target::ALL.into_iter().find(|t| t.name() == name).ok_or_else(|| {
            let names: Vec<&str> = Target::ALL.iter().map(|t| t.name()).collect();
            let near = codelaw_config::did_you_mean(name, names.iter().copied()).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
            format!("unknown agent `{name}` (one of {}){near}", names.join(", "))
        })
    }
}

/// Parse a list of agent names, deduplicated and in matrix order.
pub fn parse_targets(names: &[String]) -> Result<Vec<Target>, String> {
    let mut out = Vec::new();
    for n in names {
        let t = Target::parse(n.trim())?;
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out.sort();
    Ok(out)
}

/// The `[agents]` section (SR-AGENT-SURFACES decision 6; SR-CONFIG
/// decision 5: the loader refuses an unknown key, this refuses an unknown
/// agent, naming the nearest).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentsConfig {
    /// `targets`: the agents whose files `sync-agents` writes.
    pub targets: Vec<Target>,
    /// `command`: how hooks and MCP registrations start codelaw; detected
    /// from the repo when unset.
    pub command: Option<String>,
}

impl AgentsConfig {
    pub fn from_config(config: &codelaw_config::Config) -> Result<AgentsConfig, String> {
        let Some(file) = config.files.first() else { return Ok(AgentsConfig::default()) };
        let at = codelaw_core::normalize_path(&file.path);
        let raw = &file.raw.agents;
        let targets = parse_targets(raw.targets.as_deref().unwrap_or_default()).map_err(|e| format!("{at}: [agents] targets: {e}"))?;
        let command = match &raw.command {
            Some(c) if c.trim().is_empty() => return Err(format!("{at}: `[agents] command` is a non-empty string")),
            c => c.clone(),
        };
        Ok(AgentsConfig { targets, command })
    }
}

/// How an agent's hook or MCP client starts codelaw in this repo: the
/// `[agents] command`, else the repo's own install — `npx --no-install
/// codelaw` beside a `package.json`, `uv run codelaw` beside a `uv.lock`,
/// else `codelaw` on the PATH.
pub fn launcher(root: &Path, configured: Option<&str>) -> Vec<String> {
    if let Some(c) = configured {
        return c.split_whitespace().map(str::to_owned).collect();
    }
    let words: &[&str] = if root.join("package.json").is_file() {
        &["npx", "--no-install", "codelaw"]
    } else if root.join("uv.lock").is_file() {
        &["uv", "run", "codelaw"]
    } else {
        &["codelaw"]
    };
    words.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_parse_in_matrix_order_and_name_the_fix() {
        let t = parse_targets(&["kiro".into(), "claude".into(), "kiro".into()]).unwrap();
        assert_eq!(t, [Target::Claude, Target::Kiro]);
        let e = parse_targets(&["claud".into()]).unwrap_err();
        assert!(e.contains("did you mean `claude`?"), "{e}");
    }
}
