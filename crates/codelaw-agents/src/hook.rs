//! `codelaw hook` (SR-AGENT-SURFACES decision 2, the post-edit hook; L4
//! decision 8): the file an agent just wrote goes through `codelaw fix`
//! with `auto` fixes only — fix, format, verify (L8 `fix_one_file`) — and
//! what is left goes back to the agent as `codelaw-report`'s hook text.
//!
//! The verdict is `Project::fix`'s, the same call `codelaw fix` makes (L5
//! decision 5); this file only finds the paths and picks the exit code.
//! Exit `2` with the findings on stderr is how Claude Code and Codex feed a
//! `PostToolUse` hook's output back to the agent; `0` lets the edit stand.
//!
//! `--pre` is the pre-write mode, the Gate (decision 9): `crate::pre`.

use clap::Args;
use codelaw_core::Language;
use codelaw_engine::{EngineOptions, FixOptions, Project};
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Args, Clone, Debug, Default)]
pub struct HookArgs {
    /// The files the agent edited. When none, they are read from the hook
    /// payload the agent writes on stdin (Claude Code, Codex, Copilot).
    pub paths: Vec<PathBuf>,
    /// Judge the write the payload on stdin proposes, before it reaches
    /// disk: refuse it when it adds an error. Fails open.
    #[arg(long, conflicts_with = "paths")]
    pub pre: bool,
    /// With `--pre`: the agent whose payload this is and whose answer to
    /// print; read off the payload when not given.
    #[arg(long, value_enum, requires = "pre")]
    pub agent: Option<crate::pre::PreAgent>,
    /// With `--pre`: allow the write when no verdict comes within this
    /// many milliseconds.
    #[arg(long, default_value_t = 5000, value_name = "MS")]
    pub timeout_ms: u64,
    /// The profile to judge with; `ci` by default, so an agent gets CI's
    /// verdict (SR-CONFIG decision 10).
    #[arg(long)]
    pub profile: Option<String>,
}

/// The paths a hook payload names. Every agent of the decision-2 matrix
/// writes JSON on stdin; the edited file is in one of these places:
/// `tool_input.file_path` / `path` / `notebook_path` (Claude Code, Codex
/// edits), `tool_input.command` holding an `apply_patch` envelope (Codex),
/// or `toolArgs`, a JSON string with `path` / `file_path` (Copilot).
/// Relative paths resolve against the payload's `cwd`.
pub fn paths_from_payload(payload: &str) -> Vec<PathBuf> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) else { return Vec::new() };
    let mut names: Vec<String> = Vec::new();
    let mut from_args = |args: &serde_json::Value| {
        for key in ["file_path", "path", "notebook_path", "filePath"] {
            if let Some(s) = args.get(key).and_then(|x| x.as_str()) {
                names.push(s.to_owned());
            }
        }
        for key in ["command", "input", "patch"] {
            if let Some(s) = args.get(key).and_then(|x| x.as_str()) {
                names.extend(patch_paths(s));
            }
        }
    };
    if let Some(args) = v.get("tool_input") {
        from_args(args);
    }
    match v.get("toolArgs") {
        Some(serde_json::Value::String(s)) => {
            if let Ok(args) = serde_json::from_str::<serde_json::Value>(s) {
                from_args(&args);
            }
        }
        Some(args @ serde_json::Value::Object(_)) => from_args(args),
        _ => {}
    }
    let cwd = v.get("cwd").and_then(|x| x.as_str()).map(PathBuf::from);
    let mut out: Vec<PathBuf> = Vec::new();
    for n in names {
        let p = PathBuf::from(&n);
        let p = match (&cwd, p.is_absolute()) {
            (Some(c), false) => c.join(p),
            _ => p,
        };
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// The files an `apply_patch` envelope adds, updates or moves to.
fn patch_paths(patch: &str) -> Vec<String> {
    patch
        .lines()
        .filter_map(|l| l.strip_prefix("*** Add File: ").or_else(|| l.strip_prefix("*** Update File: ")).or_else(|| l.strip_prefix("*** Move to: ")))
        .map(|s| s.trim().to_owned())
        .collect()
}

/// The paths codelaw checks: existing files with an L3 extension. A deleted
/// file or a `.rs` file is not the hook's business, and is dropped silently.
fn checkable(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = paths.into_iter().filter(|p| p.is_file() && Language::from_path(p).is_some()).collect();
    out.dedup();
    out
}

/// Run the hook on `paths`, or on the payload when there are none. Returns
/// the text for the agent and whether it must go back to it.
pub fn run(paths: Vec<PathBuf>, payload: Option<&str>, profile: Option<String>) -> Result<Option<(String, bool)>, String> {
    let paths = if paths.is_empty() { payload.map(paths_from_payload).unwrap_or_default() } else { paths };
    let paths = checkable(paths);
    let Some(first) = paths.first() else { return Ok(None) };
    let start = first.parent().map(Path::to_path_buf).filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| PathBuf::from("."));
    // The files the agent just edited are touched: a grandfathered finding
    // in one gets the verdict CI would give it (SR-BASELINE decision 8).
    // The `ci` profile unless `--profile` or `CODELAW_PROFILE` says
    // otherwise: an agent never leaves behind what CI would fail.
    // A hook never waits for a cold type program (L8 decision 8).
    let options = EngineOptions { touched_paths: paths.clone(), profile, profile_surface: Some(("ci", "codelaw hook")), no_cold_types: true, ..EngineOptions::from_env() };
    let mut project = Project::load(&start, options).map_err(|e| e.to_string())?;
    // `auto` only, written: L4 decision 8. `suggest` goes back as a diff.
    let report = project.fix(&paths, &FixOptions { unsafe_fixes: false, dry_run: false, trace: false }).map_err(|e| e.to_string())?;
    let text = codelaw_report::hook_text(&report.files, &report.check);
    Ok(Some((text, codelaw_report::hook_blocks(&report.check))))
}

