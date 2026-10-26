//! SR-CLI-MCP decision 12 (W-51) through the binary: `init --ci … --autofix`
//! writes the scheduled auto-fix workflow from its template, in a repo that is
//! already configured, never over a file that is not ours.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-ci-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("codelaw.toml"), "extends = []\n").unwrap();
    fs::canonicalize(d).unwrap()
}

fn init(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).arg("init").args(args).current_dir(cwd).env("NO_COLOR", "1").output().unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

fn actionlint(file: &Path) {
    // Run when the tool is installed; CI installs it.
    if let Ok(out) = Command::new("actionlint").arg(file).output() {
        assert!(out.status.success(), "actionlint: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    }
}

#[test]
fn github_weekly_writes_one_job_and_is_actionlint_clean() {
    let d = temp("gh");
    let (code, out) = init(&d, &["--ci", "github", "--autofix", "weekly"]);
    assert_eq!(code, 0, "{out}");
    let f = d.join(".github/workflows/codelaw-autofix.yml");
    let text = fs::read_to_string(&f).unwrap();
    assert!(text.contains(r#"cron: "17 4 * * 1""#));
    assert!(text.contains(&format!("codelaw@{}", env!("CARGO_PKG_VERSION"))));
    assert!(text.contains("contents: write") && text.contains("pull-requests: write"));
    assert!(text.contains(r#"--as-of "$day""#));
    assert!(text.contains("codelaw/autofix") && !text.contains("autofix-suggest"));
    assert!(!text.contains("\\{{"), "escapes are resolved");
    actionlint(&f);
    // A second run changes nothing.
    let (code, out) = init(&d, &["--ci", "github", "--autofix", "weekly"]);
    assert_eq!(code, 0);
    assert!(out.contains("already up to date"), "{out}");
}

#[test]
fn daily_and_unsafe_add_the_suggest_branch() {
    let d = temp("unsafe");
    let (code, out) = init(&d, &["--ci", "github", "--autofix", "daily", "--unsafe"]);
    assert_eq!(code, 0, "{out}");
    let f = d.join(".github/workflows/codelaw-autofix.yml");
    let text = fs::read_to_string(&f).unwrap();
    assert!(text.contains(r#"cron: "17 4 * * *""#));
    assert!(text.contains("codelaw/autofix-suggest") && text.contains("fix --unsafe"));
    actionlint(&f);
}

#[test]
fn gitlab_writes_an_includable_file() {
    let d = temp("gl");
    let (code, out) = init(&d, &["--ci", "gitlab", "--autofix", "weekly", "--unsafe"]);
    assert_eq!(code, 0, "{out}");
    let text = fs::read_to_string(d.join(".gitlab/codelaw-autofix.yml")).unwrap();
    assert!(text.contains("codelaw-autofix:") && text.contains("codelaw-autofix-suggest:") && text.contains("CODELAW_AUTOFIX_TOKEN"));
    assert!(out.contains("include:"), "{out}");
}

#[test]
fn refuses_a_file_that_is_not_ours_and_bad_values() {
    let d = temp("refuse");
    fs::create_dir_all(d.join(".github/workflows")).unwrap();
    fs::write(d.join(".github/workflows/codelaw-autofix.yml"), "name: mine\n").unwrap();
    let (code, _) = init(&d, &["--ci", "github", "--autofix", "weekly"]);
    assert_eq!(code, 2);
    assert_eq!(fs::read_to_string(d.join(".github/workflows/codelaw-autofix.yml")).unwrap(), "name: mine\n");
    assert_eq!(init(&d, &["--ci", "bitbucket", "--autofix", "weekly"]).0, 2);
    assert_eq!(init(&d, &["--ci", "github", "--autofix", "hourly"]).0, 2);
    assert_eq!(init(&d, &["--ci", "github"]).0, 2, "--ci needs --autofix");
}
