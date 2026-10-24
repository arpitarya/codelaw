//! W-14: the adapters through the binary (SR-LINTER decision 2,
//! SR-ARCHITECTURE decision 19). A stand-in `ruff` in the project's
//! `node_modules/.bin` prints what Ruff 0.16.9 prints, so no tool is needed.
#![cfg(unix)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-adapters-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .env_remove("CODELAW_SITE_PACKAGES")
        .output()
        .unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

const SOURCE: &str = "import os\nimport sys\n\nprint(sys.argv)\n";

/// Ruff 0.16.9's report for `a.py` while it still imports `os`: F401 with
/// its safe fix; nothing once the import is gone.
const REPORT: &str = r#"[{"cell":null,"code":"F401","end_location":{"column":10,"row":1},"filename":"ROOT/a.py","fix":{"applicability":"safe","edits":[{"content":"","end_location":{"column":1,"row":2},"location":{"column":1,"row":1}}],"message":"Remove unused import: `os`"},"location":{"column":8,"row":1},"message":"`os` imported but unused","name":"unused-import","noqa_row":1,"severity":"error","url":"https://docs.astral.sh/ruff/rules/unused-import"}]"#;

fn project(name: &str, config: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let d = temp(name);
    fs::write(d.join("codelaw.toml"), config).unwrap();
    fs::write(d.join("a.py"), SOURCE).unwrap();
    fs::create_dir_all(d.join("node_modules/.bin")).unwrap();
    let ruff = d.join("node_modules/.bin/ruff");
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in --version) echo 'ruff 0.16.9'; exit 0;; esac\nif grep -q '^import os' a.py; then echo '{}' | sed \"s|ROOT|$(pwd -P)|\"; else echo '[]'; fi\n",
        REPORT
    );
    fs::write(&ruff, script).unwrap();
    fs::set_permissions(&ruff, fs::Permissions::from_mode(0o755)).unwrap();
    d
}

fn diagnostics(out: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(out).unwrap_or_else(|e| panic!("{e}: {out}"));
    v["diagnostics"].as_array().unwrap().clone()
}

fn rules(d: &[Value]) -> Vec<&str> {
    d.iter().map(|x| x["rule"].as_str().unwrap()).collect()
}

#[test]
fn check_reports_an_installed_adapter_and_no_adapters_turns_it_off() {
    let d = project("check", "");
    let (code, out, err) = codelaw(&d, &["check", "--format", "json", "--no-cache"]);
    let found = diagnostics(&out);
    let f401 = found.iter().find(|x| x["rule"] == "adapter/ruff/F401").unwrap_or_else(|| panic!("{out}{err}"));
    assert_eq!(code, 1, "an adapter error is an error");
    assert_eq!(f401["fix"]["class"].as_str().or(f401["fix"].as_str()), Some("auto"), "{f401}");
    assert_eq!(f401["source"], "ruff@0.16.9");
    let (_, out, _) = codelaw(&d, &["check", "--format", "json", "--no-cache", "--no-adapters"]);
    assert!(!rules(&diagnostics(&out)).iter().any(|r| r.starts_with("adapter/")), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn the_config_picks_the_adapters_and_refuses_unknown_ones() {
    let d = project("config", "[adapters]\nenabled = [\"yamllint\"]\n");
    let (code, _, err) = codelaw(&d, &["check"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown adapter `yamllint`"), "{err}");
    fs::write(d.join("codelaw.toml"), "[adapters]\nenabled = []\n").unwrap();
    let (_, out, _) = codelaw(&d, &["check", "--format", "json", "--no-cache"]);
    assert!(!rules(&diagnostics(&out)).iter().any(|r| r.starts_with("adapter/")), "{out}");
    // Named and not installed: a warning, and the run goes on.
    fs::write(d.join("codelaw.toml"), "[adapters]\nenabled = [\"stylelint\"]\n").unwrap();
    let (code, out, _) = codelaw(&d, &["check", "--format", "json", "--no-cache"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("adapter `stylelint` is enabled in [adapters] but `stylelint` is not installed"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn fix_applies_the_adapters_safe_edits_and_dry_run_does_not() {
    let d = project("fix", "");
    let (_, out, err) = codelaw(&d, &["fix", "--dry-run", "--format", "json"]);
    assert_eq!(fs::read_to_string(d.join("a.py")).unwrap(), SOURCE, "{out}{err}");
    let (_, out, err) = codelaw(&d, &["fix"]);
    assert_eq!(fs::read_to_string(d.join("a.py")).unwrap(), "import sys\n\nprint(sys.argv)\n", "{out}{err}");
    assert!(out.contains("fixed a.py (adapter/ruff/F401 ×1)"), "{out}{err}");
    let (code, out, _) = codelaw(&d, &["check", "--format", "json"]);
    assert_eq!(code, 0, "{out}");
    let _ = fs::remove_dir_all(&d);
}

/// A finding a pack rule ported from the same tool rule also reports is
/// shown once, as codelaw's.
#[test]
fn a_ported_rule_and_its_adapter_report_once() {
    let d = project("ported", "extends = [\"codelaw:py/recommended\"]\npreview = true\n[rules]\n\"py/unused-import\" = \"error\"\n");
    let packs = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("packs/python");
    copy_dir(&packs, &d.join(".venv/lib/python3.12/site-packages/codelaw_rules_python")); // PyPI only (L6 decision 11)
    let (_, out, err) = codelaw(&d, &["check", "--format", "json", "--no-cache"]);
    let found = rules(&diagnostics(&out)).into_iter().map(str::to_owned).collect::<Vec<_>>();
    assert!(found.contains(&"py/unused-import".to_owned()), "{found:?}{err}");
    assert!(!found.contains(&"adapter/ruff/F401".to_owned()), "{found:?}");
    let _ = fs::remove_dir_all(&d);
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            fs::copy(e.path(), target).unwrap();
        }
    }
}
