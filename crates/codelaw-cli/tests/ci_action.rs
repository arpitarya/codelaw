//! SR-CLI-MCP decision 12a (W-97) through the binary: `init --ci github`
//! without `--autofix` writes the check workflow that runs codelaw's GitHub
//! Action at this binary's version, from its template, never over a file that
//! is not ours. The Action itself is `action.yml`; CI's `action` job runs it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-ci-action-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::write(d.join("codelaw.toml"), "extends = []\n").unwrap();
    fs::canonicalize(d).unwrap()
}

fn init(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).arg("init").args(args).current_dir(cwd).env("NO_COLOR", "1").output().unwrap();
    (out.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)))
}

#[test]
fn github_alone_writes_the_check_workflow_at_this_version() {
    let d = temp("gh");
    let (code, out) = init(&d, &["--ci", "github"]);
    assert_eq!(code, 0, "{out}");
    let f = d.join(".github/workflows/codelaw.yml");
    let text = fs::read_to_string(&f).unwrap();
    assert!(text.contains(&format!("uses: arpitarya/codelaw@v{}", env!("CARGO_PKG_VERSION"))), "{text}");
    assert!(text.contains("contents: read") && text.contains("pull_request:"));
    assert!(!text.contains("{{"), "every slot filled");
    assert!(!d.join(".github/workflows/codelaw-autofix.yml").exists(), "no auto-fix workflow without --autofix");
    if let Ok(o) = Command::new("actionlint").arg(&f).output() {
        assert!(o.status.success(), "actionlint: {}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    }
    let (code, out) = init(&d, &["--ci", "github"]);
    assert_eq!(code, 0);
    assert!(out.contains("already up to date"), "{out}");
}

#[test]
fn refuses_a_file_that_is_not_ours_and_gitlab() {
    let d = temp("refuse");
    fs::create_dir_all(d.join(".github/workflows")).unwrap();
    fs::write(d.join(".github/workflows/codelaw.yml"), "name: mine\n").unwrap();
    assert_eq!(init(&d, &["--ci", "github"]).0, 2);
    assert_eq!(fs::read_to_string(d.join(".github/workflows/codelaw.yml")).unwrap(), "name: mine\n");
    let (code, out) = init(&d, &["--ci", "gitlab"]);
    assert_eq!(code, 2);
    assert!(out.contains("needs --autofix"), "{out}");
}

#[test]
fn the_action_installs_the_version_of_its_tag() {
    // scripts/version.py writes this default at every bump: the tag v<x> installs codelaw <x>.
    let action = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../action.yml")).unwrap();
    assert!(action.contains(&format!("default: '{}' # scripts/version.py writes this line", env!("CARGO_PKG_VERSION"))));
    assert!(action.contains("using: composite"));
    let steps = action.lines().filter(|l| !l.trim_start().starts_with('#'));
    assert!(steps.clone().all(|l| !l.contains("uses:")), "the Action uses no other action (L11)");
}
