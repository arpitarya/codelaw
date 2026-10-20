//! The agent surfaces (SR-AGENT-SURFACES): `codelaw hook`, which checks and
//! fixes the files an agent just edited, and `codelaw sync-agents`, which
//! writes every agent's files from one source — the skills in `agents/`
//! and the project's rules. Like every surface it only translates; the
//! verdicts come from `codelaw-engine` (SR-ARCHITECTURE decision 2).

pub mod hook;
pub mod skills;
pub mod sync;

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

/// The `[agents]` section (SR-CONSUMER-LAYOUT decision 5; SR-CONFIG
/// decision 5: strict, an unknown key is an error).
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
        let Some(table) = &file.raw.agents else { return Ok(AgentsConfig::default()) };
        let at = codelaw_core::normalize_path(&file.path);
        let mut out = AgentsConfig::default();
        for (key, value) in table {
            match key.as_str() {
                "targets" => {
                    let names = value
                        .as_array()
                        .and_then(|a| a.iter().map(|v| v.as_str().map(str::to_owned)).collect::<Option<Vec<_>>>())
                        .ok_or_else(|| format!("{at}: `[agents] targets` is a list of agent names"))?;
                    out.targets = parse_targets(&names).map_err(|e| format!("{at}: [agents] targets: {e}"))?;
                }
                "command" => {
                    let c = value.as_str().filter(|s| !s.trim().is_empty()).ok_or_else(|| format!("{at}: `[agents] command` is a non-empty string"))?;
                    out.command = Some(c.to_owned());
                }
                other => {
                    let near = codelaw_config::did_you_mean(other, ["targets", "command"]).map(|n| format!("; did you mean `{n}`?")).unwrap_or_default();
                    return Err(format!("{at}: unknown key `{other}` in [agents]{near}"));
                }
            }
        }
        Ok(out)
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
