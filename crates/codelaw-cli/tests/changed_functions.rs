//! `codelaw debug changed-functions` (W-71 step 1) on real git repos: new,
//! edited and untouched functions by token stream.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-chfn-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    fs::canonicalize(d).unwrap()
}

fn git(cwd: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.email=t@example.com", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

fn changed(cwd: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_codelaw"))
        .args(["debug", "changed-functions", "--format", "json", "--no-cache"])
        .args(args)
        .current_dir(cwd)
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}\n{}", String::from_utf8_lossy(&out.stderr)))
}

fn summary(v: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for f in v["files"].as_array().unwrap() {
        for c in f["functions"].as_array().unwrap() {
            out.push(format!("{}:{}:{}", f["path"].as_str().unwrap(), c["name"].as_str().unwrap(), c["change"].as_str().unwrap()));
        }
    }
    out
}

fn repo(name: &str) -> PathBuf {
    let d = temp(name);
    git(&d, &["init", "-q", "-b", "main"]);
    fs::write(d.join("a.ts"), "export function keep() { return 1; }\nexport function edit() { return 2; }\nexport function reformat(x: number) { return x + 1; }\nexport function oldName() { return 3; }\n").unwrap();
    fs::write(d.join("b.py"), "def untouched():\n    return 1\n").unwrap();
    git(&d, &["add", "."]);
    git(&d, &["commit", "-q", "-m", "base"]);
    d
}

#[test]
fn staged_judges_only_new_and_edited_functions_by_token_stream() {
    let d = repo("staged");
    fs::write(
        d.join("a.ts"),
        "// a header comment\nexport function keep() { return 1; }\nexport function edit() { return 99; }\nexport function reformat(x: number) {\n  return x + 1;\n}\nexport function newName() { return 3; }\nexport const arrow = () => 1;\n",
    )
    .unwrap();
    fs::write(d.join("c.py"), "def fresh():\n    pass\n\nclass K:\n    def m(self):\n        pass\n").unwrap();
    git(&d, &["add", "."]);
    let v = changed(&d, &["--staged"]);
    assert_eq!(
        summary(&v),
        [
            "a.ts:keep:untouched",
            "a.ts:edit:edited",
            "a.ts:reformat:untouched",
            "a.ts:newName:new",
            "a.ts:arrow:new",
            "c.py:fresh:new",
            "c.py:K.m:new",
        ]
    );
    assert_eq!(v["base"], "HEAD");
    assert!(v["files"][1]["new_file"].as_bool().unwrap());
}

#[test]
fn a_reformat_only_commit_judges_no_function() {
    let d = repo("reformat");
    git(&d, &["checkout", "-q", "-b", "topic"]);
    fs::write(d.join("a.ts"), "export function keep() {\n  return 1;\n}\n\nexport function edit() {\n  return 2;\n}\nexport function reformat(x: number) { return x + 1; }\nexport function oldName() {\n  return 3;\n}\n").unwrap();
    git(&d, &["commit", "-q", "-am", "prettier"]);
    let v = changed(&d, &["--changed-since", "main"]);
    assert!(summary(&v).iter().all(|s| s.ends_with(":untouched")), "{v}");
    assert_eq!(summary(&v).len(), 4);
}

#[test]
fn without_a_change_set_nothing_is_judged() {
    let d = repo("none");
    let v = changed(&d, &[]);
    assert!(v["base"].is_null());
    assert!(v["files"].as_array().unwrap().is_empty());
    let text = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["debug", "changed-functions"]).current_dir(&d).output().unwrap();
    assert!(String::from_utf8_lossy(&text.stdout).starts_with("new code: no change set; test rules not evaluated"));
}
