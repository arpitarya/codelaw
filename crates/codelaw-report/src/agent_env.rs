//! Which format `check` and `fix` print when `--format` is not given
//! (SR-CLI-MCP decision 4a): `CODELAW_FORMAT`, else `agent` when a coding
//! agent's own environment variable says it ran the command, else `text`.
//! The environment is an input like any flag (L5): the same variables give
//! the same bytes.

use crate::Format;

/// The variables an agent sets in the commands it runs, in the order they
/// are looked at: the variable, the agent, and whether it counts only when
/// stdout is not a terminal. Each is named by that agent's own documentation
/// or source; a variable no such source names is not here.
pub const AGENT_VARS: &[(&str, &str, bool)] = &[
    // Claude Code: set in the Bash and PowerShell tools and hook commands;
    // IDE extensions do not set it (code.claude.com/docs/en/env-vars).
    ("CLAUDE_CODE_CHILD_SESSION", "Claude Code", false),
    // Claude Code: set in every subprocess it spawns, and also in the
    // integrated terminals of its IDE extensions, where a person types, so
    // it counts only when stdout is not a terminal.
    ("CLAUDECODE", "Claude Code", true),
    // Codex: injected into a shell tool call's environment
    // (codex-rs/core/src/exec_env.rs, spawn.rs).
    ("CODEX_THREAD_ID", "Codex", false),
    ("CODEX_SANDBOX", "Codex", false),
    ("CODEX_SANDBOX_NETWORK_DISABLED", "Codex", false),
    // Gemini CLI: set in the shell tool's subprocess
    // (geminicli.com/docs/tools/shell).
    ("GEMINI_CLI", "Gemini CLI", false),
];

/// What the format default is taken from: captured once from the process,
/// or sent by a warm process's client (SR-ARCHITECTURE decision 21).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FormatEnv {
    /// `CODELAW_FORMAT`.
    pub format: Option<String>,
    /// Each variable of [`AGENT_VARS`] that is set, with its value.
    pub agent_vars: Vec<(String, String)>,
    /// stdout is a terminal.
    pub tty: bool,
}

impl FormatEnv {
    /// The format inputs a variable lookup gives.
    pub fn from_lookup(var: impl Fn(&str) -> Option<String>, tty: bool) -> FormatEnv {
        let set = |k: &str| var(k).filter(|v| !v.is_empty());
        FormatEnv {
            format: set("CODELAW_FORMAT"),
            agent_vars: AGENT_VARS.iter().filter_map(|(k, _, _)| set(k).map(|v| ((*k).to_owned(), v))).collect(),
            tty,
        }
    }

    /// This process's.
    pub fn from_env(tty: bool) -> FormatEnv {
        FormatEnv::from_lookup(|k| std::env::var(k).ok(), tty)
    }

    /// The agent that ran this command, and the variable that says so:
    /// `CLAUDECODE=1 (Claude Code)`. `0` and `false` count as unset.
    pub fn agent(&self) -> Option<String> {
        AGENT_VARS.iter().find_map(|(k, agent, needs_pipe)| {
            let (_, v) = self.agent_vars.iter().find(|(name, _)| name == k)?;
            let off = v == "0" || v.eq_ignore_ascii_case("false");
            (!off && !(*needs_pipe && self.tty)).then(|| format!("{k}={v} ({agent})"))
        })
    }
}

/// The format `check` and `fix` print, and where it came from: `--format`,
/// `CODELAW_FORMAT`, the agent variable, or `default`. `CI` alone never
/// picks `agent`: it is not one of [`AGENT_VARS`].
pub fn resolve_format(flag: Option<&str>, env: &FormatEnv) -> Result<(Format, String), String> {
    const ALL: &str = "text, json, sarif, agent, github, gitlab, junit, checkstyle or bitbucket";
    if let Some(s) = flag {
        return Format::parse(s).map(|f| (f, "--format".to_owned())).ok_or_else(|| format!("unknown --format `{s}` ({ALL})"));
    }
    if let Some(s) = env.format.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        return Format::parse(s).map(|f| (f, "CODELAW_FORMAT".to_owned())).ok_or_else(|| format!("unknown CODELAW_FORMAT `{s}` ({ALL})"));
    }
    Ok(match env.agent() {
        Some(why) => (Format::Agent, why),
        None => (Format::Text, "default".to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(vars: &[(&str, &str)], tty: bool) -> FormatEnv {
        FormatEnv::from_lookup(|k| vars.iter().find(|(n, _)| *n == k).map(|(_, v)| (*v).to_owned()), tty)
    }

    fn pick(flag: Option<&str>, vars: &[(&str, &str)], tty: bool) -> (Format, String) {
        resolve_format(flag, &env(vars, tty)).unwrap()
    }

    #[test]
    fn every_agent_variable_picks_agent() {
        for (k, agent, _) in AGENT_VARS {
            assert_eq!(pick(None, &[(k, "1")], false), (Format::Agent, format!("{k}=1 ({agent})")));
        }
    }

    #[test]
    fn a_flag_wins_then_codelaw_format() {
        assert_eq!(pick(Some("json"), &[("CLAUDECODE", "1"), ("CODELAW_FORMAT", "sarif")], false).0, Format::Json);
        assert_eq!(pick(None, &[("CLAUDECODE", "1"), ("CODELAW_FORMAT", "text")], false), (Format::Text, "CODELAW_FORMAT".into()));
        assert!(resolve_format(None, &env(&[("CODELAW_FORMAT", "xml")], false)).unwrap_err().contains("CODELAW_FORMAT `xml`"));
    }

    #[test]
    fn ci_and_a_terminal_are_not_an_agent() {
        let ci = [("CI", "true"), ("GITHUB_ACTIONS", "true"), ("GITLAB_CI", "true"), ("TERM_PROGRAM", "vscode")];
        assert_eq!(pick(None, &ci, false), (Format::Text, "default".into()));
        assert_eq!(pick(None, &[], true), (Format::Text, "default".into()));
        // An IDE extension's terminal carries CLAUDECODE; a person reads it.
        assert_eq!(pick(None, &[("CLAUDECODE", "1")], true).0, Format::Text);
        assert_eq!(pick(None, &[("CLAUDECODE", "1"), ("CLAUDE_CODE_CHILD_SESSION", "1")], true).0, Format::Agent);
        assert_eq!(pick(None, &[("GEMINI_CLI", "1")], true).0, Format::Agent);
        // Empty, `0` and `false` are unset.
        for v in ["", "0", "false"] {
            assert_eq!(pick(None, &[("CLAUDECODE", v)], false).0, Format::Text);
        }
    }
}
