//! SR-CLI-MCP decision 4a through the binary: inside a coding agent `check`
//! and `fix` print the agent format with no flag; `--format` always wins,
//! `CODELAW_FORMAT` wins over the detection, and CI is not an agent.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-agent-format-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::copy(repo().join("fixtures/l3/rules/fixture/py-no-print.md"), d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    fs::write(d.join("codelaw.toml"), "").unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    d
}

/// The binary with none of the format inputs of the test's own
/// environment (this repo's tests often run inside an agent), plus `env`.
fn codelaw(cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.args(args).current_dir(cwd).env_remove("CODELAW_FORMAT").env_remove("CODELAW_CACHE_DIR").env_remove("CODELAW_PROFILE").env_remove("CI").env("NO_COLOR", "1");
    for (k, _, _) in codelaw_report::AGENT_VARS {
        c.env_remove(k);
    }
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn an_agent_gets_the_agent_format_and_a_flag_wins() {
    let d = temp("check");
    let agent = codelaw(&d, &["check", "--format", "agent"], &[]);
    let text = codelaw(&d, &["check", "--format", "text"], &[]);
    assert_ne!(agent, text, "the two formats differ on a finding");

    // DoD: CLAUDECODE=1 and no --format → agent; --format json → JSON;
    // neither → text.
    assert_eq!(codelaw(&d, &["check"], &[("CLAUDECODE", "1")]), agent);
    let json = codelaw(&d, &["check", "--format", "json"], &[("CLAUDECODE", "1")]);
    assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok(), "{json}");
    assert_eq!(codelaw(&d, &["check"], &[]), text);

    // Every variable of the list, on `check` and `fix --dry-run`.
    let fix_agent = codelaw(&d, &["fix", "--dry-run", "--format", "agent"], &[]);
    for (k, _, _) in codelaw_report::AGENT_VARS {
        assert_eq!(codelaw(&d, &["check"], &[(k, "1")]), agent, "{k}");
        assert_eq!(codelaw(&d, &["fix", "--dry-run"], &[(k, "1")]), fix_agent, "{k}");
    }

    // CODELAW_FORMAT wins over the detection; the flag over both.
    assert_eq!(codelaw(&d, &["check"], &[("CLAUDECODE", "1"), ("CODELAW_FORMAT", "text")]), text);
    assert_eq!(codelaw(&d, &["check"], &[("CODELAW_FORMAT", "agent")]), agent);
    assert_eq!(codelaw(&d, &["check", "--format", "text"], &[("CODELAW_FORMAT", "agent"), ("GEMINI_CLI", "1")]), text);
}

#[test]
fn ci_is_not_an_agent() {
    let d = temp("ci");
    let text = codelaw(&d, &["check", "--format", "text"], &[("CI", "true")]);
    let ci = [("CI", "true"), ("GITHUB_ACTIONS", "true"), ("GITLAB_CI", "true"), ("TERM_PROGRAM", "vscode")];
    assert_eq!(codelaw(&d, &["check"], &ci), text);
    // `0` and `false` switch an agent variable off.
    assert_eq!(codelaw(&d, &["check"], &[("CI", "true"), ("CLAUDECODE", "0")]), text);
}

#[test]
fn config_print_says_which_format_applies_and_why() {
    let d = temp("config");
    let values = |env: &[(&str, &str)]| -> serde_json::Value {
        let out = codelaw(&d, &["config", "--print", "--format", "json"], env);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
        v["values"].as_array().unwrap().iter().find(|x| x["key"] == "format").cloned().expect("a format value")
    };
    let v = values(&[("CLAUDECODE", "1")]);
    assert_eq!((v["value"].as_str(), v["source"].as_str()), (Some("agent"), Some("CLAUDECODE=1 (Claude Code)")));
    let v = values(&[("CLAUDECODE", "1"), ("CODELAW_FORMAT", "json")]);
    assert_eq!((v["value"].as_str(), v["source"].as_str()), (Some("json"), Some("CODELAW_FORMAT")));
    let v = values(&[]);
    assert_eq!((v["value"].as_str(), v["source"].as_str()), (Some("text"), Some("default")));
}
