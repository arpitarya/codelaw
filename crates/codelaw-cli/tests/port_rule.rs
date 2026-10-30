//! W-140: the `codelaw-port-rule` skill's three recorded ports, on
//! `fixtures/port-rule/` — ESLint's `no-nested-ternary`, Pylint's
//! `global-statement` and stylelint's `declaration-no-important`, each a
//! project rule green under `codelaw test` (SR-RULE-AUTHORING decision 2) —
//! and its first step, finding a rule by `ported_from:` through
//! `codelaw rules --format json` and MCP `list_rules`.

use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            copy_dir(&p, &to.join(e.file_name()));
        } else {
            fs::copy(&p, to.join(e.file_name())).unwrap();
        }
    }
}

fn command(cwd: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_codelaw"));
    c.current_dir(cwd).env_remove("VIRTUAL_ENV").env_remove("CODELAW_SITE_PACKAGES").env("CODELAW_CACHE_DIR", cwd.join(".codelaw/cache")).env("NO_COLOR", "1");
    c
}

fn codelaw(cwd: &Path, args: &[&str]) -> (i32, String) {
    let out = command(cwd).args(args).output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap())
}

/// The fixture, with `packs/css` installed beside it, in a fresh folder.
fn project(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("codelaw-port-rule-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    copy_dir(&repo().join("fixtures/port-rule"), &d);
    copy_dir(&repo().join("packs/css"), &d.join("node_modules/@codelaw/rules-css"));
    fs::canonicalize(&d).unwrap()
}

const PORTS: [(&str, &str); 3] = [
    ("acme/no-nested-ternary", "eslint/no-nested-ternary"),
    ("acme/no-global-statement", "pylint/global-statement"),
    ("acme/no-important", "stylelint/declaration-no-important"),
];

/// `ported_from` of `id` in a `rules` JSON object.
fn ported_from(rules: &Value, id: &str) -> Vec<String> {
    let card = rules["rules"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap_or_else(|| panic!("{id} not listed: {rules}"));
    card["ported_from"].as_array().map(|a| a.iter().map(|v| v.as_str().unwrap().to_owned()).collect()).unwrap_or_default()
}

#[test]
fn the_three_ports_are_green_under_codelaw_test() {
    let d = project("test");
    let (code, out) = codelaw(&d, &["rules", "lint"]);
    assert_eq!(code, 0, "{out}");
    for (id, _) in PORTS {
        let (code, out) = codelaw(&d, &["test", id]);
        assert_eq!(code, 0, "{id}: {out}");
        assert!(out.contains("0 failed"), "{id}: {out}");
    }
    // Each finds the one violation the fixture's src/ holds for it.
    for (id, file) in [("acme/no-nested-ternary", "src/label.js"), ("acme/no-global-statement", "src/hits.py"), ("acme/no-important", "src/notice.css")] {
        let (code, out) = codelaw(&d, &["check", "--no-adapters", "--rule", id, "--format", "json"]);
        assert_eq!(code, 1, "{id}: {out}");
        let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
        let found: Vec<&str> = v["diagnostics"].as_array().unwrap().iter().map(|d| d["path"].as_str().or(d["file"].as_str()).unwrap_or("?")).collect();
        assert_eq!(found, [file], "{id}");
    }
}

#[test]
fn a_rule_is_found_by_ported_from_in_the_cli_and_over_mcp() {
    let d = project("lookup");
    let (code, out) = codelaw(&d, &["rules", "--format", "json"]);
    assert_eq!(code, 0, "{out}");
    let rules: Value = serde_json::from_str(&out).unwrap();
    for (id, source) in PORTS {
        assert_eq!(ported_from(&rules, id), [source], "{id}");
    }
    // A first-party rule the preset turns on: the skill's step 2 stops here.
    assert!(ported_from(&rules, "css/declaration-block-no-duplicate-properties").contains(&"stylelint/declaration-block-no-duplicate-properties".to_owned()));

    // MCP `list_rules` gives the same cards.
    let mut child = command(&d).arg("mcp").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut request = |id: u64, method: &str, params: Value| -> Value {
        writeln!(stdin, "{}", json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })).unwrap();
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"))
    };
    request(1, "initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "port-rule", "version": "0" } }));
    let r = request(2, "tools/call", json!({ "name": "list_rules", "arguments": {} }));
    let listed = &r["result"]["structuredContent"];
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(ported_from(listed, "acme/no-global-statement"), ["pylint/global-statement"], "{r}");
    assert_eq!(listed["rules"], rules["rules"]);
}
