//! The commands beyond check and fix, through the binary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-cmd-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(args).current_dir(cwd).env_remove("VIRTUAL_ENV").output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap())
}

#[test]
fn init_writes_config_rules_folder_and_gitignore() {
    let d = temp("init");
    fs::write(d.join("package.json"), r#"{"name":"x","devDependencies":{"react":"19"}}"#).unwrap();
    fs::write(d.join("pnpm-lock.yaml"), "").unwrap();
    fs::write(d.join("pyproject.toml"), "[project]\nname = \"x\"\n").unwrap();
    fs::write(d.join("uv.lock"), "").unwrap();
    fs::create_dir_all(d.join("src")).unwrap();
    fs::write(d.join("src/app.tsx"), "export const A = () => <p />;\n").unwrap();
    fs::write(d.join("src/tool.py"), "x = 1\n").unwrap();
    fs::write(d.join(".gitignore"), "dist/").unwrap();
    let (code, out, err) = codelaw(&d, &["init", "--no-install"]);
    assert_eq!(code, 0, "{out}{err}");
    let config = fs::read_to_string(d.join("codelaw.toml")).unwrap();
    assert!(config.contains(r#"extends = ["codelaw:data/recommended", "codelaw:js/recommended", "codelaw:ts/recommended", "codelaw:react/recommended", "codelaw:py/recommended"]"#), "{config}");
    assert!(out.contains("pnpm add -D @codelaw/rules-data @codelaw/rules-js"), "{out}");
    assert!(out.contains("uv add --dev codelaw-rules-python"), "{out}");
    assert_eq!(fs::read_to_string(d.join(".gitignore")).unwrap(), "dist/\n.codelaw/cache/\n");
    assert!(d.join(".codelaw/rules/README.md").is_file());
    // A second init refuses.
    let (code, _, err) = codelaw(&d, &["init", "--no-install"]);
    assert_eq!(code, 2);
    assert!(err.contains("already configures codelaw"), "{err}");
    // --registry pypi puts every pack on PyPI.
    fs::remove_file(d.join("codelaw.toml")).unwrap();
    let (_, out, _) = codelaw(&d, &["init", "--no-install", "--registry", "pypi"]);
    assert!(out.contains("uv add --dev codelaw-rules-data codelaw-rules-js codelaw-rules-python"), "{out}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn new_rule_scaffold_passes_rules_lint_and_fails_test() {
    let d = temp("newrule");
    let (code, out, err) = codelaw(&d, &["new-rule", "acme/no-foo", "--lang", "typescript,python", "--fix", "manual"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = codelaw(&d, &["rules", "lint"]);
    assert_eq!(code, 0, "a fresh scaffold passes rules lint (SR-RULE-FORMAT veto): {out}{err}");
    assert!(out.contains("1 rule files · 0 problems"), "{out}");
    // The placeholder Don't (`TODO`) is what the placeholder matcher finds,
    // so the scaffold's own examples pass until the author replaces them.
    let (code, out, _) = codelaw(&d, &["test"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("✔ acme/no-foo"), "{out}");
    let (code, _, err) = codelaw(&d, &["new-rule", "acme/no-foo", "--lang", "python"]);
    assert_eq!(code, 2, "{err}");
    let _ = fs::remove_dir_all(&d);
}

#[test]
fn explain_tree_and_rules_why() {
    let d = temp("explain");
    let rules = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().join("fixtures/l3/rules/fixture/py-no-print.md");
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    fs::copy(rules, d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    let (code, out, _) = codelaw(&d, &["explain", "fixture/py-no-print"]);
    assert_eq!(code, 0);
    assert!(out.contains("How to fix:\n  1. Change it by hand."), "{out}");
    let (code, out, _) = codelaw(&d, &["explain", "fixture/py-no-prnt"]);
    assert_eq!(code, 2, "{out}");
    let (_, out, err) = codelaw(&d, &["explain", "fixture/py-no-prnt"]);
    assert!(err.contains("did you mean `fixture/py-no-print`?"), "{out}{err}");
    let (code, out, _) = codelaw(&d, &["tree", "a.py"]);
    assert_eq!(code, 0);
    assert!(out.contains("call  [1:1-1:9]") && out.contains("function: identifier  [1:1-1:6]  \"print\""), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--why", "fixture/py-no-print", "--for", "a.py"]);
    assert!(out.contains("source:     local") && out.contains("for a.py:  error"), "{out}");
    let (_, out, _) = codelaw(&d, &["rules", "--for", "a.py"]);
    assert!(out.starts_with("fixture/py-no-print"), "{out}");
    let (code, out, _) = codelaw(&d, &["check", "--format", "agent"]);
    assert_eq!(code, 1);
    assert!(out.starts_with("a.py:1:1 fixture/py-no-print print(1) in library code — fix: manual — Change it by hand."), "{out}");
    let _ = fs::remove_dir_all(&d);
}
