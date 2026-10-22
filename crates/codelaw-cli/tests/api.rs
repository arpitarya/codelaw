//! The npm and Python APIs (SR-PACKAGES decisions 3 and 5; L6 decision 5):
//! `check`, `fix`, `format` and `rulesFor` / `rules_for` return the
//! binary's JSON, identical between the two. Runs the sources in
//! `packaging/` against the binary under test through `CODELAW_BINARY`; a
//! host without `node` or `python3` skips its half and says so.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_path_buf()
}

fn have(tool: &str) -> bool {
    Command::new(tool).arg("--version").output().is_ok_and(|o| o.status.success())
}

fn run(cwd: &Path, tool: &str, args: &[&str]) -> Value {
    let out = Command::new(tool)
        .args(args)
        .current_dir(cwd)
        .env("CODELAW_BINARY", env!("CARGO_BIN_EXE_codelaw"))
        .env_remove("VIRTUAL_ENV")
        .env_remove("CODELAW_CACHE_DIR")
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("{tool}: {e}: {stdout}\n{}", String::from_utf8_lossy(&out.stderr)))
}

#[test]
fn both_apis_return_the_binary_json() {
    let d = std::env::temp_dir().join(format!("codelaw-api-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join(".codelaw/rules/fixture")).unwrap();
    let d = fs::canonicalize(d).unwrap();
    fs::copy(repo().join("fixtures/l3/rules/fixture/py-no-print.md"), d.join(".codelaw/rules/fixture/py-no-print.md")).unwrap();
    fs::write(d.join("a.py"), "print(1)\n").unwrap();
    fs::write(d.join("z.py"), "x=1\n").unwrap();

    let cli = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["rules", "--for", "a.py", "--format", "json"]).current_dir(&d).output().unwrap();
    let want_rules: Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(want_rules["rules"][0]["id"], "fixture/py-no-print");
    let cli = Command::new(env!("CARGO_BIN_EXE_codelaw")).args(["format", "--check", "--format", "json"]).current_dir(&d).output().unwrap();
    let want_format: Value = serde_json::from_slice(&cli.stdout).unwrap();
    assert_eq!(want_format["formatted"], serde_json::json!([{ "file": "z.py", "written": false }]), "{want_format}");

    let mut ran = Vec::new();
    if have("node") {
        let index = repo().join("packaging/npm/codelaw/index.js");
        let url = format!("file://{}", index.display());
        let script = format!(
            "import {{ check, fix, format, rulesFor }} from {url:?}; console.log(JSON.stringify({{ rules: rulesFor('a.py'), check: check([], {{ noCache: true }}), fix: fix([], {{ dryRun: true, noCache: true }}), format: format([], {{ check: true }}) }}));"
        );
        ran.push(("node", run(&d, "node", &["--input-type=module", "-e", &script])));
    } else {
        eprintln!("skipped: no node on this host");
    }
    if have("python3") {
        let src = repo().join("packaging/python/python");
        let script = format!(
            "import sys, json; sys.path.insert(0, {src:?}); import codelaw; print(json.dumps({{'rules': codelaw.rules_for('a.py'), 'check': codelaw.check(no_cache=True), 'fix': codelaw.fix(dry_run=True, no_cache=True), 'format': codelaw.format(check=True)}}))",
            src = src.display().to_string()
        );
        ran.push(("python3", run(&d, "python3", &["-I", "-c", &script])));
    } else {
        eprintln!("skipped: no python3 on this host");
    }
    for (tool, v) in &ran {
        assert_eq!(v["rules"], want_rules, "{tool} rulesFor");
        assert_eq!(v["check"]["diagnostics"][0]["rule"], "fixture/py-no-print", "{tool} check");
        assert_eq!(v["fix"]["schema_version"], 1, "{tool} fix");
        assert_eq!(v["format"], want_format, "{tool} format");
    }
    if let [(_, a), (_, b)] = ran.as_slice() {
        assert_eq!(a, b, "npm and Python return the same JSON (L6 decision 5)");
    }
    let _ = fs::remove_dir_all(&d);
}