/// `codelaw hook`: exit `0` when nothing is left (the auto-fix notes on
/// stdout), `2` with what is left on stderr, `2` with `error:` on a config
/// or rule error — which also needs the agent's attention.
pub fn hook(args: HookArgs) -> ExitCode {
    let mut payload = String::new();
    if args.paths.is_empty() && !std::io::stdin().is_terminal() {
        let _ = std::io::stdin().read_to_string(&mut payload);
    }
    if args.pre {
        return crate::pre::hook_pre(payload, args.agent, args.profile, args.timeout_ms);
    }
    match run(args.paths, Some(&payload), args.profile) {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some((text, false))) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Ok(Some((text, true))) => {
            eprint!("{text}");
            ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_agent_payload_names_its_file() {
        let claude = r#"{"hook_event_name":"PostToolUse","cwd":"/r","tool_name":"Edit","tool_input":{"file_path":"/r/src/a.ts","old_string":"x"}}"#;
        assert_eq!(paths_from_payload(claude), [PathBuf::from("/r/src/a.ts")]);
        let relative = r#"{"cwd":"/r","tool_input":{"path":"src/b.py"}}"#;
        assert_eq!(paths_from_payload(relative), [PathBuf::from("/r/src/b.py")]);
        let codex = r#"{"cwd":"/r","tool_name":"apply_patch","tool_input":{"command":"*** Begin Patch\n*** Update File: a.md\n@@\n-x\n+y\n*** Add File: b.json\n+{}\n*** End Patch"}}"#;
        assert_eq!(paths_from_payload(codex), [PathBuf::from("/r/a.md"), PathBuf::from("/r/b.json")]);
        let copilot = r#"{"cwd":"/r","toolName":"edit","toolArgs":"{\"path\":\"c.css\"}"}"#;
        assert_eq!(paths_from_payload(copilot), [PathBuf::from("/r/c.css")]);
        assert!(paths_from_payload("not json").is_empty());
        assert!(paths_from_payload(r#"{"tool_input":{"command":"ls"}}"#).is_empty());
    }

    #[test]
    fn a_file_codelaw_does_not_check_is_left_alone() {
        assert_eq!(run(vec![PathBuf::from("Cargo.toml.missing"), PathBuf::from("src/lib.rs")], None, None), Ok(None));
    }
}
